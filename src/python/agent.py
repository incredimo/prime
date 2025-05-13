#!/usr/bin/env python3
"""
Infinite AI Agent - An autonomous, root-capable agent running on Linux systems.
This agent can execute tasks, manage context, and interact with LLMs to accomplish goals.
"""

import os
import sys
import subprocess
import sqlite3
import pathlib
import json
import time
import datetime
import threading
import textwrap
import re
import requests
import asyncio
import uuid
import shlex
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, Request, Form
from fastapi.responses import HTMLResponse, RedirectResponse, FileResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi.middleware.cors import CORSMiddleware
from sse_starlette.sse import EventSourceResponse
from pydantic import BaseModel
from typing import Dict, Any, Optional, List, Tuple, Union
from infra.logger import log, get_recent_logs, add_log_listener, remove_log_listener

# =====================================================================
# CONFIGURATION
# =====================================================================

WORKDIR = pathlib.Path(__file__).resolve().parent
SKILL_DIR = WORKDIR / "skills"
DB_PATH = WORKDIR / "skills.db"
LOGS_DIR = WORKDIR / "logs"
TASK_LOGS_DIR = LOGS_DIR / "tasks"
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://127.0.0.1:11434")
MODEL = os.getenv("OLLAMA_MODEL", "gemma3")
API_PORT = int(os.getenv("prime_PORT", 8000))
UI_PORT = int(os.getenv("prime_UI_PORT", 8080))
MAX_CONTEXT = 4000  # approximate token limit for truncation
MAX_LOG_SIZE = 100000  # maximum number of characters to keep in a log file
LOG_TIMESTAMP_FORMAT = "%Y%m%d%H%M%S"  # Format for log filenames

# Ensure required directories exist
SKILL_DIR.mkdir(exist_ok=True)
LOGS_DIR.mkdir(exist_ok=True)
TASK_LOGS_DIR.mkdir(exist_ok=True)

# =====================================================================
# DATABASE MANAGEMENT
# =====================================================================

_thread_local = threading.local()

def get_connection():
    """Get a thread-local database connection"""
    if not hasattr(_thread_local, "conn"):
        _thread_local.conn = sqlite3.connect(str(DB_PATH))
        # Enable foreign keys
        _thread_local.conn.execute("PRAGMA foreign_keys = ON")
    return _thread_local.conn

def init_db():
    """Initialize the database schema"""
    conn = get_connection()
    cur = conn.cursor()
    
    # Main conversation table
    cur.execute("""
        CREATE TABLE IF NOT EXISTS convo (
            id INTEGER PRIMARY KEY,
            ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            role TEXT,
            content TEXT
        )
    """)
    
    # Task history table
    cur.execute("""
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            goal TEXT,
            status TEXT DEFAULT 'completed',
            output TEXT,
            duration INTEGER
        )
    """)
    
    # Task tracking table for sequential IDs
    cur.execute("""
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY, 
            task_id TEXT UNIQUE,
            goal TEXT,
            status TEXT DEFAULT 'pending',
            created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            environment TEXT
        )
    """)
    
    # Task log reference table
    cur.execute("""
        CREATE TABLE IF NOT EXISTS task_logs (
            id INTEGER PRIMARY KEY,
            task_id TEXT,
            log_type TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            filename TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks (task_id)
        )
    """)
    
    conn.commit()

# Initialize database on module load
init_db()

def remember(role, content):
    """Store a message in the conversation history"""
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("INSERT INTO convo(role, content) VALUES(?, ?)", (role, content))
    conn.commit()

