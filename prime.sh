#!/usr/bin/env bash
#
# prime.sh   —  WSL-aware, Ollama-powered, gemma3 agent with UI
# --------------------------------------------------------------------
set -euo pipefail
IFS=$'\n\t'
export DEBIAN_FRONTEND=noninteractive

ME="$(whoami)"
WORKDIR="$HOME/infinite_ai"
LOG="$WORKDIR/install.log"
# Determine if we need sudo
need_sudo() {
  if [ "$ME" = "root" ]; then
    return 1  # False, no sudo needed
  else
    if command -v sudo >/dev/null 2>&1; then
      return 0  # True, sudo exists and needed
    else
      echo "Error: Not running as root and sudo not available. Please install sudo or run as root."
      exit 1
    fi
  fi
}

# Function to run commands with sudo only if needed
run_elevated() {
  if need_sudo; then
    sudo "$@"
  else
    "$@"
  fi
}

# Function to log messages
log(){ printf "[%(%F %T)T] %s\n" -1 "$*" | tee -a "$LOG" ; }

# Create directories and set proper permissions
mkdir -p "$WORKDIR" "$WORKDIR/bin" "$WORKDIR/logs" "$WORKDIR/ui" "$WORKDIR/tmp"
chmod 755 "$WORKDIR"
chmod 755 "$WORKDIR/bin"


# Clean old setup if requested
if [[ "$*" == *"--clean"* ]] || [[ "$*" == *"-c"* ]]; then
  echo "🧹 Cleaning old installation..."
  run_elevated pkill -f ollama 2>/dev/null || true
  pkill -f "python.*agent.py" 2>/dev/null || true
  run_elevated rm -rf "$WORKDIR" 2>/dev/null || true
  run_elevated rm -f /etc/sudoers.d/90-$ME-ai 2>/dev/null || true
  echo "✅ Cleanup complete. Starting fresh installation."
fi


install_packages() {
  log "Installing system prerequisites..."
  run_elevated apt-get update -y
  run_elevated apt-get install -y --no-install-recommends \
    python3 python3-venv python3-pip git curl wget build-essential \
    sqlite3 jq unzip net-tools htop tmux lsof nodejs npm

  # Check if Node.js is too old, install newer version if needed
  NODE_VERSION=$(node -v 2>/dev/null | cut -d'v' -f2 || echo "0.0.0")
  if [[ "$(echo "$NODE_VERSION" | cut -d'.' -f1)" -lt "14" ]]; then
    log "Node.js is too old ($NODE_VERSION). Installing newer version..."
    if need_sudo; then
      curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
      sudo apt-get install -y nodejs
    else
      curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
      apt-get install -y nodejs
    fi
  fi
}

# ------------------------------------------------------------
# 1.  password-less sudo setup (skip if already root)
# ------------------------------------------------------------
if need_sudo; then
  log "Configuring password-less sudo for $ME…"
  sudo bash -c "echo '$ME ALL=(ALL) NOPASSWD:ALL' >/etc/sudoers.d/90-$ME-ai && chmod 0440 /etc/sudoers.d/90-$ME-ai"
else
  log "Running as root, skipping sudo configuration..."
fi

# ------------------------------------------------------------
# 2.  base system packages
# ------------------------------------------------------------
install_packages
# ------------------------------------------------------------
# 3.  FUCKING DOWNLOAD AND INSTALL OLLAMA BY BRUTE FORCE
# ------------------------------------------------------------
OLLAMA_ENDPOINT="http://127.0.0.1:11434"
OLLAMA_BIN="$WORKDIR/bin/ollama"
log "🔥 AGGRESSIVELY INSTALLING OLLAMA - WILL NOT FAIL THIS TIME 🔥"

# Kill any existing Ollama processes
log "Killing any existing Ollama processes..."
pkill -f ollama 2>/dev/null || true
sudo pkill -f ollama 2>/dev/null || true
sleep 2

# Check for port conflicts
if lsof -i:11434 -sTCP:LISTEN 2>/dev/null; then
  log "⚠️ Warning: Port 11434 is in use by another process!"
  log "Attempting to kill the process..."
  sudo lsof -i:11434 -sTCP:LISTEN -t | xargs sudo kill -9 2>/dev/null || true
  sleep 2
fi

# ATTEMPT 1: Try using the pre-installed Ollama
SYSTEM_OLLAMA=$(which ollama 2>/dev/null || echo "")
if [[ -n "$SYSTEM_OLLAMA" && -x "$SYSTEM_OLLAMA" ]]; then
  log "System has Ollama already installed at: $SYSTEM_OLLAMA"
  log "Creating symlink to our bin directory..."
  ln -sf "$SYSTEM_OLLAMA" "$OLLAMA_BIN" || {
    log "Warning: Failed to create symlink. Using direct system path instead."
    OLLAMA_BIN="$SYSTEM_OLLAMA"
  }
