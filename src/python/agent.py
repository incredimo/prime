#!/usr/bin/env python3
import os, sys, subprocess, uuid, sqlite3, pathlib, json, importlib.util
import datetime, threading, textwrap, re, requests, time, asyncio
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, Request, Depends, Form
from fastapi.responses import HTMLResponse, JSONResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi.middleware.cors import CORSMiddleware
from sse_starlette.sse import EventSourceResponse
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
from infra.logger import log, get_recent_logs, add_log_listener, remove_log_listener

# --------------------------- CONFIG ---------------------------
WORKDIR    = pathlib.Path(__file__).resolve().parent
SKILL_DIR  = WORKDIR / "skills"
DB_PATH    = WORKDIR / "skills.db"
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://127.0.0.1:11434")
MODEL      = os.getenv("OLLAMA_MODEL", "gemma3")
API_PORT   = int(os.getenv("INFINITE_AI_PORT", 8000))
UI_PORT    = int(os.getenv("INFINITE_AI_UI_PORT", 8080))
# --------------------------------------------------------------

# ---------- Thread-safe SQLite connection ----------
_thread_local = threading.local()

def get_connection():
    if not hasattr(_thread_local, "conn"):
        _thread_local.conn = sqlite3.connect(DB_PATH)
    return _thread_local.conn

def get_cursor():
    return get_connection().cursor()

# ---------- memory ----------
def init_db():
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("""CREATE TABLE IF NOT EXISTS convo
               (id INTEGER PRIMARY KEY, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                role TEXT, content TEXT)""")
    cur.execute("""CREATE TABLE IF NOT EXISTS history
               (id INTEGER PRIMARY KEY, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                goal TEXT, status TEXT DEFAULT 'completed',
                output TEXT, duration INTEGER)""")
    conn.commit()

# Initialize the database
init_db()

def remember(r, c):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("INSERT INTO convo(role,content) VALUES(?,?)", (r,c))
    conn.commit()

def save_history(goal, status, output, duration):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("INSERT INTO history(goal,status,output,duration) VALUES(?,?,?,?)",
                (goal, status, output, duration))
    conn.commit()

def get_history(limit=50):
    conn = get_connection()
    cur = conn.cursor()
    cur.execute("SELECT id, ts, goal, status, output, duration FROM history ORDER BY ts DESC LIMIT ?",
                (limit,))
    columns = ['id', 'timestamp', 'goal', 'status', 'output', 'duration']
    return [dict(zip(columns, row)) for row in cur.fetchall()]

# Active tasks tracking
active_tasks = {}
ws_connections = set()

# ---------- Ollama wrapper ----------
def ollama_chat(prompt:str) -> str:
    remember("user", prompt)
    log(f"→AI PROMPT  {prompt[:120]}…")

    # Try to connect to Ollama service
    max_retries = 3
    for attempt in range(max_retries):
        try:
            # According to Ollama API docs, streaming is enabled by default
            # We'll handle both streaming and non-streaming responses
            payload = {
                "model": MODEL,
                "messages": [
                    {"role": "system", "content":
                        "You are an autonomous, root-capable agent inside WSL. "
                        "Return exactly one code block starting with #SH or #PY, "
                        "or #DONE when finished, or #SELFUPDATE followed by python code to replace agent.py."},
                    {"role": "user", "content": prompt}
                ],
                "stream": False  # Explicitly disable streaming to get a complete response
            }

            # Make the API request
            response = requests.post(f"{OLLAMA_URL}/api/chat", json=payload, timeout=600)
            response.raise_for_status()

            # Process the response
            try:
                # First try to parse as a single JSON object (non-streaming response)
                try:
                    response_json = response.json()
                    txt = response_json.get("message", {}).get("content", "").strip()

                    if not txt:
                        raise ValueError("Empty response from Ollama")

                    remember("assistant", txt)
                    log(f"←AI REPLY   {txt[:120]}…")
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
                    log(f"←AI REPLY   {full_content[:120]}…")
                    return full_content

            except Exception as e:
                log(f"Error processing Ollama response: {str(e)}")
                log(f"Response content: {response.text[:200]}")

                # Last resort: try to return any text content
                if response.text:
                    log("Returning raw response text as fallback")
                    txt = response.text.strip()
                    remember("assistant", txt)
                    return txt
                raise

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

# ---------- execution ----------
def extract(txt:str):
    m = re.search(r"^#(SH|PY)\s*\n(.*)", txt, re.S|re.M)
    return (m.group(1), textwrap.dedent(m.group(2))) if m else (None,None)

