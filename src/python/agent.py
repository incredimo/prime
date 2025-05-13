#!/usr/bin/env python3
import os, sys, subprocess, uuid, sqlite3, pathlib, json, time, datetime, threading, textwrap, re, requests, asyncio
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, Request, Form
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi.middleware.cors import CORSMiddleware
from sse_starlette.sse import EventSourceResponse
from pydantic import BaseModel
from typing import Dict, Any, Optional, List
from infra.logger import log, get_recent_logs, add_log_listener, remove_log_listener

# --------------------------- CONFIG ---------------------------

WORKDIR       = pathlib.Path(__file__).resolve().parent
SKILL_DIR     = WORKDIR / "skills"
DB_PATH       = WORKDIR / "skills.db"
OLLAMA_URL    = os.getenv("OLLAMA_URL", "http://127.0.0.1:11434")
MODEL         = os.getenv("OLLAMA_MODEL", "gemma3")
API_PORT      = int(os.getenv("INFINITE_AI_PORT", 8000))
UI_PORT       = int(os.getenv("INFINITE_AI_UI_PORT", 8080))
MESSAGES_DIR  = WORKDIR / "messages"
MAX_CONTEXT   = 4000  # approximate token limit for truncation
LOG_DIR       = WORKDIR / "logs"

# Ensure message storage and log directories exist
MESSAGES_DIR.mkdir(exist_ok=True)
LOG_DIR.mkdir(exist_ok=True)

# --------------------------- DB & Memory ---------------------------

_thread_local = threading.local()

def get_connection():
    if not hasattr(_thread_local, "conn"):
        _thread_local.conn = sqlite3.connect(DB_PATH)
    return _thread_local.conn

def init_db():
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("CREATE TABLE IF NOT EXISTS convo (id INTEGER PRIMARY KEY, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP, role TEXT, content TEXT)")
    cur.execute("CREATE TABLE IF NOT EXISTS history (id INTEGER PRIMARY KEY, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP, goal TEXT, status TEXT DEFAULT 'completed', output TEXT, duration INTEGER)")
    conn.commit()

init_db()

def save_history(goal, status, output, duration):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("INSERT INTO history(goal,status,output,duration) VALUES(?,?,?,?)", (goal, status, output, duration))
    conn.commit()

def get_history(limit=50):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("SELECT id, ts, goal, status, output, duration FROM history ORDER BY ts DESC LIMIT ?", (limit,))
    rows = cur.fetchall()
    result = []
    for row in rows:
        result.append({
            "id": row[0],
            "timestamp": row[1],
            "goal": row[2],
            "status": row[3],
            "output": row[4],
            "duration": row[5]
        })
    return result

def remember(role, content):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("INSERT INTO convo(role,content) VALUES(?,?)", (role, content))
    conn.commit()

# --------------------------- Message Persistence ---------------------------

def save_message(convo_id: str, direction: str, content: str):
    """Store each message (in/out/func_result) with timestamp and direction."""
    convo_path = MESSAGES_DIR / convo_id
    convo_path.mkdir(parents=True, exist_ok=True)
    
    # Create sortable timestamp (YYYYMMDD_HHMMSS_ms)
    ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S_%f")
    
    # Create filename with timestamp and direction
    filename = f"{ts}_{direction}.json"
    filepath = convo_path / filename
    
    # Save structured message data
    with open(filepath, 'w', encoding='utf-8') as f:
        json.dump({
            "timestamp": ts,
            "direction": direction,
            "content": content
        }, f, ensure_ascii=False)
    
    return filepath

def get_conversation_history(convo_id: str, max_tokens=MAX_CONTEXT):
    """Load message history for a conversation with truncation"""
    convo_path = MESSAGES_DIR / convo_id
    messages = []
    total_tokens = 0
    
    if not convo_path.exists():
        return messages
    
    # Get all files in sorted order (oldest to newest)
    files = sorted(convo_path.iterdir())
    
    # Process files from newest to oldest to prioritize recent context
    for file in reversed(files):
        try:
            data = json.loads(file.read_text(encoding='utf-8'))
            content = data["content"]
            direction = data["direction"]
            
            # Estimate tokens (1 token ≈ 4 characters)
            token_estimate = len(content) / 4
            
            # Check if adding this would exceed token limit
            if total_tokens + token_estimate > max_tokens:
                # Add a note about truncation at the beginning
                messages.insert(0, {
                    "role": "system", 
                    "content": f"[Note: {len(files) - len(messages)} earlier messages were truncated to stay within context limits]"
                })
                break
            
            # Add to messages (at beginning since we're processing newest to oldest)
            role = "assistant" if direction == "out" else "user"
            messages.insert(0, {"role": role, "content": content})
            total_tokens += token_estimate
            
        except Exception as e:
            log(f"Error loading message file {file}: {e}")
    
    return messages