def save_history(goal, status, output, duration):
    """Save a task execution to history"""
    conn = get_connection()
    cur = conn.cursor()
    # Cap output size to prevent database bloat
    output = output if len(output) < MAX_LOG_SIZE else output[:MAX_LOG_SIZE//2] + "\n...[truncated]...\n" + output[-MAX_LOG_SIZE//2:]
    cur.execute("INSERT INTO history(goal, status, output, duration) VALUES(?, ?, ?, ?)", 
               (goal, status, output, duration))
    conn.commit()

def get_history(limit=50):
    """Get recent task history"""
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("""
        SELECT id, ts, goal, status, output, duration 
        FROM history 
        ORDER BY ts DESC 
        LIMIT ?
    """, (limit,))
    
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

def get_next_task_id():
    """Generate sequential task IDs starting from 100000001"""
    conn = get_connection()
    cur = conn.cursor()
    
    # Check if tasks table has any entries
    cur.execute("SELECT COUNT(*) FROM tasks")
    count = cur.fetchone()[0]
    
    if count == 0:
        # No tasks yet, start with 100000001
        return "100000001"
    
    # Get the highest numerical ID
    cur.execute("SELECT MAX(CAST(task_id AS INTEGER)) FROM tasks")
    max_id = cur.fetchone()[0]
    
    if max_id is None:
        return "100000001"
    
    # Increment and return
    return str(int(max_id) + 1)

def register_task(task_id, goal, env_context=None):
    """Register a new task in the database"""
    conn = get_connection()
    cur = conn.cursor()
    
    # Convert environment context to JSON if provided
    env_json = json.dumps(env_context) if env_context else None
    
    cur.execute("""
        INSERT INTO tasks(task_id, goal, status, environment) 
        VALUES(?, ?, 'running', ?)
    """, (task_id, goal, env_json))
    conn.commit()

def update_task_status(task_id, status, env_context=None):
    """Update a task's status"""
    conn = get_connection()
    cur = conn.cursor()
    
    # Update environment if provided
    if env_context:
        env_json = json.dumps(env_context)
        cur.execute("""
            UPDATE tasks 
            SET status = ?, updated = CURRENT_TIMESTAMP, environment = ? 
            WHERE task_id = ?
        """, (status, env_json, task_id))
    else:
        cur.execute("""
            UPDATE tasks 
            SET status = ?, updated = CURRENT_TIMESTAMP
            WHERE task_id = ?
        """, (status, task_id))
    
    conn.commit()

def get_task_info(task_id):
    """Get detailed information about a task"""
    conn = get_connection()
    cur = conn.cursor()
    
    cur.execute("""
        SELECT task_id, goal, status, created, updated, environment
        FROM tasks
        WHERE task_id = ?
    """, (task_id,))
    
    row = cur.fetchone()
    if not row:
        return None
    
    # Parse environment JSON if available
    environment = json.loads(row[5]) if row[5] else None
    
    return {
        "task_id": row[0],
        "goal": row[1],
        "status": row[2],
        "created": row[3],
        "updated": row[4],
        "environment": environment
    }

def register_task_log(task_id, log_type, filename):
    """Register a log file in the database"""
    conn = get_connection()
    cur = conn.cursor()
    
    cur.execute("""
        INSERT INTO task_logs(task_id, log_type, filename)
        VALUES(?, ?, ?)
    """, (task_id, log_type, filename))
    
    conn.commit()

def get_task_logs(task_id):
    """Get all logs for a specific task"""
    conn = get_connection()
    cur = conn.cursor()
    
    cur.execute("""
        SELECT log_type, timestamp, filename
        FROM task_logs
        WHERE task_id = ?
        ORDER BY timestamp
    """, (task_id,))
    
    logs = []
    for row in cur.fetchall():
        logs.append({
            "type": row[0],
            "timestamp": row[1],
            "filename": row[2],
            "path": str(TASK_LOGS_DIR / task_id / row[2])
        })
    
    return logs

# =====================================================================
# LOGGING AND FILE MANAGEMENT
# =====================================================================

def save_task_log(task_id, log_type, content):
    """
    Save a log entry for a task with proper naming convention
    
    log_type options:
    - user_to_system: User input to the system
    - system_to_llm: System prompt to LLM
    - llm_to_system: LLM response to system
    - system_shell_execution: System shell execution
    - system_shell_output: Output from shell execution
    - system_function_execution: Function execution by the system
    - system_error: Error logs
    - system_task_complete: Task completion logs
    """
    # Create task directory if it doesn't exist
    task_dir = TASK_LOGS_DIR / task_id
    task_dir.mkdir(exist_ok=True, parents=True)
    
    # Create timestamp
    timestamp = datetime.datetime.now().strftime(LOG_TIMESTAMP_FORMAT)
    
    # Create filename with requested format
    filename = f"{task_id}_{timestamp}_{log_type}.md"
    filepath = task_dir / filename
    
    # Write content to file with proper markdown formatting
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(f"# Task {task_id} - {log_type.replace('_', ' ').title()}\n\n")
        f.write(f"Timestamp: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write("```\n")
        f.write(content)
        f.write("\n```\n")
    
    # Register log in database
    register_task_log(task_id, log_type, filename)
    
    # Real-time log to terminal as well
    log(f"[Task {task_id}] [{log_type}] Log saved: {filename}")
    
    return filepath

def read_file(path):
    """Read file contents with robust error handling"""
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return f.read()
    except UnicodeDecodeError:
        # Try alternate encoding if UTF-8 fails
        try:
            with open(path, 'r', encoding='latin-1') as f:
                return f.read()
        except Exception as e:
            return f"Error reading file {path} with latin-1 encoding: {str(e)}"
    except FileNotFoundError:
        return f"Error: File not found: {path}"
    except PermissionError:
        return f"Error: Permission denied when reading {path}"
    except Exception as e:
        return f"Error reading file {path}: {str(e)}"

def get_task_log_content(task_id, filename):
    """Get content of a specific log file"""
    filepath = TASK_LOGS_DIR / task_id / filename
    
    if not filepath.exists():
        return f"Error: Log file {filename} not found for task {task_id}"
    
    return read_file(filepath)

def get_task_conversation(task_id):
    """Build conversation history from task logs"""
    messages = []
    task_dir = TASK_LOGS_DIR / task_id
    
    if not task_dir.exists():
        return messages
    
    # Get all log files and sort by timestamp
    log_files = sorted(task_dir.glob(f"{task_id}_*_system_to_llm.md")) + \
                sorted(task_dir.glob(f"{task_id}_*_llm_to_system.md"))
    
    for file in log_files:
        filename = file.name
        parts = filename.replace('.md', '').split('_')
        
        # Skip if filename doesn't have the expected format
        if len(parts) < 4:
            continue
        
        log_type = '_'.join(parts[3:])
        
        try:
            content = read_file(file)
            # Extract content between triple backticks
            match = re.search(r"```\n(.*?)\n```", content, re.DOTALL)
            if match:
                message_content = match.group(1).strip()
                role = "user" if log_type == "system_to_llm" else "assistant"
                messages.append({"role": role, "content": message_content})
        except Exception as e:
            log(f"Error reading log file {file}: {e}")
    
    # Sort messages by timestamp from filename
    messages.sort(key=lambda msg: msg.get("timestamp", "0"))
    
    # If we have too many messages, keep only the most recent ones to stay within context limits
    total_chars = sum(len(msg.get("content", "")) for msg in messages)
    if total_chars > MAX_CONTEXT * 4:  # Approximate 4 chars per token
        # Always keep the first message for context
        first_msg = messages[0] if messages else None
        # Keep most recent messages that fit within the context window
        messages = messages[-10:]  # Start with last 10 messages
        if first_msg and first_msg not in messages:
            messages.insert(0, first_msg)
    
    return messages

# =====================================================================
# ENVIRONMENT CONTEXT AND SYSTEM INFORMATION
# =====================================================================

def get_environment_context():
    """
    Returns critical environment information
    
    This is one of the most important functions as it gives the LLM
    awareness of its execution environment.
    """
    context = {}
    
    try:
        # Get user info - critical for understanding privileges
        context['user'] = run_sh("whoami").strip()
        context['is_root'] = (context['user'] == 'root')
        
        # Get basic system info
        os_info = run_sh("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME || uname -a").strip()
        context['os_info'] = os_info
        
        # Get kernel info
        kernel = run_sh("uname -r").strip()
        context['kernel'] = kernel
        
        # Get available disk space
        disk_space = run_sh("df -h / | tail -n 1 | awk '{print $4}'").strip()
        context['free_disk_space'] = disk_space
        
        # Get memory info
        mem_info = run_sh("free -h | grep Mem | awk '{print $2\" total, \"$4\" free\"}'").strip()
        context['memory'] = mem_info
        
        # Get CPU info
        cpu_info = run_sh("grep 'model name' /proc/cpuinfo | head -n1 | cut -d: -f2").strip()
        context['cpu'] = cpu_info
        
        # Check common commands availability
        cmd_check = {}
        for cmd in ['apt', 'apt-get', 'yum', 'dnf', 'pip', 'python', 'docker', 'sudo']:
            cmd_check[cmd] = run_sh(f"command -v {cmd} >/dev/null 2>&1 && echo 'available' || echo 'not available'").strip()
        context['available_commands'] = cmd_check
        
        # Get working directory
        context['working_dir'] = run_sh("pwd").strip()
        
        # Check if docker is installed and running
        docker_status = run_sh("command -v docker >/dev/null 2>&1 && echo 'installed' || echo 'not installed'").strip()
        context['docker_status'] = docker_status
        
        if docker_status == 'installed':
            docker_running = run_sh("systemctl is-active docker 2>/dev/null || echo 'unknown'").strip()
            context['docker_running'] = docker_running
            
            if docker_running == 'active':
                # Get docker version if running
                docker_version = run_sh("docker --version").strip()
                context['docker_version'] = docker_version
        
        # Get network info
        ip_address = run_sh("hostname -I | awk '{print $1}'").strip()
        context['ip_address'] = ip_address
        
        # Get hostname
        hostname = run_sh("hostname").strip()
        context['hostname'] = hostname
        
    except Exception as e:
        log(f"Error getting environment context: {e}")
        context['error'] = str(e)
    
    return context

# =====================================================================
# CODE EXECUTION AND VALIDATION
# =====================================================================

def clean_code(code):
    """Clean up code by removing any backticks or markdown artifacts"""
    # Remove any trailing backticks that might have been included
    code = re.sub(r'`\s*$', '', code)
    # Remove any other markdown artifacts
    code = re.sub(r'^`.*$', '', code, flags=re.MULTILINE)
    return code.strip()

def run_sh(command, timeout=300):
    """
    Run shell command with proper error handling
    
    Args:
        command: The shell command to execute
        timeout: Maximum time in seconds (default: 5 minutes)
        
    Returns:
        Command output (stdout + stderr)
    """
    # Clean the command
    clean_command = clean_code(command)
    log(f"$ Executing shell command: {clean_command}")
    
    try:
        # Use check=False to capture all output regardless of exit code
        p = subprocess.run(
            clean_command, 
            shell=True, 
            capture_output=True, 
            text=True, 
            timeout=timeout
        )
        
        # Combine stdout and stderr
        output = p.stdout
        if p.stderr:
            if output:
                output += "\n" + p.stderr
            else:
                output = p.stderr
                
        # Log command exit code
        log(f"Command exit code: {p.returncode}")
        
        # Include non-zero exit code in output for visibility
        if p.returncode != 0:
            output_with_code = f"[Exit code: {p.returncode}]\n{output}"
            return output_with_code
            
        return output
        
    except subprocess.TimeoutExpired:
        log(f"Command timed out after {timeout} seconds: {clean_command}")
        return f"ERROR: Command timed out after {timeout} seconds"
        
    except Exception as e:
        log(f"Error running shell command: {e}")
        return f"ERROR: {str(e)}"

def run_py(code, task_id=None):
    """
    Run Python code in a separate process
    
    Args:
        code: Python code to execute
        task_id: Task ID for logging
        
    Returns:
        Execution output
    """
    # Clean the code
    clean_code_str = clean_code(code)
    
    # Generate a unique filename for this execution
    file_id = uuid.uuid4().hex
    tmp_file = SKILL_DIR / f"tmp_{file_id}.py"
    
    try:
        # Write code to temporary file
        with open(tmp_file, 'w', encoding='utf-8') as f:
            f.write(clean_code_str)
        
        # Execute the code
        result = run_sh(f"python {tmp_file}")
        
        # Handle syntax errors related to markdown artifacts
        if "SyntaxError: invalid syntax" in result and ("```" in result or "`" in result):
            log("Detected syntax error with backticks, attempting to fix...")
            
            # More aggressive cleaning
            cleaner_code = re.sub(r'```.*?```', '', clean_code_str, flags=re.DOTALL)
            cleaner_code = re.sub(r'`.*?`', '', cleaner_code)
            
            # Write the cleaned code and try again
            with open(tmp_file, 'w', encoding='utf-8') as f:
                f.write(cleaner_code)
                
            result = run_sh(f"python {tmp_file}")
        
        # Clean up temp file
        try:
            os.remove(tmp_file)
        except:
            pass
            
        return result
        
    except Exception as e:
        log(f"Error in Python execution: {e}")
        # Try to clean up temp file
        try:
            if tmp_file.exists():
                os.remove(tmp_file)
        except:
            pass
            
        return f"ERROR: {str(e)}"

def validate_code(code, kind):
    """
    Validate code before execution to identify security issues
    
    Args:
        code: The code to validate
        kind: Type of code (SH or PY)
        
    Returns:
        Tuple of (is_valid, reason)
    """
    # Don't execute empty code
    if not code.strip():
        return (False, "Empty code block")
    
    # Dangerous patterns for all code types
    dangerous_patterns = [
        (r"rm\s+-rf\s+/", "Dangerous recursive deletion of root directory"),
        (r"mkfs", "Filesystem formatting command detected"),
        (r"dd\s+if=.*\s+of=/dev/(sd|hd|nvme|xvd)", "Disk overwrite operation detected"),
        (r":\(\)\{\s+:\|\:\&\s+\};:", "Fork bomb detected"),
        (r">>/etc/passwd", "Modifying system password file"),
        (r"chmod\s+777\s+/", "Setting dangerous permissions on system directories"),
        (r"wget.*\|\s*bash", "Piping web content directly to bash"),
        (r"curl.*\|\s*bash", "Piping web content directly to bash"),
    ]
    
    # Check for dangerous patterns
    for pattern, reason in dangerous_patterns:
        if re.search(pattern, code):
            return (False, reason)
    
    # Additional Python-specific checks
    if kind == "PY":
        py_dangerous = [
            (r"__import__\(['\"]os['\"].*system", "Indirect os.system call"),
            (r"exec\s*\(.*input", "Executing user input"),
            (r"eval\s*\(.*input", "Evaluating user input"),
        ]
        
        for pattern, reason in py_dangerous:
            if re.search(pattern, code):
                return (False, reason)
        
        # Check for potentially dangerous imports
        dangerous_imports = {
            "subprocess": "subprocess module can execute shell commands",
        }
        
        for module, reason in dangerous_imports.items():
            if re.search(rf"import\s+{module}|from\s+{module}\s+import", code):
                # These aren't necessarily dangerous, but worth logging
                log(f"Warning: Code contains potentially security-sensitive module: {module}")
    
    # Shell-specific checks
    elif kind == "SH":
        # Check for usage of dangerous utilities
        dangerous_utils = [
            "shred", "fdisk", "mkfs", "sfdisk"
        ]
        
        for util in dangerous_utils:
            if re.search(rf"\b{util}\b", code):
                log(f"Warning: Shell code contains potentially destructive utility: {util}")
    
    return (True, "Code passed validation")

def extract_code_blocks(txt):
    """
    Extract code blocks from LLM output
    
    Args:
        txt: Text to extract code blocks from
        
    Returns:
        Tuple of (kind, code) where kind is SH, PY, or None
    """
    # First, try to extract code from markdown-style code blocks with backticks
    backtick_pattern = r"```(?:python|bash|sh)?\s*#(SH|PY)\s*\n(.*?)```"
    m_backticks = re.search(backtick_pattern, txt, re.DOTALL)
    
    if m_backticks:
        # Found code in backticks format
        return (m_backticks.group(1), textwrap.dedent(m_backticks.group(2)))
    
    # Alternative format with backticks but no explicit language
    alt_backtick = r"```\s*#(SH|PY)\s*\n(.*?)```"
    m_alt = re.search(alt_backtick, txt, re.DOTALL)
    
    if m_alt:
        return (m_alt.group(1), textwrap.dedent(m_alt.group(2)))
    
    # If no backtick format found, try the original format
    m = re.search(r"#(SH|PY)\s*\n(.*)", txt, re.DOTALL)
    if m:
        return (m.group(1), textwrap.dedent(m.group(2)))
    
    return (None, None)

# =====================================================================
# FUNCTION CALLING
# =====================================================================

def extract_functions(txt):
    """
    Extract function calls from LLM output
    
    Args:
        txt: Text to extract function calls from
        
    Returns:
        List of (function_name, args) tuples
    """
    # Look for #CALL function_name(arguments) pattern
    pattern = r"#CALL\s+(\w+)\s*\((.*?)\)"
    matches = re.findall(pattern, txt, re.DOTALL)
    
    # Process and clean the arguments
    result = []
    for name, args in matches:
        # Strip whitespace, handle quoted arguments
        cleaned_args = args.strip()
        result.append((name, cleaned_args))
    
    return result

def execute_functions(functions, task_id):
    """
    Execute functions extracted from LLM output
    
    Args:
        functions: List of (function_name, args) tuples
        task_id: Task ID for logging
        
    Returns:
        Tuple of (results, wait_time)
    """
    results = []
    
    for func_name, args in functions:
        log(f"Executing function {func_name}({args}) for task {task_id}")
        
        try:
            # Log function execution start
            save_task_log(task_id, "system_function_execution", 
                         f"Function: {func_name}({args})\nExecution start")
            
            if func_name == "read_file":
                # Read file content
                cleaned_args = args.strip('\'" \t')
                content = read_file(cleaned_args)
                
                # Truncate if too long
                if len(content) > 4000:
                    truncated = content[:2000] + "\n...[content truncated, showing 4000/{}]...\n".format(len(content)) + content[-2000:]
                    results.append(f"#FILE_CONTENT from {cleaned_args}\n{truncated}\n#END_FILE_CONTENT")
                else:
                    results.append(f"#FILE_CONTENT from {cleaned_args}\n{content}\n#END_FILE_CONTENT")
                
                # Log function result
                save_task_log(task_id, "system_function_execution", 
                             f"Function: read_file({cleaned_args})\n\nResult: Read {len(content)} characters")
                
            elif func_name == "list_directory":
                # List directory contents
                path = args.strip('\'" \t') or "."
                try:
                    # Use shlex to properly escape the path
                    safe_path = shlex.quote(path)
                    cmd = f"ls -la {safe_path} 2>&1"
                    dir_content = run_sh(cmd)
                    
                    results.append(f"Directory listing for {path}:\n{dir_content}")
                    
                    # Log function result
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: list_directory({path})\n\nResult: {len(dir_content.splitlines())} items")
                    
                except Exception as e:
                    error_msg = f"Error listing directory {path}: {e}"
                    results.append(error_msg)
                    
                    # Log function error
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: list_directory({path})\n\nError: {e}")
                
            elif func_name == "check_status":
                # Check status of a task
                target_task = args.strip('\'" \t') or task_id
                task_info = get_task_info(target_task)
                
                if task_info:
                    results.append(f"Task {target_task} status: {task_info['status']}")
                    
                    # Log function result
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: check_status({target_task})\n\nResult: Status is {task_info['status']}")
                else:
                    results.append(f"Task {target_task} not found")
                    
                    # Log function result
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: check_status({target_task})\n\nResult: Task not found")
                
            elif func_name == "wait":
                # Wait for specified time
                try:
                    wait_seconds = int(args.strip('\'" \t'))
                    max_wait = 60  # Cap at 60 seconds
                    wait_time = min(wait_seconds, max_wait)
                    
                    results.append(f"Waiting for {wait_time} seconds...")
                    
                    # Log function result
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: wait({wait_seconds})\n\nResult: Waiting for {wait_time} seconds")
                    
                    return (results, wait_time)
                    
                except ValueError:
                    results.append(f"Invalid wait duration: {args}. Please provide a number of seconds.")
                    
                    # Log function error
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: wait({args})\n\nError: Invalid wait duration")
                    
            elif func_name == "check_command":
                # Check if a command exists
                cmd = args.strip('\'" \t')
                if not cmd:
                    results.append("No command specified for check_command")
                else:
                    cmd_exists = run_sh(f"command -v {shlex.quote(cmd)} >/dev/null 2>&1 && echo 'available' || echo 'not available'").strip()
                    results.append(f"Command '{cmd}' is {cmd_exists}")
                    
                    # Log function result
                    save_task_log(task_id, "system_function_execution", 
                                 f"Function: check_command({cmd})\n\nResult: {cmd_exists}")
                    
            elif func_name == "get_environment":
                # Return full environment information
                env = get_environment_context()
                env_str = "Environment Information:\n"
                for key, value in env.items():
                    if isinstance(value, dict):
                        env_str += f"- {key}:\n"
                        for k, v in value.items():
                            env_str += f"  - {k}: {v}\n"
                    else:
                        env_str += f"- {key}: {value}\n"
                
                results.append(env_str)
                
                # Log function result
                save_task_log(task_id, "system_function_execution", 
                             f"Function: get_environment()\n\nResult: Retrieved {len(env)} environment variables")
                
            else:
                results.append(f"Unknown function: {func_name}")
                
                # Log function error
                save_task_log(task_id, "system_function_execution", 
                             f"Function: {func_name}({args})\n\nError: Unknown function")
                             
        except Exception as e:
            error_msg = f"Error executing function {func_name}: {str(e)}"
            results.append(error_msg)
            log(error_msg)
            
            # Log function error
            save_task_log(task_id, "system_function_execution", 
                         f"Function: {func_name}({args})\n\nError: {str(e)}")
    
    return (results, 0)  # Return with no wait time by default

# =====================================================================
# OLLAMA LLM INTERACTION
# =====================================================================

def ollama_chat(prompt, task_id, history=None):
    """
    Send a prompt to Ollama and get a response
    
    Args:
        prompt: The prompt to send
        task_id: Task ID for logging
        history: Optional conversation history
        
    Returns:
        LLM response text
    """
    remember("user", prompt)
    log(f"→ AI PROMPT: {prompt[:100]}...")
    
    # Log the full prompt
    save_task_log(task_id, "system_to_llm", prompt)
    
    # Prepare messages
    messages = []
    
    # Add system message
    system_content = """
    You are an autonomous, root-capable agent running on a Linux system. 
    Return exactly one code block starting with #SH or #PY, or #DONE when finished, 
    or #SELFUPDATE followed by python code to replace agent.py.
    You can also use #CALL function_name(args) to call special functions.
    """
    
    # Add history if provided
    history_messages = []
    if history:
        history_messages.extend(history)
    
    # Calculate approximate token count for history messages
    history_tokens = sum(len(msg.get("content", "")) for msg in history_messages) // 4
    
    # Truncate history if needed
    if history_tokens > MAX_CONTEXT:
        log(f"History too large ({history_tokens} tokens), truncating...")
        
        # Keep most recent messages
        while history_tokens > MAX_CONTEXT * 0.8 and history_messages:
            removed = history_messages.pop(0)
            history_tokens -= len(removed.get("content", "")) // 4
        
        # Add notice about truncation
        history_messages.insert(0, {
            "role": "system",
            "content": "Note: Earlier conversation history was truncated due to length."
        })
    
    # Add current prompt
    current_msg = {"role": "user", "content": prompt}
    
    # Make the API request
    max_retries = 3
    for attempt in range(max_retries):
        try:
            # Construct payload
            payload = {
                "model": MODEL,
                "messages": [
                    {"role": "system", "content": system_content}
                ] + history_messages + [current_msg],
                "stream": False
            }
            
            # Send request
            response = requests.post(
                f"{OLLAMA_URL}/api/chat", 
                json=payload, 
                timeout=600  # 10 minutes timeout
            )
            response.raise_for_status()
            
            # Process response
            response_json = response.json()
            txt = response_json.get("message", {}).get("content", "").strip()
            
            if not txt:
                raise ValueError("Empty response from Ollama")
            
            remember("assistant", txt)
            log(f"← AI REPLY: {txt[:100]}...")
            
            # Log the full response
            save_task_log(task_id, "llm_to_system", txt)
            
            return txt
            
        except requests.RequestException as e:
            if attempt < max_retries - 1:
                log(f"Request error (attempt {attempt+1}/{max_retries}): {e}")
                time.sleep(5)  # Wait 5 seconds before retrying
            else:
                log(f"Failed to communicate with Ollama after {max_retries} attempts: {e}")
                error_response = """#PY
print("ERROR: Failed to communicate with Ollama LLM.")
print(f"Make sure Ollama is running at {OLLAMA_URL}.")
"""
                save_task_log(task_id, "system_error", f"Failed to communicate with Ollama: {str(e)}")
                return error_response
                
        except Exception as e:
            log(f"Unexpected error communicating with Ollama: {e}")
            error_response = f"""#PY
print("ERROR: Unexpected error communicating with Ollama: {str(e)}")
"""
            save_task_log(task_id, "system_error", f"Unexpected error: {str(e)}")
            return error_response

# =====================================================================
# MAIN TASK EXECUTION FUNCTIONS
# =====================================================================

def wait_and_continue(goal, task_id, wait_time):
    """
    Wait for specified time and then continue task execution
    
    Args:
        goal: The goal being pursued
        task_id: Task ID
        wait_time: Time to wait in seconds
    """
    log(f"Waiting for {wait_time} seconds before continuing task {task_id}")
    
    # Update task status
    update_task_status(task_id, f"Waiting ({wait_time}s)")
    
    # Notify websockets
    asyncio.run(notify_websockets({
        "type": "task_update",
        "id": task_id,
        "status": f"Waiting ({wait_time}s)",
        "output": active_tasks[task_id].get("output", "")
    }))
    
    # Wait
    time.sleep(wait_time)
    
    # Continue execution
    prompt = f"""The wait period of {wait_time} seconds has completed.
Please continue with the goal: {goal}

Available functions:
- #CALL read_file(path) - Read file content
- #CALL list_directory(path) - List directory contents
- #CALL wait(seconds) - Wait before continuing
- #CALL check_command(cmd) - Check if a command is available
- #CALL get_environment() - Get complete environment information

Use #DONE when the goal is complete.
"""
    
    # Continue the task
    iterate_next_step(goal, task_id, prompt)

def process_response(response, goal, task_id):
    """
    Process a response from the LLM
    
    Args:
        response: LLM response text
        goal: The goal being pursued
        task_id: Task ID
    """
    # Get current task state
    if task_id not in active_tasks:
        log(f"Error: Task {task_id} not found in active tasks")
        return
        
    output = active_tasks[task_id].get("output", "")
    iteration = active_tasks[task_id].get("step", 0) + 1
    active_tasks[task_id]["step"] = iteration
    
    # Check for task completion
    if "#DONE" in response.upper():
        log(f"Task {task_id} complete")
        output += "\n✅ Goal completed successfully."
        active_tasks[task_id]["status"] = "completed"
        active_tasks[task_id]["output"] = output
        
        # Save task log
        save_task_log(task_id, "system_task_complete", "Goal completed successfully")
        
        # Update task status in database
        update_task_status(task_id, "completed")
        
        # Notify websockets
        asyncio.run(notify_websockets({
            "type": "task_complete",
            "id": task_id,
            "status": "completed",
            "output": output
        }))
        
        # Record history
        save_history(
            goal, 
            "completed", 
            output, 
            int(time.time() - active_tasks[task_id].get("start_time", time.time()))
        )
        return
    
    # Check for self-update
    if "#SELFUPDATE" in response.upper():
        new_code = response.split("#SELFUPDATE", 1)[1].strip()
        is_valid, reason = validate_code(new_code, "PY")
        
        if not is_valid:
            log(f"Self-update rejected: {reason}")
            output += f"\n❌ Self-update rejected: {reason}"
            active_tasks[task_id]["output"] = output
            
            # Save task log
            save_task_log(task_id, "system_selfupdate_rejected", f"Self-update rejected: {reason}")
            
            # Notify the LLM about the rejection
            prompt = f"""Self-update was rejected: {reason}
Please continue with the goal using standard code blocks instead of self-update.

Goal: {goal}
"""
            iterate_next_step(goal, task_id, prompt)
            return
        
        # Apply the self-update
        try:
            # Create backup
            backup_path = WORKDIR / f"agent.py.bak.{int(time.time())}"
            if (WORKDIR / "agent.py").exists():
                with open(WORKDIR / "agent.py", 'r', encoding='utf-8') as src:
                    with open(backup_path, 'w', encoding='utf-8') as dst:
                        dst.write(src.read())
            
            # Write new code
            with open(WORKDIR / "agent.py", 'w', encoding='utf-8') as f:
                f.write(new_code)
                
            log(f"Self-updated code. Backed up to {backup_path}")
            output += f"\n🔄 Agent self-updated. Backup saved to {backup_path}. Restarting..."
            active_tasks[task_id]["status"] = "restarting"
            active_tasks[task_id]["output"] = output
            
            # Save task log
            save_task_log(task_id, "system_selfupdate", f"Self-update applied. Backup saved to {backup_path}")
            
            # Update task status in database
            update_task_status(task_id, "restarting")
            
            # Notify websockets
            asyncio.run(notify_websockets({
                "type": "task_update",
                "id": task_id,
                "status": "restarting",
                "output": output
            }))
            
            # Record history
            save_history(
                goal, 
                "restarting", 
                output, 
                int(time.time() - active_tasks[task_id].get("start_time", time.time()))
            )
            
            # Restart the agent
            os.execv(sys.executable, [sys.executable, str(WORKDIR / "agent.py")])
            
        except Exception as e:
            log(f"Error during self-update: {e}")
            output += f"\n❌ Self-update failed: {str(e)}"
            active_tasks[task_id]["output"] = output
            
            # Save task log
            save_task_log(task_id, "system_error", f"Self-update failed: {str(e)}")
            
            # Notify the LLM about the failure
            prompt = f"""Self-update failed: {str(e)}
Please continue with the goal using standard code blocks instead of self-update.

Goal: {goal}
"""
            iterate_next_step(goal, task_id, prompt)
            return
    
    # Extract and execute functions
    functions = extract_functions(response)
    if functions:
        log(f"Found {len(functions)} function calls")
        func_results, wait_time = execute_functions(functions, task_id)
        
        # Add function results to output
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
            
            # Schedule continuation after wait
            threading.Thread(
                target=wait_and_continue,
                args=(goal, task_id, wait_time),
                daemon=True
            ).start()
            return
        
        # If no wait, continue with function results
        prompt = f"""Function call results:
{chr(10).join(func_results)}

Continue with the goal: {goal}

Available functions:
- #CALL read_file(path) - Read file content
- #CALL list_directory(path) - List directory contents
- #CALL wait(seconds) - Wait before continuing
- #CALL check_command(cmd) - Check if a command is available
- #CALL get_environment() - Get complete environment information

Use #DONE when the goal is complete.
"""
        iterate_next_step(goal, task_id, prompt)
        return
    
    # Extract code blocks
    kind, code = extract_code_blocks(response)
    if not kind:
        log(f"No code blocks or function calls detected in task {task_id}")
        output += "\n❌ No executable code or function calls detected. Please provide a code block starting with #SH or #PY, or use #CALL to call a function."
        active_tasks[task_id]["status"] = "awaiting_code"
        active_tasks[task_id]["output"] = output
        
        # Save task log
        save_task_log(task_id, "system_error", "No executable code or function calls detected")
        
        # Notify the LLM
        prompt = f"""I couldn't find any valid code blocks or function calls in your response.

Please provide a valid code block starting with #SH or #PY, or use #CALL to call a function.

Goal: {goal}

Available functions:
- #CALL read_file(path) - Read file content
- #CALL list_directory(path) - List directory contents
- #CALL wait(seconds) - Wait before continuing
- #CALL check_command(cmd) - Check if a command is available
- #CALL get_environment() - Get complete environment information
"""
        iterate_next_step(goal, task_id, prompt)
        return
    
    # Validate code before execution
    is_valid, reason = validate_code(code, kind)
    if not is_valid:
        log(f"Code validation failed for task {task_id}: {reason}")
        output += f"\n❌ Code validation failed: {reason}"
        active_tasks[task_id]["output"] = output
        
        # Save task log
        save_task_log(task_id, "system_code_validation_failed", f"Code validation failed: {reason}\n\nCode: {code}")
        
        # Notify the LLM about the validation failure
        prompt = f"""Code validation failed: {reason}
Please revise your approach and provide a safer solution.

Goal: {goal}

Available functions:
- #CALL read_file(path) - Read file content
- #CALL list_directory(path) - List directory contents
- #CALL wait(seconds) - Wait before continuing
- #CALL check_command(cmd) - Check if a command is available
- #CALL get_environment() - Get complete environment information
"""
        iterate_next_step(goal, task_id, prompt)
        return
    
    # Execute the code
    output += f"\n--- Step {iteration} ({kind}) ---\n"
    output += code
    active_tasks[task_id]["output"] = output
    
    # Update status before execution
    active_tasks[task_id]["status"] = f"Running (step {iteration})"
    update_task_status(task_id, f"Running (step {iteration})")
    
    # Notify websockets
    asyncio.run(notify_websockets({
        "type": "task_update",
        "id": task_id,
        "status": f"Running (step {iteration})",
        "output": output,
        "step": iteration
    }))
    
    # Save code execution log
    save_task_log(task_id, "system_shell_execution", f"Step {iteration} ({kind}):\n\n{code}")
    
    # Execute code
    if kind == "PY":
        out = run_py(code, task_id)
    else:  # SH
        out = run_sh(code)
        
    # Add output
    output += f"\n--- Output ---\n"
    output += out
    active_tasks[task_id]["output"] = output
    
    # Save execution output log
    save_task_log(task_id, "system_shell_output", f"Step {iteration} output:\n\n{out}")
    
    # Update environment context
    env_context = get_environment_context()
    
    # Update task environment in database
    update_task_status(task_id, f"Completed step {iteration}", env_context)
    
    # Compose prompt for next step
    env_summary = []
    important_keys = ['user', 'is_root', 'os_info', 'working_dir', 'docker_status']
    for key in important_keys:
        if key in env_context:
            value = env_context[key]
            env_summary.append(f"- {key}: {value}")
    
    # Add command availability
    if 'available_commands' in env_context and isinstance(env_context['available_commands'], dict):
        available = [cmd for cmd, status in env_context['available_commands'].items() if status == 'available']
        unavailable = [cmd for cmd, status in env_context['available_commands'].items() if status != 'available']
        if available:
            env_summary.append(f"- Available commands: {', '.join(available)}")
        if unavailable:
            env_summary.append(f"- Unavailable commands: {', '.join(unavailable)}")
    
    # Docker-specific info
    if 'docker_status' in env_context and env_context['docker_status'] == 'installed':
        if 'docker_running' in env_context:
            env_summary.append(f"- Docker running: {env_context['docker_running']}")
        if 'docker_version' in env_context:
            env_summary.append(f"- Docker version: {env_context['docker_version']}")
    
    # Add environment summary to the prompt
    environment_block = "\n".join(env_summary)
    
    # Prepare output for inclusion in prompt (truncate if too long)
    if len(out) > 4000:
        truncated_out = out[:2000] + f"\n...[output truncated, {len(out)} characters total]...\n" + out[-2000:]
    else:
        truncated_out = out
    
    next_prompt = f"""Output from step {iteration} ({kind}):

{truncated_out}

Current environment:
{environment_block}

Continue with the goal: {goal}

Available functions:
- #CALL read_file(path) - Read file content
- #CALL list_directory(path) - List directory contents
- #CALL wait(seconds) - Wait before continuing
- #CALL check_command(cmd) - Check if a command is available
- #CALL get_environment() - Get complete environment information

You can:
1. Execute another step with #SH or #PY
2. Call a function with #CALL
3. Finish with #DONE when complete
"""
    
    iterate_next_step(goal, task_id, next_prompt)

def iterate_next_step(goal, task_id, prompt, extract_history=True):
    """
    Continue task execution with a new prompt
    
    Args:
        goal: The goal being pursued
        task_id: Task ID
        prompt: Prompt for the LLM
        extract_history: Whether to include conversation history
    """
    # Get conversation history if requested
    history = get_task_conversation(task_id) if extract_history else None
    
    # Get LLM response
    response = ollama_chat(prompt, task_id, history)
    
    # Update task output
    if task_id in active_tasks:
        active_tasks[task_id]["last_response"] = response
    
    # Create a new thread to process this response
    thread = threading.Thread(
        target=process_response,
        args=(response, goal, task_id),
        daemon=True
    )
    thread.start()

def iterate(goal, task_id=None):
    """
    Start a new task execution
    
    Args:
        goal: The goal to pursue
        task_id: Optional task ID (will be generated if not provided)
        
    Returns:
        Dictionary with task ID and status
    """
    # Generate sequential task ID if none provided
    if not task_id:
        task_id = get_next_task_id()
    
    # Record start time
    start_time = time.time()
    
    # Initialize task in active tasks
    active_tasks[task_id] = {
        "goal": goal,
        "status": "starting",
        "output": "",
        "step": 0,
        "start_time": start_time
    }
    
    # Log task start
    save_task_log(task_id, "user_to_system", goal)
    
    try:
        # Get environment context
        env_context = get_environment_context()
        
        # Register task in database with environment context
        register_task(task_id, goal, env_context)
        
        # Store environment in active tasks
        active_tasks[task_id]["environment"] = env_context
        
        # Create command availability summary
        available_cmds = []
        unavailable_cmds = []
        if isinstance(env_context.get('available_commands'), dict):
            for cmd, status in env_context['available_commands'].items():
                if status == 'available':
                    available_cmds.append(cmd)
                else:
                    unavailable_cmds.append(cmd)
        
        # Format the system prompt
        system_prompt = f"""I am an autonomous agent on a Linux system.

GOAL: {goal}

CURRENT ENVIRONMENT:
- User: {env_context.get('user', 'unknown')}
- Root privileges: {env_context.get('is_root', False)}
- OS: {env_context.get('os_info', 'unknown')}
- Working directory: {env_context.get('working_dir', 'unknown')}
- Available commands: {', '.join(available_cmds)}
{"- Unavailable commands: " + ', '.join(unavailable_cmds) if unavailable_cmds else ""}

CRITICAL INSTRUCTIONS:
1. {'YOU ARE RUNNING AS ROOT - NEVER USE SUDO!' if env_context.get('is_root') else 'Use sudo for privileged operations'}
2. Always verify commands exist before using them
3. Return ONE code block per step, starting with:
   - #SH for shell commands, OR
   - #PY for Python code
4. You can call special functions:
   - #CALL read_file(path) - Read file content
   - #CALL list_directory(path) - List directory contents (use quotes around path)
   - #CALL wait(seconds) - Pause execution for a specified time
   - #CALL check_command(cmd) - Check if a command exists
   - #CALL get_environment() - Get detailed environment information
5. Return #DONE when the goal is completed

First, analyze the goal and break it down into executable steps.
"""
        
        # Update task with prompt
        active_tasks[task_id]["status"] = "prompting"
        active_tasks[task_id]["output"] = "Analyzing goal and creating execution plan..."
        
        # Notify websockets
        asyncio.run(notify_websockets({
            "type": "task_update",
            "id": task_id,
            "status": "prompting",
            "output": "Analyzing goal and creating execution plan...",
            "step": 0
        }))
        
        # Get initial response from LLM
        response = ollama_chat(system_prompt, task_id)
        
        # Store response in active tasks
        active_tasks[task_id]["output"] = response
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
        
        # Log error
        save_task_log(task_id, "system_error", f"Error starting task: {str(e)}")
        
        # Update task status in database
        update_task_status(task_id, "failed")
        
        # Record in history
        save_history(
            goal, 
            "failed", 
            f"❌ Error: {str(e)}", 
            int(time.time() - start_time)
        )
        
        return {"id": task_id, "status": "failed", "error": str(e)}

# =====================================================================
# WEBSOCKET COMMUNICATION
# =====================================================================

async def notify_websockets(data):
    """
    Send updates to all connected websockets
    
    Args:
        data: The data to send
    """
    disconnected = set()
    for ws in ws_connections:
        try:
            await ws.send_json(data)
        except Exception:
            disconnected.add(ws)
    
    # Remove disconnected websockets
    for ws in disconnected:
        if ws in ws_connections:
            ws_connections.remove(ws)

# =====================================================================
# FASTAPI APPLICATION
# =====================================================================

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
    result = iterate(g.text)
    return result

@app.get("/api/task/{task_id}")
async def get_task(task_id: str):
    # First check active tasks
    if task_id in active_tasks:
        task_data = active_tasks[task_id].copy()
        
        # Add database info
        db_info = get_task_info(task_id)
        if db_info:
            task_data.update({
                "created": db_info["created"],
                "updated": db_info["updated"],
                "environment": db_info["environment"]
            })
        
        return task_data
    
    # If not in active tasks, check database
    db_info = get_task_info(task_id)
    if db_info:
        # Task exists in database but not in memory
        return {
            "task_id": db_info["task_id"],
            "goal": db_info["goal"],
            "status": db_info["status"],
            "created": db_info["created"],
            "updated": db_info["updated"],
            "environment": db_info["environment"],
            "output": "Task data not in memory. Check logs for details."
        }
        
    # Task not found
    return JSONResponse(
        status_code=404,
        content={"error": f"Task {task_id} not found"}
    )

@app.post("/api/task/{task_id}/cancel")
async def cancel_task(task_id: str):
    if task_id not in active_tasks:
        return JSONResponse(
            status_code=404,
            content={"error": f"Task {task_id} not found or already completed"}
        )
    
    # Update task status
    active_tasks[task_id]["status"] = "cancelled"
    update_task_status(task_id, "cancelled")
    
    # Log cancellation
    save_task_log(task_id, "system_task_cancel", "Task cancelled by user")
    
    # Notify websockets
    await notify_websockets({
        "type": "task_update",
        "id": task_id,
        "status": "cancelled",
        "output": active_tasks[task_id].get("output", "") + "\n❌ Task cancelled by user."
    })
    
    return {"success": True, "message": "Task cancelled"}

@app.get("/api/tasks")
async def get_tasks():
    return active_tasks

@app.get("/api/history")
async def get_task_history(limit: int = 50):
    return get_history(limit)

@app.get("/api/logs")
async def get_logs(limit: int = 100):
    return get_recent_logs(limit)

@app.get("/api/task_logs/{task_id}")
async def get_task_logs_endpoint(task_id: str):
    logs = get_task_logs(task_id)
    return {
        "task_id": task_id,
        "logs": logs
    }

@app.get("/api/task_log/{task_id}/{filename}")
async def get_task_log_content_api(task_id: str, filename: str):
    log_path = TASK_LOGS_DIR / task_id / filename
    
    if not log_path.exists():
        return JSONResponse(
            status_code=404,
            content={"error": f"Log file {filename} not found for task {task_id}"}
        )
    
    content = read_file(log_path)
    return {"content": content, "filename": filename}

@app.get("/logs/{task_id}/{filename}")
async def get_log_file(task_id: str, filename: str):
    log_path = TASK_LOGS_DIR / task_id / filename
    
    if not log_path.exists():
        return HTMLResponse(content="Log file not found", status_code=404)
    
    return FileResponse(log_path)

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
    except Exception:
        pass
    
    return {
        "agent": "running",
        "ollama": ollama_status,
        "models": ollama_models,
        "current_model": MODEL,
        "active_tasks": len(active_tasks),
        "api_port": API_PORT,
        "ui_port": UI_PORT
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
            # Create task in event loop
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

@app.get("/task_logs/{task_id}", response_class=HTMLResponse)
async def task_logs_page(request: Request, task_id: str):
    logs = get_task_logs(task_id)
    return templates.TemplateResponse("task_logs.html", {"request": request, "task_id": task_id, "logs": logs})

@app.post("/submit")
async def submit_goal(goal: str = Form(...)):
    result = iterate(goal)
    return RedirectResponse(url=f"/task/{result['id']}", status_code=303)

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

# =====================================================================
# OLLAMA SERVICE MANAGEMENT
# =====================================================================

def start_ollama():
    """Try to start Ollama if it's not running"""
    log("Attempting to start Ollama...")
    
    # Try local binary first
    local_bin = WORKDIR / "bin" / "ollama"
    if local_bin.exists() and os.access(local_bin, os.X_OK):
        log(f"Starting Ollama from local binary: {local_bin}")
        LOGS_DIR.mkdir(exist_ok=True)
        subprocess.Popen([str(local_bin), "serve"],
                        stdout=open(LOGS_DIR/"ollama_agent.log", "w"),
                        stderr=subprocess.STDOUT)
        time.sleep(5)
        return check_ollama()
    
    # Try system binary
    try:
        log("Starting Ollama from system path...")
        LOGS_DIR.mkdir(exist_ok=True)
        subprocess.Popen(["ollama", "serve"],
                        stdout=open(LOGS_DIR/"ollama_agent.log", "w"),
                        stderr=subprocess.STDOUT)
        time.sleep(5)
        return check_ollama()
    except:
        log("Could not start Ollama. Please start it manually.")
        return False

def check_ollama():
    """Check if Ollama is running and models are available"""
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

# =====================================================================
# MAIN ENTRY POINT
# =====================================================================

if __name__ == "__main__":
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