else
  # ATTEMPT 2: Try downloading pre-compiled binary directly
  log "Downloading Ollama binary directly..."
  curl -fsSL "https://github.com/ollama/ollama/releases/latest/download/ollama-linux-amd64" -o "$OLLAMA_BIN" || {
    log "Direct download failed. Trying alternate URL..."
    curl -fsSL "https://ollama.com/download/ollama-linux-amd64" -o "$OLLAMA_BIN" || {
      log "Alternate download failed. Will try official install script."
      OFFICIAL_SCRIPT=true
    }
  }

  # If direct download was successful, make it executable
  if [[ -f "$OLLAMA_BIN" && ! -x "$OLLAMA_BIN" ]]; then
    chmod +x "$OLLAMA_BIN"
    log "Made Ollama binary executable."
  else
    # ATTEMPT 3: Try official install script
    if [[ "${OFFICIAL_SCRIPT:-false}" == "true" ]]; then
      log "Using official install script..."
      curl -fsSL https://ollama.com/install.sh | sh
      
      # Find where it was installed
      SYSTEM_OLLAMA=$(which ollama 2>/dev/null || echo "")
      if [[ -n "$SYSTEM_OLLAMA" && -x "$SYSTEM_OLLAMA" ]]; then
        log "Official script installed Ollama at: $SYSTEM_OLLAMA"
        # Create a symlink in our bin directory
        ln -sf "$SYSTEM_OLLAMA" "$OLLAMA_BIN" || {
          log "Warning: Failed to create symlink. Using direct system path instead."
          OLLAMA_BIN="$SYSTEM_OLLAMA"
        }
      else
        log "Official script failed! Final attempt: manual compilation..."
        
        # ATTEMPT 4: Manual build from source with proper permissions
        TMP_BUILD="$WORKDIR/tmp/ollama_build"
        rm -rf "$TMP_BUILD" 2>/dev/null || true
        mkdir -p "$TMP_BUILD"
        cd "$TMP_BUILD"
        
        # Install Go if needed
        if ! command -v go &>/dev/null; then
          log "Installing Go for compilation..."
          GO_VERSION="1.21.0"
          wget "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz"
          sudo rm -rf /usr/local/go
          sudo tar -C /usr/local -xzf "go${GO_VERSION}.linux-amd64.tar.gz"
          export PATH=$PATH:/usr/local/go/bin
        fi
        
        # Clone and build Ollama
        log "Cloning and building Ollama from source..."
        git clone https://github.com/ollama/ollama.git
        cd ollama
        go build
        
        if [[ -f "./ollama" && -x "./ollama" ]]; then
          log "Successfully built Ollama from source!"
          # Use sudo to copy the binary to ensure we have permissions
          sudo cp "./ollama" "$OLLAMA_BIN"
          sudo chown "$ME:$ME" "$OLLAMA_BIN"
          sudo chmod +x "$OLLAMA_BIN"
        else
          log "CRITICAL FAILURE: All Ollama installation methods failed!"
          log "Will proceed with setup, but you'll need to install Ollama manually."
        fi
        
        # Return to workdir
        cd "$WORKDIR"
      fi
    fi
  fi
fi

# Check if we have a working Ollama binary
if [[ -f "$OLLAMA_BIN" && -x "$OLLAMA_BIN" ]]; then
  log "Ollama binary is ready at: $OLLAMA_BIN"
  run_elevated chmod +x "$OLLAMA_BIN"  # Ensure it's executable
else
  log "Checking for system-wide Ollama..."
  SYSTEM_OLLAMA=$(which ollama 2>/dev/null || echo "")
  if [[ -n "$SYSTEM_OLLAMA" && -x "$SYSTEM_OLLAMA" ]]; then
    log "Using system Ollama: $SYSTEM_OLLAMA"
    OLLAMA_BIN="$SYSTEM_OLLAMA"
  else
    log "🚨 WARNING: No working Ollama binary found!"
    log "🔄 Will continue with setup, but you need to run the Windows helper script to install Ollama on Windows."
  fi
fi


# ------------------------------------------------------------
# 4.  Start Ollama service - WITH MULTIPLE APPROACHES
# ------------------------------------------------------------
log "Starting Ollama service..."

# First check if it's already running (e.g., on Windows side)
if curl -s --max-time 3 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
  log "Ollama service already running and accessible."
  OLLAMA_RUNNING=true
else
  OLLAMA_RUNNING=false
  
  # If we have a binary, try to start it
  if [[ -x "$OLLAMA_BIN" ]]; then
    # Try to start it directly
    log "Launching Ollama service from: $OLLAMA_BIN"
    nohup "$OLLAMA_BIN" serve > "$WORKDIR/logs/ollama.log" 2>&1 &
    OLLAMA_PID=$!
    log "Started Ollama with PID $OLLAMA_PID"
    
    # Wait for it to start
    MAX_ATTEMPTS=10
    for i in $(seq 1 $MAX_ATTEMPTS); do
      log "Checking Ollama service (attempt $i/$MAX_ATTEMPTS)..."
      if curl -s --max-time 3 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
        log "🎉 SUCCESS! Ollama service is now running! 🎉"
        OLLAMA_RUNNING=true
        break
      fi
      sleep 3
    done
    
    # If still not running, try with elevated privileges
    if [[ "${OLLAMA_RUNNING:-false}" == "false" ]]; then
      log "Failed to start Ollama normally. Trying with elevated privileges..."
      run_elevated pkill -f ollama 2>/dev/null || true
      sleep 2
      run_elevated nohup "$OLLAMA_BIN" serve > "$WORKDIR/logs/ollama_elevated.log" 2>&1 &
      OLLAMA_PID=$!
      log "Started Ollama with elevated privileges (PID: $OLLAMA_PID)"
      
      # Wait again
      for i in $(seq 1 5); do
        log "Checking Ollama service with elevated privileges (attempt $i/5)..."
        if curl -s --max-time 3 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
          log "🎉 SUCCESS! Ollama service is now running with elevated privileges! 🎉"
          OLLAMA_RUNNING=true
          break
        fi
        sleep 3
      done
    fi
  fi
  
  # If still not running, check if it's running on Windows side
  if [[ "$OLLAMA_RUNNING" == "false" ]]; then
    log "Checking if Ollama is running on Windows side..."
    WIN_IP=$(ip route | grep default | awk '{print $3}' 2>/dev/null || echo "localhost")
    WINDOWS_OLLAMA_URL="http://$WIN_IP:11434"
    
    if curl -s --max-time 3 "$WINDOWS_OLLAMA_URL/api/tags" >/dev/null 2>&1; then
      log "🎉 Ollama found running on Windows side at $WINDOWS_OLLAMA_URL"
      OLLAMA_URL="$WINDOWS_OLLAMA_URL"
      OLLAMA_RUNNING=true
      # Export this for the agent to use
      export OLLAMA_URL="$WINDOWS_OLLAMA_URL"
    else
      log "Could not find Ollama on Windows side either."
    fi
  fi