# --------------------------- Environment Context ---------------------------

def get_environment_context():
    """Returns critical environment information"""
    context = {}
    
    try:
        # Get user info - critical for understanding privileges
        context['user'] = run_sh("whoami").strip()
        context['is_root'] = (context['user'] == 'root')
        
        # Get basic system info without overwhelming the LLM
        os_info = run_sh("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME || uname -a").strip()
        context['os_info'] = os_info
        
        # Check common commands
        cmd_check = run_sh("which apt apt-get yum dnf pip python docker 2>/dev/null").strip()
        context['available_commands'] = cmd_check
        
        # Get working directory
        context['working_dir'] = run_sh("pwd").strip()
        
    except Exception as e:
        log(f"Error getting environment context: {e}")
        context['error'] = str(e)
    
    return context

# --------------------------- Function Calling ---------------------------

def extract_functions(txt):
    """Extract function calls from LLM output"""
    pattern = r"#CALL\s+(\w+)\s*\((.*?)\)"
    matches = re.findall(pattern, txt, re.DOTALL)
    
    # Return list of (function_name, args) tuples
    return [(name, args.strip()) for name, args in matches]

def execute_functions(functions, task_id):
    """Execute functions extracted from LLM output"""
    results = []
    
    for func_name, args in functions:
        if func_name == "read_file":
            # Simple file reading with limits
            path = args.strip('"\'')
            try:
                content = read_file(path)
                # Truncate if too long
                if len(content) > 4000:  # Reasonable limit to avoid token explosion
                    content = content[:2000] + "\n...[content truncated]...\n" + content[-2000:]
                results.append(f"#FILE_CONTENT from {path}\n{content}\n#END_FILE_CONTENT")
            except Exception as e:
                results.append(f"Error reading file {path}: {e}")
                
        elif func_name == "check_status":
            # Check status of a running task
            t_id = args.strip('"\' ') or task_id
            if t_id in active_tasks:
                status = active_tasks[t_id]["status"]
                results.append(f"Task {t_id} status: {status}")
            else:
                results.append(f"Task {t_id} not found")
                
        elif func_name == "wait":
            # Implement wait functionality
            try:
                seconds = int(args.strip())
                max_wait = 60  # Cap at 60 seconds to prevent abuse
                wait_time = min(seconds, max_wait)
                results.append(f"Waiting for {wait_time} seconds...")
                return (results, wait_time)  # Special return for wait
            except ValueError:
                results.append(f"Invalid wait duration: {args}")
    
    return (results, 0)  # Normal return with no wait

def read_file(path: str) -> str:
    """Read file contents with error handling"""
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return f.read()
    except UnicodeDecodeError:
        # Try alternate encoding or binary mode
        try:
            with open(path, 'r', encoding='latin-1') as f:
                return f.read()
        except Exception as e:
            return f"Error reading file {path}: {e}"
    except Exception as e:
        return f"Error reading file {path}: {e}"

# --------------------------- Execution Helpers ---------------------------

def extract(txt: str):
    """Extract code blocks from LLM output"""
    # First, try to extract code from markdown-style code blocks with backticks
    backtick_pattern = r"```(?:python|bash|sh)?\s*#(SH|PY)\s*\n(.*?)```"
    m_backticks = re.search(backtick_pattern, txt, re.DOTALL)
    
    if m_backticks:
        # Found code in backticks format
        return (m_backticks.group(1), textwrap.dedent(m_backticks.group(2)))
    
    # If no backtick format found, try the original format
    m = re.search(r"^#(SH|PY)\s*\n(.*)", txt, re.DOTALL)
    return (m.group(1), textwrap.dedent(m.group(2))) if m else (None, None)