def run_sh(code:str) -> str:
    log(f"$ bash ‹‹\n{code}\n››")
    p = subprocess.run(code, shell=True, capture_output=True, text=True, timeout=1800)
    out = p.stdout + p.stderr
    log(out); return out

def run_py(code:str) -> str:
    tmp = SKILL_DIR / f"tmp_{uuid.uuid4().hex}.py"
    tmp.write_text(code)
    return run_sh(f"python {tmp}")

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

def iterate(goal:str, task_id=None):
    """Run the AI goal iteration loop"""
    start_time = time.time()
    full_output = []
    status = "completed"

    try:
        step = ollama_chat(f"Goal: {goal}")
        iteration = 1

        while True:
            # Update task status
            if task_id:
                active_tasks[task_id]["status"] = f"Running (step {iteration})"
                active_tasks[task_id]["output"] = "\n".join(full_output)
                asyncio.run(notify_websockets({
                    "type": "task_update",
                    "id": task_id,
                    "status": active_tasks[task_id]["status"],
                    "output": active_tasks[task_id]["output"],
                    "step": iteration
                }))

            if "#DONE" in step.upper():
                log("Goal complete.")
                full_output.append("✅ Goal completed successfully.")
                break

            if "#SELFUPDATE" in step.upper():
                new_code = step.split("#SELFUPDATE",1)[1].strip()
                (WORKDIR/"agent.py").write_text(new_code)
                log("Self-updated code. Restarting…")
                full_output.append("🔄 Agent self-updated. Restarting...")
                status = "restarting"
                os.execv(sys.executable, ["python", "agent.py"])

            kind, code = extract(step)
            if not kind:
                log("No code detected; abort.")
                full_output.append("❌ No executable code detected. Aborting.")
                status = "failed"
                break

            out = run_py(code) if kind=="PY" else run_sh(code)
            full_output.append(f"--- Step {iteration} ({kind}) ---")
            full_output.append(code)
            full_output.append(f"--- Output ---")
            full_output.append(out)

            step = ollama_chat(f"Output:\n{out}\nNext?")
            iteration += 1

    except Exception as e:
        log(f"Error during goal execution: {e}")
        full_output.append(f"❌ Error: {str(e)}")
        status = "failed"

    duration = int(time.time() - start_time)
    output_text = "\n".join(full_output)
    save_history(goal, status, output_text, duration)

    # Update and remove task if it was being tracked
    if task_id:
        active_tasks[task_id]["status"] = status
        active_tasks[task_id]["output"] = output_text
        active_tasks[task_id]["duration"] = duration
        asyncio.run(notify_websockets({
            "type": "task_complete",
            "id": task_id,
            "status": status,
            "output": output_text,
            "duration": duration
        }))

    return output_text

# ---------- API & UI App ----------
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

# API endpoints
@app.post("/api/goal")
async def api_goal(g: Goal):
    task_id = str(uuid.uuid4())
    active_tasks[task_id] = {
        "goal": g.text,
        "status": "starting",
        "output": "",
        "created": datetime.datetime.now().isoformat()
    }

    # Run the task in a background thread
    thread = threading.Thread(
        target=iterate,
        args=(g.text, task_id),
        daemon=True
    )
    thread.start()

    return {"id": task_id, "status": "started"}

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
    active_tasks[task_id] = {
        "goal": goal,
        "status": "starting",
        "output": "",
        "created": datetime.datetime.now().isoformat()
    }

    # Run the task in a background thread
    thread = threading.Thread(
        target=iterate,
        args=(goal, task_id),
        daemon=True
    )
    thread.start()

    return RedirectResponse(url=f"/task/{task_id}", status_code=303)

# CLI interface
def cli():
    while True:
        try: goal=input("\nGoal › ").strip()
        except EOFError: break
        if goal.lower() in {"exit","quit"}: break
        if goal: iterate(goal)

def start_ollama():
    """Try to start Ollama if it's not running"""
    log("Attempting to start Ollama...")

    # Try local binary first
    local_bin = WORKDIR / "bin" / "ollama"
    if local_bin.exists() and os.access(local_bin, os.X_OK):
        log(f"Starting Ollama from local binary: {local_bin}")
        subprocess.Popen([str(local_bin), "serve"],
                         stdout=open(WORKDIR/"logs/ollama_agent.log", "w"),
                         stderr=subprocess.STDOUT)
        time.sleep(5)
        return check_ollama()

    # Try system binary
    try:
        log("Starting Ollama from system path...")
        subprocess.Popen(["ollama", "serve"],
                         stdout=open(WORKDIR/"logs/ollama_agent.log", "w"),
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