fi

# Final check
if curl -s --max-time 3 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
  log "Ollama service is confirmed running."
  OLLAMA_RUNNING=true
else
  log "⚠️ Warning: Could not start Ollama service."
  log "Will create Windows helper script to run Ollama there."
  OLLAMA_RUNNING=false
fi

# ------------------------------------------------------------
# 5.  Pull gemma3 model if Ollama is running
# ------------------------------------------------------------
if [[ "${OLLAMA_RUNNING:-false}" == "true" ]]; then
  log "Checking for gemma3 model..."
  if ! curl -s "$OLLAMA_ENDPOINT/api/tags" | grep -q '"name":"gemma3"'; then
    log "Pulling gemma3 model… (this may take a while)"
    if [[ -x "$OLLAMA_BIN" ]]; then
      "$OLLAMA_BIN" pull gemma3 || {
        log "Failed pulling gemma3. Will try later."
      }
    else
      ollama pull gemma3 || {
        log "Failed pulling gemma3. Will try later."
      }
    fi
  else
    log "gemma3 model already present. ✓"
  fi
else
  log "Skipping model pull since Ollama is not running."
  log "Once Ollama is running, pull the model with: ollama pull gemma3"
fi

# ------------------------------------------------------------
# 6.  Create Windows helper script 
# ------------------------------------------------------------
cat > "$WORKDIR/windows_ollama_helper.ps1" <<'PS1'
# Windows PowerShell script to install and run Ollama
# Save this to your Windows system and run with PowerShell

Write-Host "Windows Ollama Helper for WSL"
Write-Host "============================"