def clean_code(code: str) -> str:
    """Clean up code by removing any backticks or markdown artifacts"""
    # Remove any trailing backticks that might have been included
    code = re.sub(r'`\s*$', '', code)
    # Remove any other markdown artifacts that might cause issues
    code = re.sub(r'^`.*$', '', code, flags=re.MULTILINE)
    return code.strip()

def run_sh(code: str) -> str:
    """Run shell command with proper error handling"""
    # Clean the code before executing it
    clean_code_str = clean_code(code)
    log(f"$ bash <<\n{clean_code_str}\n>>")
    
    try:
        p = subprocess.run(clean_code_str, shell=True, capture_output=True, text=True, timeout=1800)
        out = p.stdout + p.stderr
        log(out)
        return out
    except subprocess.TimeoutExpired:
        log("Command timed out after 30 minutes")
        return "ERROR: Command timed out after 30 minutes"
    except Exception as e:
        log(f"Error running shell command: {e}")
        return f"ERROR: {str(e)}"

def run_py(code: str) -> str:
    """Run Python code with proper error handling"""
    # Clean the code before writing it to a file
    clean_code_str = clean_code(code)
    tmp = SKILL_DIR / f"tmp_{uuid.uuid4().hex}.py"
    tmp.write_text(clean_code_str)
    
    # Try to execute the code
    result = run_sh(f"python {tmp}")
    
    # If there's a syntax error related to backticks, try to fix and retry
    if "SyntaxError: invalid syntax" in result and "```" in result:
        log("Detected syntax error with backticks, attempting to fix...")
        # More aggressive cleaning
        cleaner_code = re.sub(r'```.*?```', '', clean_code_str, flags=re.DOTALL)
        cleaner_code = re.sub(r'`.*?`', '', cleaner_code)
        
        # Write the cleaned code and try again
        tmp.write_text(cleaner_code)
        result = run_sh(f"python {tmp}")
    
    return result

def validate_code(code: str, kind: str) -> tuple:
    """Validate code before execution to identify potential security issues"""
    dangerous_patterns = [
        r"rm\s+-rf\s+/", r"mkfs", r"dd\s+if=", r":\(\)\{\s+:\|\:\&\s+\};:", r">>/etc/passwd",
        r"chmod\s+777", r"wget.*\|\s*bash", r"curl.*\|\s*bash",
    ]
    
    for pattern in dangerous_patterns:
        if re.search(pattern, code):
            return (False, f"Potentially dangerous pattern detected: {pattern}")
    
    # Additional validation specific to Python
    if kind == "PY" and ("os.system(" in code or "subprocess.call(" in code):
        # Not necessarily dangerous, but worth flagging for review
        log("Warning: Nested command execution in Python code")
    
    return (True, "Code passed validation")

# --------------------------- Ollama Chat ---------------------------

def ollama_chat(prompt: str, history=None):
    """Enhanced chat function with history management and truncation"""
    remember("user", prompt)
    log(f"→AI PROMPT {prompt[:120]}…")
    
    # Prepare messages
    messages = []
    
    # Add history if provided
    if history:
        messages.extend(history)
    
    # Add current prompt
    messages.append({"role": "user", "content": prompt})
    
    # Calculate approximate token count
    total_tokens = sum(len(msg["content"]) / 4 for msg in messages)
    
    # Truncate history if needed
    if total_tokens > MAX_CONTEXT:
        log(f"Warning: Context too large ({int(total_tokens)} estimated tokens). Truncating...")
        
        # Always keep the last message (current prompt)
        current_prompt = messages.pop()
        
        # Remove oldest messages until we're under the limit
        while total_tokens > MAX_CONTEXT * 0.8 and len(messages) > 0:
            removed = messages.pop(0)
            total_tokens -= len(removed["content"]) / 4
        
        # Add truncation notice
        messages.insert(0, {
            "role": "system",
            "content": "Note: Some earlier messages were truncated to stay within context limits."
        })
        
        # Add back the current prompt
        messages.append(current_prompt)
    
    # Make the API request
    max_retries = 3
    for attempt in range(max_retries):
        try:
            # Check if the prompt contains a system message
            system_content = "You are an autonomous, root-capable agent running on a Linux system. Return exactly one code block starting with #SH or #PY, or #DONE when finished, or #SELFUPDATE followed by python code to replace agent.py."
            user_content = prompt
            
            # Check if the prompt contains a system message
            if prompt.startswith("System:"):
                parts = prompt.split("\nGoal:", 1)
                if len(parts) == 2:
                    system_content = parts[0].replace("System:", "").strip()
                    user_content = "Goal:" + parts[1].strip()
            
            # Construct payload
            payload = {
                "model": MODEL,
                "messages": [
                    {"role": "system", "content": system_content}
                ],
                "stream": False
            }
            
            # Add history messages if provided
            if history:
                for msg in history:
                    if msg["role"] != "system":  # Skip system messages in history
                        payload["messages"].append(msg)
            
            # Add current user message
            payload["messages"].append({"role": "user", "content": user_content})
            
            # Make the API request
            response = requests.post(f"{OLLAMA_URL}/api/chat", json=payload, timeout=600)
            response.raise_for_status()
            
            # Process the response
            try:
                # First try to parse as a single JSON object (non-streaming response)
                response_json = response.json()
                txt = response_json.get("message", {}).get("content", "").strip()
                
                if not txt:
                    raise ValueError("Empty response from Ollama")
                
                remember("assistant", txt)
                log(f"←AI REPLY {txt[:120]}…")
                return txt
                
            except json.JSONDecodeError:
                # If that fails, handle as a streaming response (multiple JSON objects)
                log("Handling streaming response format...")
                
                # Ollama streaming responses are newline-delimited JSON objects
                full_content = ""
                response_lines = response.text.strip().split('\n')
                
                for line in response_lines:
                    if not line.strip():
                        continue
                    
                    try:
                        line_json = json.loads(line)
                        
                        # Extract content from each message chunk
                        if "message" in line_json and "content" in line_json["message"]:
                            chunk_content = line_json["message"]["content"]
                            if chunk_content:  # Only add non-empty content
                                full_content += chunk_content
                    
                    except json.JSONDecodeError as json_err:
                        log(f"Warning: Could not parse line as JSON: {line[:50]}...")
                
                if not full_content:
                    log("No valid content found in streaming response")
                    # Try to extract any text content from the response
                    if response.text:
                        log("Attempting to extract text content directly")
                        return response.text.strip()
                    else:
                        raise ValueError("Empty response from Ollama")
                
                remember("assistant", full_content)
                log(f"←AI REPLY {full_content[:120]}…")
                return full_content
                
        except Exception as e:
            if attempt < max_retries - 1:
                log(f"Error connecting to Ollama (attempt {attempt+1}/{max_retries}): {str(e)}")
                log("Retrying in 5 seconds...")
                time.sleep(5)
            else:
                log(f"FATAL: Failed to connect to Ollama after {max_retries} attempts.")
                return f"""#PY
print("ERROR: Could not connect to Ollama LLM service.")
print("Make sure Ollama is running at {OLLAMA_URL}.")
print("You can:")
print("1. Start Ollama manually with 'ollama serve'")
print("2. Or use the Windows helper script in {WORKDIR}")
"""

# --------------------------- Main Iteration Loop ---------------------------

def wait_and_continue(goal, task_id, wait_time):
    """Wait specified time and then continue the task"""
    log(f"Waiting for {wait_time} seconds before continuing task {task_id}")
    time.sleep(wait_time)
    
    # Update task status
    if task_id in active_tasks:
        active_tasks[task_id]["status"] = f"Resuming after {wait_time}s wait"
        asyncio.run(notify_websockets({
            "type": "task_update",
            "id": task_id,
            "status": f"Resuming after {wait_time}s wait",
            "output": active_tasks[task_id].get("output", "")
        }))
    
    # Get the latest conversation history
    history = get_conversation_history(task_id)
    
    # Create prompt with wait completion notification
    prompt = f"""The wait period of {wait_time} seconds has completed.
Please continue with the goal: {goal}

You can check status or execution logs using #CALL read_file(path).
"""
    
    # Continue the iteration
    iterate_next_step(goal, task_id, prompt, history)