# Check if Ollama is already installed
$ollamaPath = "$env:LOCALAPPDATA\Ollama\ollama.exe"
if (Test-Path $ollamaPath) {
    Write-Host "Ollama is already installed at: $ollamaPath"
} else {
    Write-Host "Ollama is not installed. Installing now..."
    
    # Download and run the Ollama installer
    $installerUrl = "https://ollama.com/download/ollama-windows-amd64.msi"
    $installerPath = "$env:TEMP\ollama-installer.msi"
    
    Write-Host "Downloading Ollama installer..."
    Invoke-WebRequest -Uri $installerUrl -OutFile $installerPath
    
    Write-Host "Running Ollama installer..."
    Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$installerPath`" /quiet" -Wait
    
    Write-Host "Ollama installation completed."
}

# Check if Ollama service is running
$ollamaRunning = $false
try {
    $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -TimeoutSec 2 -ErrorAction SilentlyContinue
    if ($response.StatusCode -eq 200) {
        $ollamaRunning = $true
        Write-Host "Ollama service is already running."
    }
} catch {
    Write-Host "Ollama service is not running."
}

# Start Ollama if it's not running
if (-not $ollamaRunning) {
    Write-Host "Starting Ollama service..."
    Start-Process -FilePath $ollamaPath -ArgumentList "serve" -WindowStyle Hidden
    
    # Wait for Ollama to start
    $attempts = 0
    $maxAttempts = 10
    $started = $false
    
    Write-Host "Waiting for Ollama service to become available..."
    while ($attempts -lt $maxAttempts -and -not $started) {
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -TimeoutSec 2 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                $started = $true
                Write-Host "Ollama service is now running and accessible."
            }
        } catch {
            $attempts++
            Write-Host "Waiting for Ollama service (attempt $attempts/$maxAttempts)..."
            Start-Sleep -Seconds 2
        }
    }
    
    if (-not $started) {
        Write-Host "Warning: Could not confirm Ollama is running. Check manually."
    }
}

# Check for gemma3 model
try {
    $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -ErrorAction SilentlyContinue
    $models = ConvertFrom-Json $response.Content
    
    $hasgemma3 = $false
    foreach ($model in $models.models) {
        if ($model.name -eq "gemma3") {
            $hasgemma3 = $true
            break
        }
    }
    
    if ($hasgemma3) {
        Write-Host "gemma3 model is already installed."
    } else {
        Write-Host "gemma3 model is not installed. Installing now..."
        Start-Process -FilePath "$env:LOCALAPPDATA\Ollama\ollama.exe" -ArgumentList "pull gemma3" -Wait
    }
} catch {
    Write-Host "Could not check for installed models."
}

Write-Host ""
Write-Host "Ollama should now be accessible from WSL at http://localhost:11434"
Write-Host "You can run your infinite_ai agent in WSL now."
PS1

# Copy to Windows accessible location
WIN_PATH="/mnt/c/repo/prime/windows_ollama_helper.ps1"
cp "$WORKDIR/windows_ollama_helper.ps1" "$WIN_PATH" 2>/dev/null || true
log "Created Windows helper script at: $WIN_PATH"


# ------------------------------------------------------------
# 7.  python environment
# ------------------------------------------------------------
log "Setting up Python environment..."
cd "$WORKDIR" || { log "Failed to change to $WORKDIR"; exit 1; }

# Ensure necessary directories exist
mkdir -p "$WORKDIR"/{skills,logs,infra,ui/static,ui/templates}

log "Setting up Python venv..."
python3 -m venv venv
source venv/bin/activate

log "Installing Python libs..."
pip install --upgrade pip
pip install fastapi uvicorn duckdb tiktoken watchdog requests jinja2 aiofiles websockets python-multipart sse-starlette


# ------------------------------------------------------------
# 8.  immutable logger (never self-modified)
# ------------------------------------------------------------
cat > infra/logger.py <<'PY'
import datetime, pathlib, sys, os, json, threading, queue

WORKDIR = pathlib.Path(__file__).resolve().parents[1]
LOG_DIR = WORKDIR / "logs"
LOG_DIR.mkdir(exist_ok=True)

# Create a queue for logs that can be consumed by the UI
log_queue = queue.Queue(maxsize=1000)  # Limit to prevent memory issues
log_listeners = set()

def log(msg: str, level="INFO"):
    """Log a message to console, file, and make it available to UI"""
    stamp = datetime.datetime.now().strftime("%F %T")
    line = f"[{stamp}] {msg}"
    print(line, flush=True)
    
    # Write to log file
    with open(LOG_DIR / f"agent_{datetime.date.today()}.log", "a") as f:
        f.write(line + "\n")
    
    # Add to queue for UI
    log_entry = {
        "timestamp": stamp,
        "message": msg,
        "level": level
    }
    
    try:
        log_queue.put_nowait(log_entry)
    except queue.Full:
        # If queue is full, remove oldest item
        try:
            log_queue.get_nowait()
            log_queue.put_nowait(log_entry)
        except:
            pass  # If concurrent access issues, just skip this log for the queue
    
    # Notify all listeners
    for callback in log_listeners:
        try:
            callback(log_entry)
        except:
            pass  # Ignore errors in callbacks

def get_recent_logs(limit=100):
    """Get recent logs for UI display"""
    logs = []
    # Copy from queue without removing items
    try:
        q_size = log_queue.qsize()
        for _ in range(min(limit, q_size)):
            item = log_queue.get()
            logs.append(item)
            log_queue.put(item)  # Put it back
    except:
        pass  # Ignore queue access issues
    return logs

def add_log_listener(callback):
    """Add a callback function that will be called for each new log entry"""
    log_listeners.add(callback)
    return callback

def remove_log_listener(callback):
    """Remove a log listener"""
    if callback in log_listeners:
        log_listeners.remove(callback)
PY


# ------------------------------------------------------------
# 9.  agent.py  —  now powered by Ollama with Web UI
# ------------------------------------------------------------
cat > agent.py <<'PY'
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
PY
chmod +x agent.py


# ------------------------------------------------------------
# 10. Create UI templates
# ------------------------------------------------------------
log "Creating UI templates..."


# Create CSS file
mkdir -p "$WORKDIR/ui/static"
cat > "$WORKDIR/ui/static/styles.css" <<'CSS'
:root {
  --primary-color: #0066cc;
  --secondary-color: #6c757d;
  --success-color: #28a745;
  --danger-color: #dc3545;
  --warning-color: #ffc107;
  --info-color: #17a2b8;
  --dark-color: #343a40;
  --light-color: #f8f9fa;
  --body-bg: #f4f7fa;
  --card-bg: #ffffff;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  line-height: 1.6;
  color: #333;
  background: var(--body-bg);
  padding: 20px;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 10px;
  border-bottom: 1px solid #eee;
}

.logo {
  font-size: 24px;
  font-weight: bold;
  color: var(--primary-color);
  display: flex;
  align-items: center;
}

.logo span {
  margin-left: 10px;
}

nav ul {
  display: flex;
  list-style: none;
}

nav ul li {
  margin-left: 20px;
}

nav ul li a {
  text-decoration: none;
  color: var(--dark-color);
  font-weight: 500;
  transition: color 0.3s;
}

nav ul li a:hover {
  color: var(--primary-color);
}

.card {
  background: var(--card-bg);
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 20px;
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #eee;
}

.card-title {
  font-size: 18px;
  font-weight: 600;
}

.button {
  display: inline-block;
  background: var(--primary-color);
  color: white;
  padding: 8px 16px;
  border-radius: 4px;
  text-decoration: none;
  font-weight: 500;
  border: none;
  cursor: pointer;
  transition: background 0.3s;
}

.button:hover {
  background: #0056b3;
}

.button-secondary {
  background: var(--secondary-color);
}

.button-success {
  background: var(--success-color);
}

.button-danger {
  background: var(--danger-color);
}

.form-group {
  margin-bottom: 15px;
}

label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
}

input[type="text"],
textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 16px;
  font-family: inherit;
}

textarea {
  min-height: 120px;
  resize: vertical;
}

/* Task status indicators */
.status {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
}

.status-running {
  background: var(--info-color);
  color: white;
}

.status-completed {
  background: var(--success-color);
  color: white;
}

.status-failed {
  background: var(--danger-color);
  color: white;
}

.status-starting {
  background: var(--warning-color);
  color: black;
}

/* Log console styling */
.console {
  background: #1e1e1e;
  color: #f0f0f0;
  padding: 15px;
  border-radius: 6px;
  font-family: 'Courier New', monospace;
  height: 500px;
  overflow-y: auto;
  margin-bottom: 20px;
  line-height: 1.4;
}

.console-line {
  margin-bottom: 4px;
}

.output-code {
  background: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  margin: 10px 0;
  font-family: 'Courier New', monospace;
  white-space: pre-wrap;
  overflow-x: auto;
  border-left: 3px solid var(--primary-color);
}

.task-list {
  margin-top: 20px;
}

.task-item {
  padding: 15px;
  margin-bottom: 10px;
  border-radius: 6px;
  background: white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s;
}

.task-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.task-goal {
  font-weight: 600;
  font-size: 16px;
}

.task-meta {
  font-size: 14px;
  color: #666;
  margin-top: 10px;
}

/* Status Dashboard */
.status-dashboard {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.status-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  text-align: center;
}

.status-card h3 {
  font-size: 16px;
  margin-bottom: 10px;
}

.status-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--primary-color);
}

.status-online {
  color: var(--success-color);
}

.status-offline {
  color: var(--danger-color);
}

/* Loading spinner */
.loader {
  border: 4px solid rgba(0, 0, 0, 0.1);
  border-left-color: var(--primary-color);
  border-radius: 50%;
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
  margin: 20px auto;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  header {
    flex-direction: column;
    align-items: flex-start;
  }
  
  nav ul {
    margin-top: 10px;
  }
  
  nav ul li {
    margin-left: 0;
    margin-right: 15px;
  }
  
  .status-dashboard {
    grid-template-columns: 1fr;
  }
}
CSS


# Create JavaScript file for UI functionality
cat > "$WORKDIR/ui/static/app.js" <<'JS'
// WebSocket connection for real-time updates
let ws;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

function connectWebSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;
  
  ws = new WebSocket(wsUrl);
  
  ws.onopen = function() {
    console.log('WebSocket connected');
    reconnectAttempts = 0;
    const statusIndicator = document.getElementById('ws-status');
    if (statusIndicator) {
      statusIndicator.className = 'status-online';
      statusIndicator.textContent = 'Connected';
    }
  };
  
  ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    
    if (data.type === 'log') {
      appendLog(data);
    } else if (data.type === 'task_update') {
      updateTaskStatus(data);
    } else if (data.type === 'task_complete') {
      completeTask(data);
    }
  };
  
  ws.onclose = function() {
    const statusIndicator = document.getElementById('ws-status');
    if (statusIndicator) {
      statusIndicator.className = 'status-offline';
      statusIndicator.textContent = 'Disconnected';
    }
    
    // Attempt to reconnect
    reconnectAttempts++;
    if (reconnectAttempts <= maxReconnectAttempts) {
      console.log(`WebSocket closed. Reconnecting (${reconnectAttempts}/${maxReconnectAttempts})...`);
      setTimeout(connectWebSocket, 3000);
    } else {
      console.log('WebSocket connection failed after multiple attempts.');
    }
  };
  
  ws.onerror = function(error) {
    console.error('WebSocket error:', error);
  };
}

// Log streaming with Server-Sent Events
function setupLogStream() {
  const logConsole = document.getElementById('log-console');
  if (!logConsole) return;
  
  const evtSource = new EventSource('/logs/stream');
  
  evtSource.onopen = function() {
    console.log('Log stream connected');
  };
  
  evtSource.addEventListener('logs', function(event) {
    const logs = JSON.parse(event.data);
    logConsole.innerHTML = '';
    
    logs.forEach(log => {
      const logLine = document.createElement('div');
      logLine.className = 'console-line';
      logLine.textContent = `[${log.timestamp}] ${log.message}`;
      logConsole.appendChild(logLine);
    });
    
    logConsole.scrollTop = logConsole.scrollHeight;
  });
  
  evtSource.addEventListener('log', function(event) {
    const log = JSON.parse(event.data);
    
    const logLine = document.createElement('div');
    logLine.className = 'console-line';
    logLine.textContent = `[${log.timestamp}] ${log.message}`;
    logConsole.appendChild(logLine);
    
    logConsole.scrollTop = logConsole.scrollHeight;
  });
  
  evtSource.onerror = function() {
    console.error('Log stream error. Reconnecting...');
  };
}

// Fetch and display active tasks
function loadActiveTasks() {
  const tasksList = document.getElementById('active-tasks');
  if (!tasksList) return;
  
  fetch('/api/tasks')
    .then(response => response.json())
    .then(tasks => {
      tasksList.innerHTML = '';
      
      if (Object.keys(tasks).length === 0) {
        tasksList.innerHTML = '<p>No active tasks.</p>';
        return;
      }
      
      for (const [id, task] of Object.entries(tasks)) {
        const taskItem = document.createElement('div');
        taskItem.className = 'task-item';
        taskItem.innerHTML = `
          <div class="task-header">
            <div class="task-goal">${task.goal}</div>
            <span class="status status-${task.status.toLowerCase().split(' ')[0]}">${task.status}</span>
          </div>
          <div class="task-meta">
            <div>Created: ${new Date(task.created).toLocaleString()}</div>
            <a href="/task/${id}" class="button">View Details</a>
          </div>
        `;
        tasksList.appendChild(taskItem);
      }
    })
    .catch(error => {
      console.error('Error loading tasks:', error);
      tasksList.innerHTML = '<p>Error loading tasks. Please try again.</p>';
    });
}

// Load task history
function loadTaskHistory() {
  const historyList = document.getElementById('task-history');
  if (!historyList) return;
  
  fetch('/api/history')
    .then(response => response.json())
    .then(tasks => {
      historyList.innerHTML = '';
      
      if (tasks.length === 0) {
        historyList.innerHTML = '<p>No task history found.</p>';
        return;
      }
      
      tasks.forEach(task => {
        const taskItem = document.createElement('div');
        taskItem.className = 'task-item';
        
        // Format duration nicely
        let duration = 'N/A';
        if (task.duration) {
          if (task.duration < 60) {
            duration = `${task.duration} seconds`;
          } else if (task.duration < 3600) {
            duration = `${Math.floor(task.duration / 60)} minutes`;
          } else {
            duration = `${Math.floor(task.duration / 3600)} hours, ${Math.floor((task.duration % 3600) / 60)} minutes`;
          }
        }
        
        taskItem.innerHTML = `
          <div class="task-header">
            <div class="task-goal">${task.goal}</div>
            <span class="status status-${task.status.toLowerCase()}">${task.status}</span>
          </div>
          <div class="task-meta">
            <div>Executed: ${new Date(task.timestamp).toLocaleString()}</div>
            <div>Duration: ${duration}</div>
            <button class="button" onclick="showOutput('${task.id}', \`${task.goal}\`, \`${task.output.replace(/`/g, '\\`')}\`)">Show Output</button>
          </div>
        `;
        historyList.appendChild(taskItem);
      });
    })
    .catch(error => {
      console.error('Error loading history:', error);
      historyList.innerHTML = '<p>Error loading task history. Please try again.</p>';
    });
}

// Task output modal
function showOutput(id, goal, output) {
  const modal = document.createElement('div');
  modal.style.position = 'fixed';
  modal.style.top = '0';
  modal.style.left = '0';
  modal.style.width = '100%';
  modal.style.height = '100%';
  modal.style.backgroundColor = 'rgba(0,0,0,0.7)';
  modal.style.zIndex = '1000';
  modal.style.display = 'flex';
  modal.style.justifyContent = 'center';
  modal.style.alignItems = 'center';
  
  const content = document.createElement('div');
  content.style.backgroundColor = 'white';
  content.style.padding = '20px';
  content.style.borderRadius = '8px';
  content.style.width = '80%';
  content.style.maxWidth = '800px';
  content.style.maxHeight = '80vh';
  content.style.overflow = 'auto';
  
  content.innerHTML = `
    <h2>${goal}</h2>
    <div class="output-code">${output.replace(/\n/g, '<br>')}</div>
    <button class="button button-secondary" onclick="document.body.removeChild(document.querySelector('[data-modal]'))">Close</button>
  `;
  
  modal.appendChild(content);
  modal.setAttribute('data-modal', '');
  
  // Close on click outside
  modal.addEventListener('click', function(e) {
    if (e.target === modal) {
      document.body.removeChild(modal);
    }
  });
  
  document.body.appendChild(modal);
}

// Task detail page functionality
function loadTaskDetails(taskId) {
  if (!taskId) return;
  
  const taskOutput = document.getElementById('task-output');
  const taskStatus = document.getElementById('task-status');
  const taskGoal = document.getElementById('task-goal');
  
  if (!taskOutput || !taskStatus || !taskGoal) return;
  
  function updateTask() {
    fetch(`/api/task/${taskId}`)
      .then(response => response.json())
      .then(task => {
        if (task.error) {
          taskOutput.innerHTML = `<p>Error: ${task.error}</p>`;
          return;
        }
        
        taskGoal.textContent = task.goal;
        taskStatus.textContent = task.status;
        taskStatus.className = `status status-${task.status.toLowerCase().split(' ')[0]}`;
        
        // Format the output with proper line breaks
        taskOutput.innerHTML = task.output.replace(/\n/g, '<br>');
        
        // If task is still running, schedule another update
        if (task.status.toLowerCase().includes('running')) {
          setTimeout(updateTask, 2000);
        }
      })
      .catch(error => {
        console.error('Error loading task details:', error);
        taskOutput.innerHTML = '<p>Error loading task details. Please try again.</p>';
      });
  }
  
  updateTask();
}

// Update system status indicators
function updateSystemStatus() {
  const statusContainer = document.getElementById('system-status');
  if (!statusContainer) return;
  
  fetch('/api/status')
    .then(response => response.json())
    .then(status => {
      // Ollama status
      const ollamaStatus = document.getElementById('ollama-status');
      if (ollamaStatus) {
        ollamaStatus.textContent = status.ollama === 'running' ? 'Online' : 'Offline';
        ollamaStatus.className = status.ollama === 'running' ? 'status-online' : 'status-offline';
      }
      
      // Current model
      const currentModel = document.getElementById('current-model');
      if (currentModel) {
        currentModel.textContent = status.current_model;
      }
      
      // Active tasks
      const activeTasks = document.getElementById('active-tasks-count');
      if (activeTasks) {
        activeTasks.textContent = status.active_tasks;
      }
      
      // Available models
      const availableModels = document.getElementById('available-models');
      if (availableModels) {
        availableModels.textContent = status.models.join(', ') || 'None';
      }
    })
    .catch(error => {
      console.error('Error updating system status:', error);
    });
}

// Initialize page functionality
document.addEventListener('DOMContentLoaded', function() {
  // Connect WebSocket
  connectWebSocket();
  
  // Setup log streaming
  setupLogStream();
  
  // Load active tasks
  loadActiveTasks();
  
  // Load task history
  loadTaskHistory();
  
  // Update system status
  updateSystemStatus();
  
  // Check if we're on a task detail page
  const taskId = document.getElementById('task-id')?.value;
  if (taskId) {
    loadTaskDetails(taskId);
  }
  
  // Refresh data periodically
  setInterval(function() {
    loadActiveTasks();
    updateSystemStatus();
  }, 10000);
});
JS


# history.html template
cat > "$WORKDIR/ui/templates/history.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Task History - Infinite AI Agent</title>
    <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 8V4m0 8v-4m0 8v-4m0 8v-4M4 6h16M4 10h16M4 14h16M4 18h16"></path>
                </svg>
                <span>Infinite AI Agent</span>
            </div>
            <nav>
                <ul>
                    <li><a href="/">Home</a></li>
                    <li><a href="/history" class="active">History</a></li>
                    <li><a href="/logs">Logs</a></li>
                </ul>
            </nav>
        </header>
        
        <div class="card">
            <div class="card-header">
                <h2 class="card-title">Task History</h2>
            </div>
            <div id="task-history" class="task-list">
                <div class="loader"></div>
            </div>
        </div>
    </div>
    
    <script src="/static/app.js"></script>
</body>
</html>
HTML


# index.html template
cat > "$WORKDIR/ui/templates/index.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Infinite AI Agent</title>
    <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 8V4m0 8v-4m0 8v-4m0 8v-4M4 6h16M4 10h16M4 14h16M4 18h16"></path>
                </svg>
                <span>Infinite AI Agent</span>
            </div>
            <nav>
                <ul>
                    <li><a href="/" class="active">Home</a></li>
                    <li><a href="/history">History</a></li>
                    <li><a href="/logs">Logs</a></li>
                </ul>
            </nav>
        </header>
        
        <div class="card">
            <div class="card-header">
                <h2 class="card-title">System Status</h2>
                <div>
                    WebSocket: <span id="ws-status" class="status-offline">Disconnected</span>
                </div>
            </div>
            <div class="status-dashboard" id="system-status">
                <div class="status-card">
                    <h3>Ollama Service</h3>
                    <div id="ollama-status" class="status-value">Checking...</div>
                </div>
                <div class="status-card">
                    <h3>Current Model</h3>
                    <div id="current-model" class="status-value">-</div>
                </div>
                <div class="status-card">
                    <h3>Active Tasks</h3>
                    <div id="active-tasks-count" class="status-value">0</div>
                </div>
                <div class="status-card">
                    <h3>Available Models</h3>
                    <div id="available-models" class="status-value">-</div>
                </div>
            </div>
        </div>
        
        <div class="card">
            <div class="card-header">
                <h2 class="card-title">Submit New Goal</h2>
            </div>
            <form action="/submit" method="post">
                <div class="form-group">
                    <label for="goal">What would you like the AI to do?</label>
                    <textarea id="goal" name="goal" placeholder="Enter your goal, task or question here..." required></textarea>
                </div>
                <div class="form-group">
                    <button type="submit" class="button">Submit Goal</button>
                </div>
            </form>
        </div>
        
        <div class="card">
            <div class="card-header">
                <h2 class="card-title">Active Tasks</h2>
            </div>
            <div id="active-tasks" class="task-list">
                <div class="loader"></div>
            </div>
        </div>
    </div>
    
    <script src="/static/app.js"></script>
</body>
</html>
HTML


# logs.html template
cat > "$WORKDIR/ui/templates/logs.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Logs - Infinite AI Agent</title>
    <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 8V4m0 8v-4m0 8v-4m0 8v-4M4 6h16M4 10h16M4 14h16M4 18h16"></path>
                </svg>
                <span>Infinite AI Agent</span>
            </div>
            <nav>
                <ul>
                    <li><a href="/">Home</a></li>
                    <li><a href="/history">History</a></li>
                    <li><a href="/logs" class="active">Logs</a></li>
                </ul>
            </nav>
        </header>
        
        <div class="card">
            <div class="card-header">
                <h2 class="card-title">Live Logs</h2>
                <div>
                    WebSocket: <span id="ws-status" class="status-offline">Disconnected</span>
                </div>
            </div>
            <div id="log-console" class="console">
                <div class="console-line">Connecting to log stream...</div>
            </div>
        </div>
    </div>
    
    <script src="/static/app.js"></script>
</body>
</html>
HTML


# task.html template
cat > "$WORKDIR/ui/templates/task.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Task Details - Infinite AI Agent</title>
    <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 8V4m0 8v-4m0 8v-4m0 8v-4M4 6h16M4 10h16M4 14h16M4 18h16"></path>
                </svg>
                <span>Infinite AI Agent</span>
            </div>
            <nav>
                <ul>
                    <li><a href="/">Home</a></li>
                    <li><a href="/history">History</a></li>
                    <li><a href="/logs">Logs</a></li>
                </ul>
            </nav>
        </header>
        
        <input type="hidden" id="task-id" value="{{ task_id }}">
        
        <div class="card">
            <div class="card-header">
                <div>
                    <h2 class="card-title">Task Details</h2>
                    <div id="task-goal" style="margin-top: 5px;">Loading...</div>
                </div>
                <div>
                    Status: <span id="task-status" class="status status-loading">Loading...</span>
                </div>
            </div>
            <div class="output-code" id="task-output">
                Loading task details...
            </div>
            <div style="margin-top: 15px;">
                <a href="/" class="button button-secondary">Back to Home</a>
            </div>
        </div>
    </div>
    
    <script src="/static/app.js"></script>
</body>
</html>
HTML


# ------------------------------------------------------------
# 11.  watchdog launcher with Web UI support
# ------------------------------------------------------------
cat > run.sh <<'SH'
#!/usr/bin/env bash
cd "$(dirname "$0")" || exit 1
source venv/bin/activate

# Function to check if Ollama is running
check_ollama() {
  curl -s --max-time 3 "http://127.0.0.1:11434/api/tags" >/dev/null 2>&1
}

# Function to check Windows Ollama
check_windows_ollama() {
  WIN_IP=$(ip route | grep default | awk '{print $3}' 2>/dev/null || echo "localhost")
  curl -s --max-time 3 "http://$WIN_IP:11434/api/tags" >/dev/null 2>&1
}

# Function to start Ollama
start_ollama() {
  echo "[watchdog] Starting Ollama service..."
  
  # Try local binary first
  if [[ -x "$PWD/bin/ollama" ]]; then
    echo "[watchdog] Using local Ollama binary"
    nohup "$PWD/bin/ollama" serve > logs/ollama_watchdog.log 2>&1 &
    OLLAMA_PID=$!
    echo "[watchdog] Started Ollama with PID $OLLAMA_PID"
  else
    # Try system binary
    if command -v ollama >/dev/null 2>&1; then
      echo "[watchdog] Using system Ollama binary"
      nohup ollama serve > logs/ollama_watchdog.log 2>&1 &
      OLLAMA_PID=$!
      echo "[watchdog] Started Ollama with PID $OLLAMA_PID"
    else
      echo "[watchdog] No Ollama binary found. Please install it manually."
    fi
  fi
  
  # Wait for Ollama to start
  for i in {1..10}; do
    if check_ollama; then
      echo "[watchdog] Ollama service is now available!"
      return 0
    fi
    echo "[watchdog] Waiting for Ollama service (attempt $i/10)..."
    sleep 2
  done
  
  # Check if Ollama is running on Windows
  if check_windows_ollama; then
    echo "[watchdog] Found Ollama running on Windows side."
    WIN_IP=$(ip route | grep default | awk '{print $3}' 2>/dev/null || echo "localhost")
    export OLLAMA_URL="http://$WIN_IP:11434"
    echo "[watchdog] Using Windows Ollama at $OLLAMA_URL"
    return 0
  fi
  
  echo "[watchdog] Warning: Could not start Ollama service."
  echo "[watchdog] Consider using the Windows helper script or start Ollama manually."
  return 1
}

# Get IP address for display
get_ip() {
  hostname -I | awk '{print $1}'
}

# Help ensure Ollama is running
OLLAMA_RUNNING=false
if check_ollama; then
  echo "[watchdog] Ollama service already running."
  OLLAMA_RUNNING=true
elif check_windows_ollama; then
  echo "[watchdog] Found Ollama running on Windows side."
  WIN_IP=$(ip route | grep default | awk '{print $3}' 2>/dev/null || echo "localhost")
  export OLLAMA_URL="http://$WIN_IP:11434"
  echo "[watchdog] Using Windows Ollama at $OLLAMA_URL"
  OLLAMA_RUNNING=true
else
  echo "[watchdog] Ollama service not running. Attempting to start..."
  if start_ollama; then
    OLLAMA_RUNNING=true
  else
    echo "[watchdog] Failed to start Ollama service."
    exit 1
  fi
fi

# Start the agent
echo "[watchdog] Starting Infinite AI Agent..."
nohup ./agent.py > logs/agent.log 2>&1 &
AGENT_PID=$!
echo "[watchdog] Infinite AI Agent started with PID $AGENT_PID"

# Monitor the agent process
while true; do
  if ! kill -0 "$AGENT_PID" 2>/dev/null; then
    echo "[watchdog] Infinite AI Agent process terminated. Restarting..."
    nohup ./agent.py > logs/agent.log 2>&1 &
    AGENT_PID=$!
    echo "[watchdog] Infinite AI Agent restarted with PID $AGENT_PID"
  fi
  sleep 10
done
SH
chmod +x run.sh


# ------------------------------------------------------------
# 12. Create handy shortcut scripts
# ------------------------------------------------------------
cat > start_agent.sh <<'SH'
#!/usr/bin/env bash
cd "$(dirname "$0")" || exit 1
./run.sh
SH
chmod +x start_agent.sh


cat > start_ollama.sh <<'SH'
#!/usr/bin/env bash
cd "$(dirname "$0")" || exit 1

# Check if Ollama is already running
if curl -s --max-time 3 "http://127.0.0.1:11434/api/tags" >/dev/null 2>&1; then
  echo "Ollama is already running."
  exit 0
fi

# Try local binary first
if [[ -x "$PWD/bin/ollama" ]]; then
  echo "Starting local Ollama binary..."
  exec "$PWD/bin/ollama" serve
else
  # Try system binary
  if command -v ollama >/dev/null 2>&1; then
    echo "Starting system Ollama binary..."
    exec ollama serve
  else
    echo "Error: No Ollama binary found."
    echo "Please install Ollama or use the Windows helper script."
    exit 1
  fi
fi
SH
chmod +x start_ollama.sh

log "Bootstrap completed ✓"
echo "
╔═════════════════════════════════════════════════════════════╗
║ 🚀 INFINITE AI BOOTSTRAP COMPLETED                          ║
╚═════════════════════════════════════════════════════════════╝

📋 Next steps:
  cd $WORKDIR
  ./start_agent.sh    # Runs the agent with Web UI and auto-restart

🌐 Access Web UI:
  http://localhost:8080
  http://$(hostname -I | awk '{print $1}'):8080

🤖 If Ollama isn't working in WSL:
  1. Use the Windows helper script:
     powershell.exe -ExecutionPolicy Bypass -File C:/repo/prime/windows_ollama_helper.ps1
  2. This will install and start Ollama on Windows
  3. WSL will connect to the Windows Ollama instance automatically

🛠️ Available Commands:
  ./start_ollama.sh   # Start just Ollama service
  ./run.sh            # Start agent with watchdog and Web UI
  
  # API Examples:
  curl -X POST http://localhost:8000/api/goal -d '{\"text\":\"<your goal>\"}' -H 'Content-Type: application/json'
  curl http://localhost:8000/api/status

💡 Options:
  bash prime.sh --clean  # Clean existing installation before setup
"


# ------------------------------------------------------------
# 12.  Start the agent
# ------------------------------------------------------------
log "Bootstrap completed ✓"
echo "
╔═════════════════════════════════════════════════════════════╗
║ 🚀 INFINITE AI BOOTSTRAP COMPLETED                          ║
╚═════════════════════════════════════════════════════════════╝

📋 Next steps:
  cd $WORKDIR
  ./start_agent.sh    # Runs the agent with Web UI and auto-restart

🌐 Access Web UI:
  http://localhost:8080
  http://$(hostname -I | awk '{print $1}'):8080

🤖 If Ollama isn't working in WSL:
  1. Use the Windows helper script:
     powershell.exe -ExecutionPolicy Bypass -File C:/repo/prime/windows_ollama_helper.ps1
  2. This will install and start Ollama on Windows
  3. WSL will connect to the Windows Ollama instance automatically

🛠️ Available Commands:
  ./start_ollama.sh   # Start just Ollama service
  ./run.sh            # Start agent with watchdog and Web UI
  
  # API Examples:
  curl -X POST http://localhost:8000/api/goal -d '{\"text\":\"<your goal>\"}' -H 'Content-Type: application/json'
  curl http://localhost:8000/api/status

💡 Options:
  bash prime.sh --clean  # Clean existing installation before setup
"