def iterate_next_step(goal, task_id, prompt, history=None):
    """Continue iteration with a new prompt"""
    # Send to LLM
    response = ollama_chat(prompt, history)
    save_message(task_id, "out", response)
    
    # Update task output
    if task_id in active_tasks:
        active_tasks[task_id]["output"] += f"\n--- Continued after wait ---\n{response}"
        active_tasks[task_id]["last_response"] = response
    
    # Create a new thread to process this response
    thread = threading.Thread(
        target=process_response,
        args=(response, goal, task_id),
        daemon=True
    )
    thread.start()

def process_response(response, goal, task_id):
    """Process a response from the LLM (execute code, handle functions, etc.)"""
    output = active_tasks[task_id].get("output", "")
    iteration = active_tasks[task_id].get("step", 0) + 1
    active_tasks[task_id]["step"] = iteration
    
    # Check for task completion
    if "#DONE" in response.upper():
        log(f"Task {task_id} complete.")
        output += "\n✅ Goal completed successfully."
        active_tasks[task_id]["status"] = "completed"
        active_tasks[task_id]["output"] = output
        
        # Notify websockets
        asyncio.run(notify_websockets({
            "type": "task_complete",
            "id": task_id,
            "status": "completed",
            "output": output
        }))
        
        save_history(goal, "completed", output, 
                   int(time.time() - active_tasks[task_id].get("start_time", time.time())))
        return
    
    # Check for self-update
    if "#SELFUPDATE" in response.upper():
        new_code = response.split("#SELFUPDATE", 1)[1].strip()
        is_valid, reason = validate_code(new_code, "PY")
        
        if not is_valid:
            output += f"\n❌ Self-update rejected: {reason}"
            active_tasks[task_id]["output"] = output
            
            # Notify the LLM about the rejection
            prompt = f"""Self-update was rejected: {reason}
Please continue with the goal using standard code blocks instead of self-update.
"""
            save_message(task_id, "in", prompt)
            history = get_conversation_history(task_id)
            iterate_next_step(goal, task_id, prompt, history)
            return
        
        # Apply the self-update
        try:
            (WORKDIR/"agent.py").write_text(new_code)
            log("Self-updated code. Restarting…")
            output += "\n🔄 Agent self-updated. Restarting..."
            active_tasks[task_id]["status"] = "restarting"
            active_tasks[task_id]["output"] = output
            
            # Notify websockets before restart
            asyncio.run(notify_websockets({
                "type": "task_update",
                "id": task_id,
                "status": "restarting",
                "output": output
            }))
            
            save_history(goal, "restarting", output, 
                       int(time.time() - active_tasks[task_id].get("start_time", time.time())))
            
            # Restart the agent
            os.execv(sys.executable, [sys.executable, "agent.py"])
        except Exception as e:
            log(f"Error during self-update: {e}")
            output += f"\n❌ Self-update failed: {str(e)}"
            active_tasks[task_id]["output"] = output
            
            # Notify the LLM about the failure
            prompt = f"""Self-update failed: {str(e)}
Please continue with the goal using standard code blocks instead of self-update.
"""
            save_message(task_id, "in", prompt)
            history = get_conversation_history(task_id)
            iterate_next_step(goal, task_id, prompt, history)
            return
    
    # Extract and execute functions
    functions = extract_functions(response)
    if functions:
        func_results, wait_time = execute_functions(functions, task_id)
        output += f"\n--- Function Results (Step {iteration}) ---\n"
        output += "\n".join(func_results)
        active_tasks[task_id]["output"] = output
        
        # Update websockets
        asyncio.run(notify_websockets({
            "type": "task_update",
            "id": task_id,
            "status": f"Running (step {iteration})",
            "output": output,
            "step": iteration
        }))
        
        # Handle wait function
        if wait_time > 0:
            log(f"Wait function called: {wait_time} seconds")
            active_tasks[task_id]["status"] = f"Waiting ({wait_time}s)"
            
            # Notify websockets about waiting
            asyncio.run(notify_websockets({
                "type": "task_update",
                "id": task_id,
                "status": f"Waiting ({wait_time}s)",
                "output": output
            }))
            
            # Schedule continuation after wait
            threading.Thread(
                target=wait_and_continue,
                args=(goal, task_id, wait_time),
                daemon=True
            ).start()
            return
        
        # If no wait, continue with function results
        prompt = f"""Function results:
{chr(10).join(func_results)}

Continue with the goal: {goal}
"""
        save_message(task_id, "in", prompt)
        history = get_conversation_history(task_id)
        iterate_next_step(goal, task_id, prompt, history)
        return
    
    # Extract code blocks
    kind, code = extract(response)
    if not kind:
        log(f"No code detected in task {task_id}; abort.")
        output += "\n❌ No executable code detected. Aborting."
        active_tasks[task_id]["status"] = "failed"
        active_tasks[task_id]["output"] = output
        
        # Notify websockets
        asyncio.run(notify_websockets({
            "type": "task_complete",
            "id": task_id,
            "status": "failed",
            "output": output
        }))
        
        save_history(goal, "failed", output, 
                   int(time.time() - active_tasks[task_id].get("start_time", time.time())))
        return
    
    # Validate code before execution
    is_valid, reason = validate_code(code, kind)
    if not is_valid:
        log(f"Code validation failed for task {task_id}: {reason}")
        output += f"\n❌ Code validation failed: {reason}"
        active_tasks[task_id]["output"] = output
        
        # Notify the LLM about the validation failure
        prompt = f"""Code validation failed: {reason}
Please revise your approach and provide a safer solution.

Goal: {goal}
"""
        save_message(task_id, "in", prompt)
        history = get_conversation_history(task_id)
        iterate_next_step(goal, task_id, prompt, history)
        return
    
    # Execute the code
    output += f"\n--- Step {iteration} ({kind}) ---\n"
    output += code
    output += f"\n--- Output ---\n"
    
    # Update status before execution
    active_tasks[task_id]["status"] = f"Running (step {iteration})"
    active_tasks[task_id]["output"] = output
    asyncio.run(notify_websockets({
        "type": "task_update",
        "id": task_id,
        "status": f"Running (step {iteration})",
        "output": output,
        "step": iteration
    }))
    
    # Execute code
    out = run_py(code) if kind == "PY" else run_sh(code)
    output += out
    active_tasks[task_id]["output"] = output
    
    # Update environment context
    env_context = get_environment_context()
    
    # Provide output to the LLM with environment context
    next_prompt = f"""Output from step {iteration}:

{out[:4000] if len(out) > 4000 else out}
{f"...[output truncated, {len(out)-4000} more characters]..." if len(out) > 4000 else ""}

Current environment:
- User: {env_context['user']}
- Root privileges: {env_context['is_root']}
- Working directory: {env_context['working_dir']}
- Available commands: {', '.join(env_context['available_commands'].split()) if env_context['available_commands'] else "unknown"}

Continue with the goal: {goal}

You can:
1. Execute another step with #SH or #PY
2. Read full output with #CALL read_file(path)
3. Wait with #CALL wait(seconds)
4. Finish with #DONE when complete
"""
    
    save_message(task_id, "in", next_prompt)
    history = get_conversation_history(task_id)
    iterate_next_step(goal, task_id, next_prompt, history)

def iterate(goal: str, task_id=None):
    """Run the AI goal iteration loop with improved environment awareness"""
    if not task_id:
        task_id = str(uuid.uuid4())
    
    start_time = time.time()
    active_tasks[task_id] = {
        "goal": goal,
        "status": "starting",
        "output": "",
        "step": 0,
        "start_time": start_time
    }
    
    try:
        # Get environment context
        env_context = get_environment_context()
        
        # Create initial system prompt with environment information
        system_prompt = f"""System: You are an autonomous agent on a Linux system.

Current environment:
- User: {env_context['user']}
- Root privileges: {env_context['is_root']}
- OS: {env_context['os_info']}
- Working directory: {env_context['working_dir']}
- Available commands: {', '.join(env_context['available_commands'].split()) if env_context['available_commands'] else "unknown"}

IMPORTANT INSTRUCTIONS:
1. If running as root, DO NOT use sudo - it's unnecessary and may not be installed
2. Always check if commands exist before using them
3. Provide ONE code block per step (#SH or #PY)
4. You can call functions:
   - #CALL read_file(path) - Read file content
   - #CALL wait(seconds) - Wait before continuing
   - #CALL check_status(task_id) - Check status of a task
5. Return #DONE when the goal is completed

Goal: {goal}

Please analyze this goal and break it down into executable steps.
First, check the environment to understand what commands and tools are available.
Then proceed with implementing the solution step by step.
"""
        
        # Save the prompt and get initial response
        save_message(task_id, "in", system_prompt)
        response = ollama_chat(system_prompt)
        save_message(task_id, "out", response)
        
        # Store response in active tasks
        active_tasks[task_id]["last_response"] = response
        
        # Process the response in a separate thread
        thread = threading.Thread(
            target=process_response,
            args=(response, goal, task_id),
            daemon=True
        )
        thread.start()
        
        return {"id": task_id, "status": "started"}
        
    except Exception as e:
        log(f"Error starting task: {e}")
        active_tasks[task_id]["status"] = "failed"
        active_tasks[task_id]["output"] = f"❌ Error: {str(e)}"
        
        save_history(goal, "failed", f"❌ Error: {str(e)}", 
                   int(time.time() - start_time))
        
        return {"id": task_id, "status": "failed", "error": str(e)}

# --------------------------- WebSocket Notifications ---------------------------

async def notify_websockets(data):
    """Send updates to all connected websockets"""
    disconnected = set()
    for ws in ws_connections:
        try:
            await ws.send_json(data)
        except:
            disconnected.add(ws)
    
    # Remove disconnected websockets
    for ws in disconnected:
        ws_connections.remove(ws)

# --------------------------- API & UI App ---------------------------

app = FastAPI(title="Infinite AI Agent")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Mount static files
app.mount("/static", StaticFiles(directory=f"{WORKDIR}/ui/static"), name="static")

# Setup templates
templates = Jinja2Templates(directory=f"{WORKDIR}/ui/templates")

# API models
class Goal(BaseModel):
    text: str

class TaskResponse(BaseModel):
    id: str
    status: str

# Active tasks tracking
active_tasks: Dict[str, Any] = {}
ws_connections = set()

# API endpoints
@app.post("/api/goal")
async def api_goal(g: Goal):
    task_id = str(uuid.uuid4())
    result = iterate(g.text, task_id)
    return result

@app.get("/api/task/{task_id}")
async def get_task(task_id: str):
    if task_id in active_tasks:
        return active_tasks[task_id]
    else:
        return {"error": "Task not found"}

@app.get("/api/tasks")
async def get_tasks():
    return active_tasks

@app.get("/api/history")
async def get_task_history(limit: int = 50):
    return get_history(limit)

@app.get("/api/logs")
async def get_logs(limit: int = 100):
    return get_recent_logs(limit)

@app.get("/api/status")
async def get_status():
    ollama_status = "unavailable"
    ollama_models = []
    
    try:
        r = requests.get(f"{OLLAMA_URL}/api/tags", timeout=3)
        if r.status_code == 200:
            ollama_status = "running"
            models_data = r.json().get("models", [])
            ollama_models = [m.get("name") for m in models_data if "name" in m]
    except:
        pass
    
    return {
        "agent": "running",
        "ollama": ollama_status,
        "models": ollama_models,
        "current_model": MODEL,
        "active_tasks": len(active_tasks)
    }

# WebSocket for real-time updates
@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    ws_connections.add(websocket)
    
    try:
        while True:
            # Keep connection alive
            await websocket.receive_text()
    except WebSocketDisconnect:
        if websocket in ws_connections:
            ws_connections.remove(websocket)

# Server-Sent Events for logs
@app.get("/logs/stream")
async def stream_logs(request: Request):
    async def event_generator():
        queue = asyncio.Queue()
        
        # Add listener that puts logs onto the queue
        def log_callback(entry):
            # Use create_task instead of run_coroutine_threadsafe
            loop = asyncio.get_event_loop()
            if loop.is_running():
                loop.create_task(queue.put(entry))
            else:
                # Fallback if no event loop is running
                asyncio.run_coroutine_threadsafe(
                    queue.put(entry),
                    loop
                )
        
        # Register the callback
        callback = add_log_listener(log_callback)
        
        try:
            # Send initial data
            yield {"event": "logs", "data": json.dumps(get_recent_logs(50))}
            
            # Stream updates
            while True:
                if await request.is_disconnected():
                    break
                
                # Wait for new logs with timeout
                try:
                    entry = await asyncio.wait_for(queue.get(), timeout=1.0)
                    yield {"event": "log", "data": json.dumps(entry)}
                except asyncio.TimeoutError:
                    # Send heartbeat
                    yield {"event": "heartbeat", "data": ""}
        finally:
            # Clean up
            remove_log_listener(callback)
    
    return EventSourceResponse(event_generator())

# UI routes
@app.get("/", response_class=HTMLResponse)
async def index(request: Request):
    return templates.TemplateResponse("index.html", {"request": request})

@app.get("/logs", response_class=HTMLResponse)
async def logs_page(request: Request):
    return templates.TemplateResponse("logs.html", {"request": request})

@app.get("/history", response_class=HTMLResponse)
async def history_page(request: Request):
    return templates.TemplateResponse("history.html", {"request": request})

@app.get("/task/{task_id}", response_class=HTMLResponse)
async def task_page(request: Request, task_id: str):
    return templates.TemplateResponse("task.html", {"request": request, "task_id": task_id})

@app.post("/submit")
async def submit_goal(goal: str = Form(...)):
    task_id = str(uuid.uuid4())
    iterate(goal, task_id)
    return RedirectResponse(url=f"/task/{task_id}", status_code=303)

# CLI interface
def cli():
    print("\n🤖 Infinite AI Agent - CLI Mode")
    print("---------------------------------")
    print("Type 'exit' or 'quit' to exit, or press Ctrl+D")
    print("Enter your goal below:")
    
    while True:
        try: 
            goal = input("\nGoal › ").strip()
            if goal.lower() in {"exit", "quit"}:
                break
            if goal:
                result = iterate(goal)
                print(f"Task started with ID: {result['id']}")
        except EOFError:
            break
        except Exception as e:
            print(f"Error: {e}")

def start_ollama():
    """Try to start Ollama if it's not running"""
    log("Attempting to start Ollama...")
    
    # Try local binary first
    local_bin = WORKDIR / "bin" / "ollama"
    if local_bin.exists() and os.access(local_bin, os.X_OK):
        log(f"Starting Ollama from local binary: {local_bin}")
        LOG_DIR.mkdir(exist_ok=True)
        subprocess.Popen([str(local_bin), "serve"],
                        stdout=open(LOG_DIR/"ollama_agent.log", "w"),
                        stderr=subprocess.STDOUT)
        time.sleep(5)
        return check_ollama()
    
    # Try system binary
    try:
        log("Starting Ollama from system path...")
        LOG_DIR.mkdir(exist_ok=True)
        subprocess.Popen(["ollama", "serve"],
                        stdout=open(LOG_DIR/"ollama_agent.log", "w"),
                        stderr=subprocess.STDOUT)
        time.sleep(5)
        return check_ollama()
    except:
        log("Could not start Ollama. Please start it manually.")
        return False

def check_ollama():
    """Check if Ollama is running"""
    try:
        response = requests.get(f"{OLLAMA_URL}/api/tags", timeout=5)
        response.raise_for_status()
        models = response.json().get("models", [])
        model_names = [m.get("name") for m in models if "name" in m]
        log(f"Available models: {', '.join(model_names) if model_names else 'none'}")
        
        # Check if our model exists
        if not any(MODEL in name for name in model_names):
            log(f"Warning: Model {MODEL} not found in available models")
            return False
        return True
    except Exception as e:
        log(f"Ollama check failed: {e}")
        return False

if __name__=="__main__":
    # Check if Ollama is running and has our model
    if not check_ollama():
        log(f"Ollama is not running or missing model {MODEL}. Attempting to start...")
        start_ollama()
        
        # If still not working, try a different model
        if not check_ollama() and MODEL == "gemma3":
            log("Trying with a different model...")
            MODEL = "llama2"
            if not check_ollama():
                log("Still having issues with Ollama. Please check manually.")
    
    # Start Web UI in a separate thread
    ui_thread = threading.Thread(
        target=lambda: __import__("uvicorn").run(
            app, host="0.0.0.0", port=UI_PORT, log_level="info"
        ),
        daemon=True
    )
    ui_thread.start()
    log(f"🌐 Web UI started at http://localhost:{UI_PORT}")
    
    # Start API server in the main thread
    log(f"🚀 Starting API server on port {API_PORT}")
    __import__("uvicorn").run(app, host="0.0.0.0", port=API_PORT)