#!/usr/bin/env bash
#
# prime.sh   —   Ollama-powered, gemma3 agent with UI
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
API_PORT = int(os.getenv("INFINITE_AI_PORT", 8000))
UI_PORT = int(os.getenv("INFINITE_AI_UI_PORT", 8080))
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
    <link rel="stylesheet" href="/static/css/bootstrap.min.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/bootstrap-icons.css">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
        }
        
        .navbar {
            background-color: #343a40 !important;
            box-shadow: 0 2px 4px rgba(0,0,0,.1);
        }
        
        .card {
            border: none;
            box-shadow: 0 0.125rem 0.25rem rgba(0,0,0,.075);
            margin-bottom: 1.5rem;
            border-radius: 0.5rem;
        }
        
        .card-header {
            background-color: #fff;
            border-bottom: 1px solid rgba(0,0,0,.05);
            border-radius: 0.5rem 0.5rem 0 0 !important;
            padding: 1rem 1.25rem;
        }
        
        .task-list {
            max-height: 600px;
            overflow-y: auto;
        }
        
        .task-item {
            border-left: 3px solid transparent;
            transition: all 0.2s ease;
        }
        
        .task-item:hover {
            background-color: rgba(0,0,0,.01);
        }
        
        .task-item-running { border-left-color: #007bff; }
        .task-item-completed { border-left-color: #28a745; }
        .task-item-failed { border-left-color: #dc3545; }
        .task-item-pending { border-left-color: #6c757d; }
        
        .status-badge {
            font-size: 0.8em;
            padding: 0.3em 0.6em;
            border-radius: 50rem;
        }
        
        .status-running { background-color: #007bff; color: white; }
        .status-completed { background-color: #28a745; color: white; }
        .status-failed { background-color: #dc3545; color: white; }
        .status-pending { background-color: #6c757d; color: white; }
        
        .system-status {
            display: flex;
            align-items: center;
            margin-bottom: 0.5rem;
        }
        
        .status-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 0.5rem;
        }
        
        .status-online { background-color: #28a745; }
        .status-offline { background-color: #dc3545; }
        
        .model-badge {
            background-color: #6610f2;
            color: white;
            font-size: 0.75em;
            margin-right: 0.25rem;
        }
        
        .goal-form {
            background-color: #fff;
            border-radius: 0.5rem;
            padding: 1.5rem;
            box-shadow: 0 0.125rem 0.25rem rgba(0,0,0,.075);
        }
        
        .stats-card {
            text-align: center;
            padding: 1rem;
        }
        
        .stats-icon {
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }
        
        .stats-number {
            font-size: 1.5rem;
            font-weight: bold;
        }
        
        .stats-label {
            font-size: 0.9rem;
            color: #6c757d;
        }
        
        #activeTasks .list-group-item {
            border-left-width: 3px;
            transition: all 0.2s;
        }
        
        #activeTasks .list-group-item:hover {
            background-color: #f8f9fa;
        }
        
        .task-time {
            font-size: 0.8em;
            color: #6c757d;
        }
        
        .task-controls {
            display: flex;
            gap: 0.5rem;
        }
        
        .btn-icon {
            padding: 0.25rem 0.5rem;
            display: inline-flex;
            align-items: center;
            justify-content: center;
        }
    </style>
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="container-fluid">
            <a class="navbar-brand" href="/">
                <i class="bi bi-robot"></i> Infinite AI Agent
            </a>
            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                <span class="navbar-toggler-icon"></span>
            </button>
            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav">
                    <li class="nav-item">
                        <a class="nav-link active" href="/"><i class="bi bi-house-fill"></i> Home</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/logs"><i class="bi bi-journal-text"></i> System Logs</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/history"><i class="bi bi-clock-history"></i> Task History</a>
                    </li>
                </ul>
            </div>
        </div>
    </nav>

    <div class="container-fluid py-4">
        <div class="row">
            <!-- Left Column -->
            <div class="col-lg-8">
                <div class="card">
                    <div class="card-header d-flex justify-content-between align-items-center">
                        <h5 class="mb-0"><i class="bi bi-list-check"></i> Active Tasks</h5>
                        <button id="refreshTasks" class="btn btn-sm btn-outline-secondary">
                            <i class="bi bi-arrow-clockwise"></i> Refresh
                        </button>
                    </div>
                    <div class="card-body p-0">
                        <div id="activeTasks" class="list-group list-group-flush task-list">
                            <!-- Tasks will be populated here -->
                            <div class="text-center py-5" id="noTasksMessage">
                                <i class="bi bi-inbox" style="font-size: 3rem; color: #dee2e6;"></i>
                                <p class="mt-3 text-muted">No active tasks</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <h5 class="mb-0"><i class="bi bi-bar-chart"></i> Task Statistics</h5>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <div class="col-md-3">
                                <div class="stats-card">
                                    <div class="stats-icon text-primary">
                                        <i class="bi bi-lightning-charge"></i>
                                    </div>
                                    <div class="stats-number" id="activeTasksCount">0</div>
                                    <div class="stats-label">Active Tasks</div>
                                </div>
                            </div>
                            <div class="col-md-3">
                                <div class="stats-card">
                                    <div class="stats-icon text-success">
                                        <i class="bi bi-check-circle"></i>
                                    </div>
                                    <div class="stats-number" id="completedTasksCount">0</div>
                                    <div class="stats-label">Completed</div>
                                </div>
                            </div>
                            <div class="col-md-3">
                                <div class="stats-card">
                                    <div class="stats-icon text-danger">
                                        <i class="bi bi-x-circle"></i>
                                    </div>
                                    <div class="stats-number" id="failedTasksCount">0</div>
                                    <div class="stats-label">Failed</div>
                                </div>
                            </div>
                            <div class="col-md-3">
                                <div class="stats-card">
                                    <div class="stats-icon text-info">
                                        <i class="bi bi-clock-history"></i>
                                    </div>
                                    <div class="stats-number" id="totalTasksCount">0</div>
                                    <div class="stats-label">Total Tasks</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <h5 class="mb-0"><i class="bi bi-clock-history"></i> Recent Tasks</h5>
                    </div>
                    <div class="card-body p-0">
                        <div id="recentTasks" class="list-group list-group-flush">
                            <!-- Recent tasks will be populated here -->
                            <div class="text-center py-5" id="noRecentTasksMessage">
                                <i class="bi bi-clock" style="font-size: 3rem; color: #dee2e6;"></i>
                                <p class="mt-3 text-muted">No recent tasks</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Right Column -->
            <div class="col-lg-4">
                <div class="card">
                    <div class="card-header">
                        <h5 class="mb-0"><i class="bi bi-send"></i> Start New Task</h5>
                    </div>
                    <div class="card-body">
                        <form action="/submit" method="post" class="goal-form">
                            <div class="mb-3">
                                <label for="goal" class="form-label">Enter your goal:</label>
                                <textarea class="form-control" id="goal" name="goal" rows="3" placeholder="e.g., Install Docker, Create a Python script to analyze logs, etc."></textarea>
                            </div>
                            <button type="submit" class="btn btn-primary w-100">
                                <i class="bi bi-play-fill"></i> Execute Task
                            </button>
                        </form>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <h5 class="mb-0"><i class="bi bi-info-circle"></i> System Status</h5>
                    </div>
                    <div class="card-body">
                        <div class="system-status">
                            <span class="status-indicator status-online" id="agentStatus"></span>
                            <span>Agent: </span>
                            <span class="ms-1" id="agentStatusText">Online</span>
                        </div>
                        <div class="system-status">
                            <span class="status-indicator" id="ollamaStatus"></span>
                            <span>Ollama: </span>
                            <span class="ms-1" id="ollamaStatusText">Checking...</span>
                        </div>
                        <hr>
                        <div class="mb-2">
                            <strong>Current Model:</strong> <span id="currentModel">Loading...</span>
                        </div>
                        <div>
                            <strong>Available Models:</strong>
                            <div id="availableModels" class="mt-1">
                                <div class="spinner-border spinner-border-sm text-primary" role="status">
                                    <span class="visually-hidden">Loading...</span>
                                </div>
                                <span class="ms-1">Loading models...</span>
                            </div>
                        </div>
                        <hr>
                        <div class="d-flex justify-content-between">
                            <span><strong>API Port:</strong> <span id="apiPort">8000</span></span>
                            <span><strong>UI Port:</strong> <span id="uiPort">8080</span></span>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <h5 class="mb-0"><i class="bi bi-life-preserver"></i> Quick Links</h5>
                    </div>
                    <div class="list-group list-group-flush">
                        <a href="/logs" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-journal-text me-2"></i> System Logs
                        </a>
                        <a href="/history" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-clock-history me-2"></i> Task History
                        </a>
                        <a href="https://docs.anthropic.com/claude/docs" target="_blank" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-book me-2"></i> Claude Documentation
                            <i class="bi bi-box-arrow-up-right ms-auto"></i>
                        </a>
                        <a href="https://github.com/ollama/ollama" target="_blank" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-github me-2"></i> Ollama GitHub
                            <i class="bi bi-box-arrow-up-right ms-auto"></i>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
    <script src="/static/js/bootstrap.bundle.min.js"></script>
    <script>
        $(document).ready(function() {
            // Function to format date
            function formatDate(dateString) {
                const date = new Date(dateString);
                return date.toLocaleString();
            }
            
            // Function to format duration
            function formatDuration(seconds) {
                if (!seconds) return 'N/A';
                
                const minutes = Math.floor(seconds / 60);
                const remainingSeconds = seconds % 60;
                
                if (minutes === 0) {
                    return `${remainingSeconds}s`;
                } else {
                    return `${minutes}m ${remainingSeconds}s`;
                }
            }
            
            // Get status badge class
            function getStatusBadgeClass(status) {
                if (status.includes('running') || status.includes('starting')) {
                    return 'status-running';
                } else if (status.includes('completed')) {
                    return 'status-completed';
                } else if (status.includes('failed') || status.includes('error')) {
                    return 'status-failed';
                } else {
                    return 'status-pending';
                }
            }
            
            // Get task item class
            function getTaskItemClass(status) {
                if (status.includes('running') || status.includes('starting')) {
                    return 'task-item-running';
                } else if (status.includes('completed')) {
                    return 'task-item-completed';
                } else if (status.includes('failed') || status.includes('error')) {
                    return 'task-item-failed';
                } else {
                    return 'task-item-pending';
                }
            }
            
            // Function to load active tasks
            function loadActiveTasks() {
                $.ajax({
                    url: '/api/tasks',
                    method: 'GET',
                    success: function(data) {
                        const tasksContainer = $('#activeTasks');
                        const noTasksMessage = $('#noTasksMessage');
                        
                        // Clear previous tasks
                        tasksContainer.find('.task-item').remove();
                        
                        // Update active tasks count
                        $('#activeTasksCount').text(Object.keys(data).length);
                        
                        if (Object.keys(data).length === 0) {
                            noTasksMessage.show();
                        } else {
                            noTasksMessage.hide();
                            
                            // Sort tasks by start time (newest first)
                            const sortedTasks = Object.values(data).sort((a, b) => {
                                return new Date(b.start_time || 0) - new Date(a.start_time || 0);
                            });
                            
                            // Add tasks to the list
                            sortedTasks.forEach(task => {
                                const taskId = task.id || task.task_id;
                                const goal = task.goal;
                                const status = task.status;
                                const step = task.step || 0;
                                const startTime = task.start_time ? new Date(task.start_time * 1000) : new Date();
                                
                                const taskHtml = `
                                    <div class="list-group-item task-item ${getTaskItemClass(status)}">
                                        <div class="d-flex justify-content-between align-items-start">
                                            <div>
                                                <h6 class="mb-1">
                                                    <a href="/task/${taskId}" class="text-decoration-none">
                                                        ${goal}
                                                    </a>
                                                </h6>
                                                <div class="d-flex align-items-center">
                                                    <span class="badge ${getStatusBadgeClass(status)} status-badge me-2">${status}</span>
                                                    <small class="text-muted">ID: ${taskId} | Step: ${step}</small>
                                                </div>
                                            </div>
                                            <div class="task-controls">
                                                <a href="/task/${taskId}" class="btn btn-sm btn-outline-primary btn-icon" title="View Task">
                                                    <i class="bi bi-eye"></i>
                                                </a>
                                                <a href="/task_logs/${taskId}" class="btn btn-sm btn-outline-secondary btn-icon" title="View Logs">
                                                    <i class="bi bi-file-text"></i>
                                                </a>
                                                <button data-task-id="${taskId}" class="btn btn-sm btn-outline-danger btn-icon cancel-task-btn" title="Cancel Task" ${status.includes('completed') || status.includes('failed') ? 'disabled' : ''}>
                                                    <i class="bi bi-x-circle"></i>
                                                </button>
                                            </div>
                                        </div>
                                        <div class="task-time mt-1">
                                            <small class="text-muted">Started: ${startTime.toLocaleString()}</small>
                                        </div>
                                    </div>
                                `;
                                
                                tasksContainer.append(taskHtml);
                            });
                        }
                    },
                    error: function(xhr, status, error) {
                        console.error('Error loading active tasks:', error);
                    }
                });
            }
            
            // Function to load recent tasks
            function loadRecentTasks() {
                $.ajax({
                    url: '/api/history?limit=5',
                    method: 'GET',
                    success: function(data) {
                        const tasksContainer = $('#recentTasks');
                        const noRecentTasksMessage = $('#noRecentTasksMessage');
                        
                        // Clear previous tasks
                        tasksContainer.find('.task-item').remove();
                        
                        if (data.length === 0) {
                            noRecentTasksMessage.show();
                        } else {
                            noRecentTasksMessage.hide();
                            
                            // Update task counts
                            let completedCount = 0;
                            let failedCount = 0;
                            
                            // Add tasks to the list
                            data.forEach(task => {
                                const taskId = task.id;
                                const goal = task.goal;
                                const status = task.status;
                                const timestamp = formatDate(task.timestamp);
                                const duration = formatDuration(task.duration);
                                
                                // Update counts
                                if (status === 'completed') {
                                    completedCount++;
                                } else if (status === 'failed') {
                                    failedCount++;
                                }
                                
                                const taskHtml = `
                                    <div class="list-group-item task-item ${getTaskItemClass(status)}">
                                        <div class="d-flex justify-content-between align-items-start">
                                            <div>
                                                <h6 class="mb-1">
                                                    <a href="/task/${taskId}" class="text-decoration-none">
                                                        ${goal}
                                                    </a>
                                                </h6>
                                                <div class="d-flex align-items-center">
                                                    <span class="badge ${getStatusBadgeClass(status)} status-badge me-2">${status}</span>
                                                    <small class="text-muted">ID: ${taskId} | Duration: ${duration}</small>
                                                </div>
                                            </div>
                                            <div class="task-controls">
                                                <a href="/task/${taskId}" class="btn btn-sm btn-outline-primary btn-icon" title="View Task">
                                                    <i class="bi bi-eye"></i>
                                                </a>
                                                <a href="/task_logs/${taskId}" class="btn btn-sm btn-outline-secondary btn-icon" title="View Logs">
                                                    <i class="bi bi-file-text"></i>
                                                </a>
                                            </div>
                                        </div>
                                        <div class="task-time mt-1">
                                            <small class="text-muted">Completed: ${timestamp}</small>
                                        </div>
                                    </div>
                                `;
                                
                                tasksContainer.append(taskHtml);
                            });
                            
                            // Update statistics
                            $('#completedTasksCount').text(completedCount);
                            $('#failedTasksCount').text(failedCount);
                            $('#totalTasksCount').text(data.length);
                        }
                    },
                    error: function(xhr, status, error) {
                        console.error('Error loading recent tasks:', error);
                    }
                });
            }
            
            // Function to load system status
            function loadSystemStatus() {
                $.ajax({
                    url: '/api/status',
                    method: 'GET',
                    success: function(data) {
                        // Update Ollama status
                        if (data.ollama === 'running') {
                            $('#ollamaStatus').removeClass('status-offline').addClass('status-online');
                            $('#ollamaStatusText').text('Online');
                        } else {
                            $('#ollamaStatus').removeClass('status-online').addClass('status-offline');
                            $('#ollamaStatusText').text('Offline');
                        }
                        
                        // Update current model
                        $('#currentModel').text(data.current_model);
                        
                        // Update available models
                        if (data.models && data.models.length > 0) {
                            const modelsContainer = $('#availableModels');
                            modelsContainer.empty();
                            
                            data.models.forEach(model => {
                                modelsContainer.append(`
                                    <span class="badge model-badge">${model}</span>
                                `);
                            });
                        } else {
                            $('#availableModels').html('<span class="text-muted">No models available</span>');
                        }
                        
                        // Update ports
                        $('#apiPort').text(data.api_port || 8000);
                        $('#uiPort').text(data.ui_port || 8080);
                    },
                    error: function(xhr, status, error) {
                        console.error('Error loading system status:', error);
                        
                        // Update status indicators to offline
                        $('#ollamaStatus').removeClass('status-online').addClass('status-offline');
                        $('#ollamaStatusText').text('Error connecting');
                    }
                });
            }
            
            // Cancel task
            $(document).on('click', '.cancel-task-btn', function() {
                const taskId = $(this).data('task-id');
                
                if (confirm(`Are you sure you want to cancel task ${taskId}?`)) {
                    $.ajax({
                        url: `/api/task/${taskId}/cancel`,
                        method: 'POST',
                        success: function(data) {
                            if (data.success) {
                                alert('Task cancelled successfully');
                                loadActiveTasks();
                            } else {
                                alert(`Failed to cancel task: ${data.error}`);
                            }
                        },
                        error: function(xhr, status, error) {
                            alert(`Error: ${error}`);
                        }
                    });
                }
            });
            
            // Refresh button
            $('#refreshTasks').click(function() {
                loadActiveTasks();
                loadRecentTasks();
                loadSystemStatus();
            });
            
            // Load data on page load
            loadActiveTasks();
            loadRecentTasks();
            loadSystemStatus();
            
            // Set up auto-refresh
            setInterval(function() {
                loadActiveTasks();
                loadSystemStatus();
            }, 10000);  // Refresh every 10 seconds
        });
    </script>
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
    <title>Infinite AI - System Logs</title>
    <link rel="stylesheet" href="/static/css/bootstrap.min.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/bootstrap-icons.css">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
        }
        
        .navbar {
            background-color: #343a40 !important;
            box-shadow: 0 2px 4px rgba(0,0,0,.1);
        }
        
        .log-container {
            background-color: #212529;
            color: #f8f9fa;
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
            font-size: 0.9rem;
            border-radius: 0.5rem;
            padding: 1rem;
            height: calc(100vh - 180px);
            overflow-y: auto;
            position: relative;
        }
        
        .log-entry {
            margin-bottom: 0.25rem;
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            word-wrap: break-word;
            border-left: 3px solid transparent;
        }
        
        .log-entry:hover {
            background-color: rgba(255, 255, 255, 0.05);
        }
        
        .log-timestamp {
            color: #6c757d;
            margin-right: 0.5rem;
        }
        
        .log-info { border-left-color: #0dcaf0; }
        .log-warning { border-left-color: #ffc107; }
        .log-error { border-left-color: #dc3545; }
        .log-success { border-left-color: #198754; }
        
        .log-toolbar {
            position: sticky;
            top: 0;
            background-color: #212529;
            padding: 0.5rem;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            z-index: 100;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .log-search {
            position: relative;
        }
        
        .log-search .form-control {
            background-color: #2c3034;
            border-color: #495057;
            color: #fff;
            padding-left: 2rem;
        }
        
        .log-search .search-icon {
            position: absolute;
            left: 0.5rem;
            top: 50%;
            transform: translateY(-50%);
            color: #6c757d;
        }
        
        .search-clear {
            position: absolute;
            right: 0.5rem;
            top: 50%;
            transform: translateY(-50%);
            color: #6c757d;
            cursor: pointer;
            display: none;
        }
        
        .filter-badge {
            cursor: pointer;
            user-select: none;
            transition: all 0.2s;
        }
        
        .filter-badge.active {
            background-color: #0d6efd !important;
            color: white !important;
        }
        
        .auto-scroll-toggle {
            cursor: pointer;
            user-select: none;
        }
        
        .btn-icon {
            padding: 0.25rem 0.5rem;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            margin-left: 0.25rem;
        }
        
        .status-card {
            background-color: #fff;
            border-radius: 0.5rem;
            padding: 1rem;
            margin-bottom: 1rem;
            box-shadow: 0 0.125rem 0.25rem rgba(0,0,0,.075);
        }
        
        .status-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 0.5rem;
        }
        
        .status-online { background-color: #28a745; }
        .status-offline { background-color: #dc3545; }
        
        .model-badge {
            background-color: #6610f2;
            color: white;
            font-size: 0.75em;
            margin-right: 0.25rem;
        }
        
        #noLogsMessage {
            text-align: center;
            padding: 2rem;
            color: #6c757d;
        }
        
        .log-actions {
            display: flex;
            gap: 0.5rem;
        }
        
        .log-stats {
            margin-bottom: 1rem;
        }
        
        .log-count {
            font-size: 1.2rem;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="container-fluid">
            <a class="navbar-brand" href="/">
                <i class="bi bi-robot"></i> Infinite AI Agent
            </a>
            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                <span class="navbar-toggler-icon"></span>
            </button>
            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav">
                    <li class="nav-item">
                        <a class="nav-link" href="/"><i class="bi bi-house-fill"></i> Home</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link active" href="/logs"><i class="bi bi-journal-text"></i> System Logs</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/history"><i class="bi bi-clock-history"></i> Task History</a>
                    </li>
                </ul>
            </div>
        </div>
    </nav>

    <div class="container-fluid py-4">
        <div class="row">
            <!-- Main Content -->
            <div class="col-lg-9">
                <div class="d-flex justify-content-between align-items-center mb-3">
                    <h2>System Logs</h2>
                    <div class="log-actions">
                        <button id="downloadLogsBtn" class="btn btn-outline-secondary">
                            <i class="bi bi-download"></i> Download Logs
                        </button>
                        <button id="clearLogsBtn" class="btn btn-outline-danger">
                            <i class="bi bi-trash"></i> Clear View
                        </button>
                    </div>
                </div>
                
                <div class="log-stats d-flex gap-3">
                    <div class="bg-light p-2 rounded">
                        <small class="text-muted">Total Logs</small>
                        <div class="log-count" id="totalLogCount">0</div>
                    </div>
                    <div class="bg-light p-2 rounded">
                        <small class="text-muted">Info</small>
                        <div class="log-count text-info" id="infoLogCount">0</div>
                    </div>
                    <div class="bg-light p-2 rounded">
                        <small class="text-muted">Warning</small>
                        <div class="log-count text-warning" id="warningLogCount">0</div>
                    </div>
                    <div class="bg-light p-2 rounded">
                        <small class="text-muted">Error</small>
                        <div class="log-count text-danger" id="errorLogCount">0</div>
                    </div>
                </div>
                
                <div class="log-container">
                    <div class="log-toolbar">
                        <div class="d-flex gap-2 align-items-center">
                            <div class="log-search">
                                <i class="bi bi-search search-icon"></i>
                                <input type="text" class="form-control form-control-sm" id="logSearch" placeholder="Search logs...">
                                <i class="bi bi-x-circle search-clear" id="clearSearch"></i>
                            </div>
                            
                            <div class="filter-badges d-flex gap-1">
                                <span class="badge bg-secondary filter-badge active" data-filter="all">All</span>
                                <span class="badge bg-info text-dark filter-badge" data-filter="info">Info</span>
                                <span class="badge bg-warning text-dark filter-badge" data-filter="warning">Warning</span>
                                <span class="badge bg-danger filter-badge" data-filter="error">Error</span>
                                <span class="badge bg-success filter-badge" data-filter="success">Success</span>
                            </div>
                        </div>
                        
                        <div class="d-flex align-items-center">
                            <div class="form-check form-switch me-2">
                                <input class="form-check-input" type="checkbox" id="autoScrollToggle" checked>
                                <label class="form-check-label text-light auto-scroll-toggle" for="autoScrollToggle">Auto-scroll</label>
                            </div>
                            
                            <button class="btn btn-sm btn-outline-light btn-icon" id="pauseLogsBtn" title="Pause Log Stream">
                                <i class="bi bi-pause-fill"></i>
                            </button>
                            
                            <button class="btn btn-sm btn-outline-light btn-icon" id="resumeLogsBtn" title="Resume Log Stream" style="display: none;">
                                <i class="bi bi-play-fill"></i>
                            </button>
                            
                            <button class="btn btn-sm btn-outline-light btn-icon" id="scrollToBottomBtn" title="Scroll to Bottom">
                                <i class="bi bi-arrow-down-square"></i>
                            </button>
                        </div>
                    </div>
                    
                    <div id="logEntries">
                        <div id="noLogsMessage">
                            <i class="bi bi-journal-text" style="font-size: 3rem;"></i>
                            <p class="mt-3">Waiting for logs...</p>
                        </div>
                    </div>
                </div>
            </div>
            
            <!-- Sidebar -->
            <div class="col-lg-3">
                <div class="status-card">
                    <h5 class="mb-3"><i class="bi bi-info-circle"></i> System Status</h5>
                    <div class="d-flex align-items-center mb-2">
                        <span class="status-indicator status-online" id="agentStatus"></span>
                        <span>Agent: </span>
                        <span class="ms-1" id="agentStatusText">Online</span>
                    </div>
                    <div class="d-flex align-items-center mb-2">
                        <span class="status-indicator" id="ollamaStatus"></span>
                        <span>Ollama: </span>
                        <span class="ms-1" id="ollamaStatusText">Checking...</span>
                    </div>
                    <hr>
                    <div class="mb-2">
                        <strong>Current Model:</strong> <span id="currentModel">Loading...</span>
                    </div>
                    <div>
                        <strong>Available Models:</strong>
                        <div id="availableModels" class="mt-1">
                            <div class="spinner-border spinner-border-sm text-primary" role="status">
                                <span class="visually-hidden">Loading...</span>
                            </div>
                            <span class="ms-1">Loading models...</span>
                        </div>
                    </div>
                </div>
                
                <div class="status-card">
                    <h5 class="mb-3"><i class="bi bi-gear"></i> Log Settings</h5>
                    <div class="mb-3">
                        <label for="logLimit" class="form-label">Max Logs to Display:</label>
                        <select class="form-select" id="logLimit">
                            <option value="100">100 entries</option>
                            <option value="200">200 entries</option>
                            <option value="500">500 entries</option>
                            <option value="1000">1000 entries</option>
                        </select>
                    </div>
                    <div class="mb-3">
                        <label for="refreshRate" class="form-label">Auto-Refresh Rate:</label>
                        <select class="form-select" id="refreshRate">
                            <option value="1000">1 second</option>
                            <option value="2000">2 seconds</option>
                            <option value="5000" selected>5 seconds</option>
                            <option value="10000">10 seconds</option>
                            <option value="0">Disabled</option>
                        </select>
                    </div>
                    <div class="form-check mb-3">
                        <input class="form-check-input" type="checkbox" id="showTimestamps" checked>
                        <label class="form-check-label" for="showTimestamps">
                            Show Timestamps
                        </label>
                    </div>
                    <div class="form-check mb-3">
                        <input class="form-check-input" type="checkbox" id="enableHighlighting" checked>
                        <label class="form-check-label" for="enableHighlighting">
                            Enable Log Highlighting
                        </label>
                    </div>
                </div>
                
                <div class="status-card">
                    <h5 class="mb-3"><i class="bi bi-link-45deg"></i> Quick Links</h5>
                    <div class="list-group list-group-flush">
                        <a href="/" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-house-fill me-2"></i> Dashboard
                        </a>
                        <a href="/history" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-clock-history me-2"></i> Task History
                        </a>
                        <a href="#" id="refreshSystemStatusBtn" class="list-group-item list-group-item-action d-flex align-items-center">
                            <i class="bi bi-arrow-clockwise me-2"></i> Refresh Status
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
    <script src="/static/js/bootstrap.bundle.min.js"></script>
    <script>
        $(document).ready(function() {
            // Variables
            let logEntries = [];
            let filteredEntries = [];
            let paused = false;
            let filterType = 'all';
            let searchTerm = '';
            let autoScroll = true;
            let eventSource = null;
            let logCounts = {
                total: 0,
                info: 0,
                warning: 0,
                error: 0,
                success: 0
            };
            
            // Elements
            const logEntriesContainer = $('#logEntries');
            const noLogsMessage = $('#noLogsMessage');
            const logSearch = $('#logSearch');
            const clearSearch = $('#clearSearch');
            const filterBadges = $('.filter-badge');
            const autoScrollToggle = $('#autoScrollToggle');
            const pauseLogsBtn = $('#pauseLogsBtn');
            const resumeLogsBtn = $('#resumeLogsBtn');
            const scrollToBottomBtn = $('#scrollToBottomBtn');
            const clearLogsBtn = $('#clearLogsBtn');
            const downloadLogsBtn = $('#downloadLogsBtn');
            const logLimit = $('#logLimit');
            const refreshRate = $('#refreshRate');
            const showTimestamps = $('#showTimestamps');
            const enableHighlighting = $('#enableHighlighting');
            
            // Initialize system status and log stream
            loadSystemStatus();
            initLogStream();
            
            // Function to determine log level class
            function getLogLevelClass(message) {
                const lowerMessage = message.toLowerCase();
                
                if (lowerMessage.includes('error') || lowerMessage.includes('fail') || lowerMessage.includes('exception')) {
                    return 'log-error';
                } else if (lowerMessage.includes('warn')) {
                    return 'log-warning';
                } else if (lowerMessage.includes('success') || lowerMessage.includes('completed') || lowerMessage.includes('done')) {
                    return 'log-success';
                } else {
                    return 'log-info';
                }
            }
            
            // Function to format timestamp
            function formatTimestamp(timestamp) {
                const date = new Date(timestamp);
                const hours = date.getHours().toString().padStart(2, '0');
                const minutes = date.getMinutes().toString().padStart(2, '0');
                const seconds = date.getSeconds().toString().padStart(2, '0');
                const ms = date.getMilliseconds().toString().padStart(3, '0');
                
                return `${hours}:${minutes}:${seconds}.${ms}`;
            }
            
            // Function to add log entry
            function addLogEntry(entry) {
                // Add to log entries array
                logEntries.push(entry);
                
                // Limit the number of entries
                const maxEntries = parseInt(logLimit.val());
                if (logEntries.length > maxEntries) {
                    logEntries = logEntries.slice(-maxEntries);
                }
                
                // Update log counts
                logCounts.total = logEntries.length;
                
                // Count by type
                logCounts.info = 0;
                logCounts.warning = 0;
                logCounts.error = 0;
                logCounts.success = 0;
                
                logEntries.forEach(entry => {
                    const logClass = getLogLevelClass(entry.message);
                    if (logClass === 'log-info') logCounts.info++;
                    else if (logClass === 'log-warning') logCounts.warning++;
                    else if (logClass === 'log-error') logCounts.error++;
                    else if (logClass === 'log-success') logCounts.success++;
                });
                
                // Update count display
                $('#totalLogCount').text(logCounts.total);
                $('#infoLogCount').text(logCounts.info);
                $('#warningLogCount').text(logCounts.warning);
                $('#errorLogCount').text(logCounts.error);
                
                // Filter and display logs
                filterLogs();
            }
            
            // Function to filter logs
            function filterLogs() {
                // Apply filters
                filteredEntries = logEntries.filter(entry => {
                    const logClass = getLogLevelClass(entry.message);
                    const matchesFilter = filterType === 'all' || 
                                        (filterType === 'info' && logClass === 'log-info') ||
                                        (filterType === 'warning' && logClass === 'log-warning') ||
                                        (filterType === 'error' && logClass === 'log-error') ||
                                        (filterType === 'success' && logClass === 'log-success');
                    
                    const matchesSearch = searchTerm === '' || 
                                        entry.message.toLowerCase().includes(searchTerm.toLowerCase());
                    
                    return matchesFilter && matchesSearch;
                });
                
                // Render filtered logs
                renderLogs();
            }
            
            // Function to render logs
            function renderLogs() {
                // Clear existing logs
                logEntriesContainer.empty();
                
                if (filteredEntries.length === 0) {
                    noLogsMessage.show();
                } else {
                    noLogsMessage.hide();
                    
                    // Add entries
                    filteredEntries.forEach(entry => {
                        const timestamp = showTimestamps.is(':checked') ? 
                                        `<span class="log-timestamp">[${formatTimestamp(entry.timestamp)}]</span>` : '';
                        
                        const logClass = enableHighlighting.is(':checked') ? 
                                        getLogLevelClass(entry.message) : '';
                        
                        const logHtml = `
                            <div class="log-entry ${logClass}">
                                ${timestamp}${entry.message}
                            </div>
                        `;
                        
                        logEntriesContainer.append(logHtml);
                    });
                    
                    // Scroll to bottom if auto-scroll is enabled
                    if (autoScroll) {
                        const container = $('.log-container');
                        container.scrollTop(container[0].scrollHeight);
                    }
                }
            }
            
            // Function to initialize log stream
            function initLogStream() {
                // Close existing event source if open
                if (eventSource) {
                    eventSource.close();
                }
                
                // Create new event source
                eventSource = new EventSource('/logs/stream');
                
                // Handle initial logs
                eventSource.addEventListener('logs', function(event) {
                    const logs = JSON.parse(event.data);
                    
                    logs.forEach(log => {
                        addLogEntry(log);
                    });
                });
                
                // Handle log events
                eventSource.addEventListener('log', function(event) {
                    if (!paused) {
                        const log = JSON.parse(event.data);
                        addLogEntry(log);
                    }
                });
                
                // Handle heartbeat events
                eventSource.addEventListener('heartbeat', function(event) {
                    // Heartbeat received, connection is alive
                });
                
                // Handle errors
                eventSource.onerror = function(error) {
                    console.error('EventSource error:', error);
                    
                    // Try to reconnect after a delay
                    setTimeout(function() {
                        initLogStream();
                    }, 5000);
                };
            }
            
            // Function to load system status
            function loadSystemStatus() {
                $.ajax({
                    url: '/api/status',
                    method: 'GET',
                    success: function(data) {
                        // Update Ollama status
                        if (data.ollama === 'running') {
                            $('#ollamaStatus').removeClass('status-offline').addClass('status-online');
                            $('#ollamaStatusText').text('Online');
                        } else {
                            $('#ollamaStatus').removeClass('status-online').addClass('status-offline');
                            $('#ollamaStatusText').text('Offline');
                        }
                        
                        // Update current model
                        $('#currentModel').text(data.current_model);
                        
                        // Update available models
                        if (data.models && data.models.length > 0) {
                            const modelsContainer = $('#availableModels');
                            modelsContainer.empty();
                            
                            data.models.forEach(model => {
                                modelsContainer.append(`
                                    <span class="badge model-badge">${model}</span>
                                `);
                            });
                        } else {
                            $('#availableModels').html('<span class="text-muted">No models available</span>');
                        }
                    },
                    error: function(xhr, status, error) {
                        console.error('Error loading system status:', error);
                        
                        // Update status indicators to offline
                        $('#ollamaStatus').removeClass('status-online').addClass('status-offline');
                        $('#ollamaStatusText').text('Error connecting');
                    }
                });
            }
            
            // Search functionality
            logSearch.on('input', function() {
                searchTerm = $(this).val();
                
                if (searchTerm === '') {
                    clearSearch.hide();
                } else {
                    clearSearch.show();
                }
                
                filterLogs();
            });
            
            // Clear search
            clearSearch.on('click', function() {
                logSearch.val('');
                searchTerm = '';
                clearSearch.hide();
                filterLogs();
            });
            
            // Filter badges
            filterBadges.on('click', function() {
                filterBadges.removeClass('active');
                $(this).addClass('active');
                
                filterType = $(this).data('filter');
                filterLogs();
            });
            
            // Auto-scroll toggle
            autoScrollToggle.on('change', function() {
                autoScroll = $(this).is(':checked');
                
                if (autoScroll) {
                    const container = $('.log-container');
                    container.scrollTop(container[0].scrollHeight);
                }
            });
            
            // Pause/Resume logs
            pauseLogsBtn.on('click', function() {
                paused = true;
                pauseLogsBtn.hide();
                resumeLogsBtn.show();
            });
            
            resumeLogsBtn.on('click', function() {
                paused = false;
                resumeLogsBtn.hide();
                pauseLogsBtn.show();
            });
            
            // Scroll to bottom
            scrollToBottomBtn.on('click', function() {
                const container = $('.log-container');
                container.scrollTop(container[0].scrollHeight);
            });
            
            // Clear logs
            clearLogsBtn.on('click', function() {
                if (confirm('Are you sure you want to clear all logs from the view? This will not delete logs from the server.')) {
                    logEntries = [];
                    filteredEntries = [];
                    logCounts = {
                        total: 0,
                        info: 0,
                        warning: 0,
                        error: 0,
                        success: 0
                    };
                    
                    // Update count display
                    $('#totalLogCount').text(0);
                    $('#infoLogCount').text(0);
                    $('#warningLogCount').text(0);
                    $('#errorLogCount').text(0);
                    
                    renderLogs();
                }
            });
            
            // Download logs
            downloadLogsBtn.on('click', function() {
                // Create log content
                let content = '';
                
                logEntries.forEach(entry => {
                    content += `[${formatTimestamp(entry.timestamp)}] ${entry.message}\n`;
                });
                
                // Create blob and download
                const blob = new Blob([content], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                const date = new Date().toISOString().replace(/:/g, '-').split('.')[0];
                
                a.href = url;
                a.download = `infinite_ai_logs_${date}.txt`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            });
            
            // Log limit change
            logLimit.on('change', function() {
                const maxEntries = parseInt($(this).val());
                
                // Limit the number of entries
                if (logEntries.length > maxEntries) {
                    logEntries = logEntries.slice(-maxEntries);
                    filterLogs();
                }
            });
            
            // Refresh rate change
            refreshRate.on('change', function() {
                const rate = parseInt($(this).val());
                
                // If disabled, pause logs
                if (rate === 0) {
                    paused = true;
                    pauseLogsBtn.hide();
                    resumeLogsBtn.show();
                } else {
                    paused = false;
                    resumeLogsBtn.hide();
                    pauseLogsBtn.show();
                }
            });
            
            // Show timestamps change
            showTimestamps.on('change', function() {
                renderLogs();
            });
            
            // Enable highlighting change
            enableHighlighting.on('change', function() {
                renderLogs();
            });
            
            // Refresh system status
            $('#refreshSystemStatusBtn').on('click', function(e) {
                e.preventDefault();
                loadSystemStatus();
            });
        });
    </script>
</body>
</html>
HTML


# task_logs.html template
cat > "$WORKDIR/ui/templates/task_logs.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Infinite AI - Task Details</title>
    <!-- Custom CSS without Bootstrap dependencies -->
    <style>
        /* Reset and base styles */
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
            line-height: 1.6;
            color: #333;
        }

        /* Navigation */
        .navbar {
            background-color: #343a40;
            color: white;
            padding: 1rem;
            box-shadow: 0 2px 4px rgba(0,0,0,.1);
        }

        .navbar-brand {
            font-size: 1.25rem;
            font-weight: bold;
            color: white;
            text-decoration: none;
        }

        .navbar-nav {
            display: flex;
            list-style: none;
            margin-top: 1rem;
        }

        .nav-item {
            margin-right: 1rem;
        }

        .nav-link {
            color: rgba(255,255,255,0.8);
            text-decoration: none;
            transition: color 0.3s;
        }

        .nav-link:hover {
            color: white;
        }

        /* Layout */
        .container {
            width: 100%;
            max-width: 1200px;
            margin: 0 auto;
            padding: 1rem;
        }

        .row {
            display: flex;
            flex-wrap: wrap;
            margin: 0 -0.5rem;
        }

        .col {
            flex: 1;
            padding: 0 0.5rem;
        }

        .col-3 {
            flex: 0 0 25%;
            max-width: 25%;
            padding: 0 0.5rem;
        }

        .col-9 {
            flex: 0 0 75%;
            max-width: 75%;
            padding: 0 0.5rem;
        }

        /* Buttons */
        .btn {
            display: inline-block;
            font-weight: 400;
            text-align: center;
            white-space: nowrap;
            vertical-align: middle;
            user-select: none;
            border: 1px solid transparent;
            padding: 0.375rem 0.75rem;
            font-size: 1rem;
            line-height: 1.5;
            border-radius: 0.25rem;
            transition: color 0.15s ease-in-out, background-color 0.15s ease-in-out, border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;
            cursor: pointer;
            text-decoration: none;
        }

        .btn-primary {
            color: #fff;
            background-color: #007bff;
            border-color: #007bff;
        }

        .btn-primary:hover {
            background-color: #0069d9;
            border-color: #0062cc;
        }

        .btn-secondary {
            color: #fff;
            background-color: #6c757d;
            border-color: #6c757d;
        }

        .btn-secondary:hover {
            background-color: #5a6268;
            border-color: #545b62;
        }

        .btn-danger {
            color: #fff;
            background-color: #dc3545;
            border-color: #dc3545;
        }

        .btn-danger:hover {
            background-color: #c82333;
            border-color: #bd2130;
        }

        .btn-outline-primary {
            color: #007bff;
            background-color: transparent;
            border-color: #007bff;
        }

        .btn-outline-primary:hover {
            color: #fff;
            background-color: #007bff;
        }

        .btn-outline-secondary {
            color: #6c757d;
            background-color: transparent;
            border-color: #6c757d;
        }

        .btn-outline-secondary:hover {
            color: #fff;
            background-color: #6c757d;
        }

        .btn-outline-danger {
            color: #dc3545;
            background-color: transparent;
            border-color: #dc3545;
        }

        .btn-outline-danger:hover {
            color: #fff;
            background-color: #dc3545;
        }

        /* Cards */
        .card {
            position: relative;
            display: flex;
            flex-direction: column;
            min-width: 0;
            word-wrap: break-word;
            background-color: #fff;
            background-clip: border-box;
            border: 1px solid rgba(0,0,0,.125);
            border-radius: 0.25rem;
            margin-bottom: 1rem;
        }

        .card-header {
            padding: 0.75rem 1.25rem;
            margin-bottom: 0;
            background-color: rgba(0,0,0,.03);
            border-bottom: 1px solid rgba(0,0,0,.125);
        }

        .card-body {
            flex: 1 1 auto;
            padding: 1.25rem;
        }

        /* Badges */
        .badge {
            display: inline-block;
            padding: 0.25em 0.4em;
            font-size: 75%;
            font-weight: 700;
            line-height: 1;
            text-align: center;
            white-space: nowrap;
            vertical-align: baseline;
            border-radius: 0.25rem;
        }

        /* Tabs */
        .nav-tabs {
            display: flex;
            flex-wrap: wrap;
            padding-left: 0;
            margin-bottom: 0;
            list-style: none;
            border-bottom: 1px solid #dee2e6;
        }

        .nav-tabs .nav-item {
            margin-bottom: -1px;
        }

        .nav-tabs .nav-link {
            border: 1px solid transparent;
            border-top-left-radius: 0.25rem;
            border-top-right-radius: 0.25rem;
            display: block;
            padding: 0.5rem 1rem;
        }

        .nav-tabs .nav-link.active {
            color: #495057;
            background-color: #fff;
            border-color: #dee2e6 #dee2e6 #fff;
        }

        .tab-content > .tab-pane {
            display: none;
        }

        .tab-content > .active {
            display: block;
        }

        /* Utilities */
        .text-center {
            text-align: center;
        }

        .mt-2 {
            margin-top: 0.5rem;
        }

        .mt-3 {
            margin-top: 1rem;
        }

        .mb-0 {
            margin-bottom: 0;
        }

        .mb-3 {
            margin-bottom: 1rem;
        }

        .py-3 {
            padding-top: 1rem;
            padding-bottom: 1rem;
        }

        .py-4 {
            padding-top: 1.5rem;
            padding-bottom: 1.5rem;
        }

        .py-5 {
            padding-top: 3rem;
            padding-bottom: 3rem;
        }

        .d-flex {
            display: flex;
        }

        .justify-content-between {
            justify-content: space-between;
        }

        .align-items-center {
            align-items: center;
        }

        .text-white {
            color: white;
        }

        .text-muted {
            color: #6c757d;
        }

        .visually-hidden {
            position: absolute;
            width: 1px;
            height: 1px;
            padding: 0;
            margin: -1px;
            overflow: hidden;
            clip: rect(0, 0, 0, 0);
            white-space: nowrap;
            border: 0;
        }

        /* Icons (simple replacements) */
        .icon {
            display: inline-block;
            width: 1em;
            height: 1em;
            vertical-align: -0.125em;
            fill: currentColor;
        }

        /* Spinner */
        .spinner {
            display: inline-block;
            width: 2rem;
            height: 2rem;
            border: 0.25em solid currentColor;
            border-right-color: transparent;
            border-radius: 50%;
            animation: spinner-border .75s linear infinite;
        }

        .spinner-sm {
            width: 1rem;
            height: 1rem;
            border-width: 0.2em;
        }

        @keyframes spinner-border {
            to { transform: rotate(360deg); }
        }
    </style>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
        }

        .navbar {
            background-color: #343a40 !important;
            box-shadow: 0 2px 4px rgba(0,0,0,.1);
        }

        .terminal {
            background-color: #212529;
            color: #cccccc;
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
            font-size: 0.9rem;
            padding: 1.5rem;
            border-radius: 0.5rem;
            overflow-y: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
            height: calc(100vh - 250px);
            position: relative;
        }

        .terminal-toolbar {
            position: sticky;
            top: 0;
            background-color: #212529;
            padding: 0.5rem;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            margin-bottom: 1rem;
            z-index: 100;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .status-badge {
            font-size: 0.85em;
            padding: 0.4em 0.7em;
            border-radius: 50rem;
        }

        .status-running { background-color: #007bff; color: white; }
        .status-completed { background-color: #28a745; color: white; }
        .status-failed { background-color: #dc3545; color: white; }
        .status-waiting { background-color: #ffc107; color: black; }

        .info-panel {
            background-color: white;
            border-radius: 0.5rem;
            padding: 1.25rem;
            margin-bottom: 1.25rem;
            box-shadow: 0 0.125rem 0.25rem rgba(0,0,0,.075);
        }

        .info-heading {
            font-size: 1.1rem;
            font-weight: 600;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 1px solid #e9ecef;
        }

        .info-list {
            list-style-type: none;
            padding-left: 0;
            margin-bottom: 0;
        }

        .info-list li {
            display: flex;
            justify-content: space-between;
            padding: 0.5rem 0;
            border-bottom: 1px solid #f8f9fa;
        }

        .info-list li:last-child {
            border-bottom: none;
        }

        .info-label {
            font-weight: 500;
            color: #495057;
        }

        .info-value {
            color: #212529;
            text-align: right;
            word-break: break-word;
        }

        .step-indicator {
            font-size: 0.85rem;
            margin-top: 0.25rem;
            color: #6c757d;
        }

        .env-list {
            max-height: 300px;
            overflow-y: auto;
        }

        .task-logs-btn {
            text-decoration: none !important;
        }

        .task-logs-btn i {
            margin-right: 0.5rem;
        }

        .terminal-output {
            line-height: 1.5;
        }

        .command-line {
            color: #50fa7b;
            font-weight: bold;
        }

        .error-output {
            color: #ff5555;
        }

        .success-output {
            color: #50fa7b;
        }

        .step-header {
            color: #bd93f9;
            font-weight: bold;
            margin-top: 1rem;
            margin-bottom: 0.5rem;
            border-bottom: 1px solid #44475a;
            padding-bottom: 0.25rem;
        }

        .output-header {
            color: #8be9fd;
            font-weight: bold;
            margin-top: 0.5rem;
            margin-bottom: 0.5rem;
        }

        .terminal-actions {
            position: absolute;
            top: 1rem;
            right: 1rem;
            display: flex;
            gap: 0.5rem;
            z-index: 10;
        }

        .terminal-actions button {
            background-color: rgba(255, 255, 255, 0.1);
            border: none;
            color: #ccc;
            border-radius: 0.25rem;
            padding: 0.25rem 0.5rem;
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.2s;
        }

        .terminal-actions button:hover {
            background-color: rgba(255, 255, 255, 0.2);
        }

        .spinner-container {
            display: inline-block;
            margin-left: 0.5rem;
        }

        #environmentDetails .command-available {
            color: #50fa7b;
        }

        #environmentDetails .command-unavailable {
            color: #ff5555;
        }

        .task-controls-row {
            margin-bottom: 1rem;
        }

        .terminal-mode-tabs {
            margin-bottom: 1rem;
        }

        .terminal-mode-tabs .nav-link {
            padding: 0.25rem 0.75rem;
            font-size: 0.85rem;
        }

        .terminal-mode-tabs .nav-link.active {
            font-weight: 600;
        }

        .blinking-cursor::after {
            content: '▋';
            color: #ccc;
            animation: blink 1s step-end infinite;
        }

        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0; }
        }
    </style>
</head>
<body>
    <nav class="navbar">
        <div class="container">
            <a class="navbar-brand" href="/">
                🤖 Infinite AI Agent
            </a>
            <ul class="navbar-nav">
                <li class="nav-item">
                    <a class="nav-link" href="/">🏠 Home</a>
                </li>
                <li class="nav-item">
                    <a class="nav-link" href="/logs">📋 System Logs</a>
                </li>
                <li class="nav-item">
                    <a class="nav-link" href="/history">⏱️ Task History</a>
                </li>
            </ul>
        </div>
    </nav>

    <div class="container py-4">
        <div class="row">
            <!-- Left Column - Task Info -->
            <div class="col-3">
                <div class="d-flex justify-content-between align-items-center mb-3">
                    <h2 class="mb-0">Task Details</h2>
                </div>

                <div class="task-controls-row">
                    <a href="/" class="btn btn-outline-secondary">
                        <i class="bi bi-arrow-left"></i> Back to Dashboard
                    </a>
                    <a href="/task_logs/{{ task_id }}" class="btn btn-outline-primary ms-2 task-logs-btn">
                        <i class="bi bi-file-text"></i> View Detailed Logs
                    </a>
                </div>

                <div class="info-panel">
                    <div class="info-heading d-flex justify-content-between align-items-center">
                        <span><i class="bi bi-info-circle"></i> Task Information</span>
                        <span class="badge status-badge" id="taskStatusBadge">Loading...</span>
                    </div>
                    <ul class="info-list">
                        <li>
                            <span class="info-label">ID:</span>
                            <span class="info-value" id="taskId">{{ task_id }}</span>
                        </li>
                        <li>
                            <span class="info-label">Goal:</span>
                            <span class="info-value" id="taskGoal">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Created:</span>
                            <span class="info-value" id="taskCreated">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Duration:</span>
                            <span class="info-value" id="taskDuration">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Current Step:</span>
                            <span class="info-value">
                                <span id="taskStep">Loading...</span>
                                <div class="spinner-container" id="stepSpinner" style="display: none;">
                                    <div class="spinner-border spinner-border-sm text-primary" role="status">
                                        <span class="visually-hidden">Loading...</span>
                                    </div>
                                </div>
                            </span>
                        </li>
                    </ul>
                </div>

                <div class="info-panel">
                    <div class="info-heading">
                        <i class="bi bi-hdd-rack"></i> Environment
                    </div>
                    <div id="environmentDetails" class="env-list">
                        <div class="text-center py-3">
                            <div class="spinner-border text-primary" role="status">
                                <span class="visually-hidden">Loading...</span>
                            </div>
                            <p class="mt-2 text-muted">Loading environment details...</p>
                        </div>
                    </div>
                </div>

                <div class="info-panel">
                    <div class="info-heading">
                        <i class="bi bi-tools"></i> Task Controls
                    </div>
                    <div class="d-grid gap-2">
                        <button class="btn btn-outline-primary" id="refreshBtn">
                            <i class="bi bi-arrow-clockwise"></i> Refresh Status
                        </button>
                        <button class="btn btn-outline-danger" id="cancelBtn" disabled>
                            <i class="bi bi-x-circle"></i> Cancel Task
                        </button>
                    </div>
                </div>
            </div>

            <!-- Right Column - Terminal Output -->
            <div class="col-9">
                <div class="card border-0 shadow-sm">
                    <div class="card-header bg-dark text-white d-flex justify-content-between align-items-center py-3">
                        <h5 class="mb-0">
                            💻 Task Execution
                            <span class="step-indicator" id="taskProgress">Step <span id="currentStep">0</span> of <span id="totalSteps">?</span></span>
                        </h5>
                        <div>
                            <div class="form-check form-switch d-inline-block me-2">
                                <input class="form-check-input" type="checkbox" id="autoScrollToggle" checked>
                                <label class="form-check-label text-white" for="autoScrollToggle">Auto-scroll</label>
                            </div>
                            <button class="btn btn-outline-light" id="scrollToBottomBtn">
                                ⬇️
                            </button>
                        </div>
                    </div>

                    <div class="card-body p-0">
                        <ul class="nav nav-tabs terminal-mode-tabs" id="outputModeTabs">
                            <li class="nav-item">
                                <a class="nav-link active" id="formatted-tab" data-bs-toggle="tab" href="#formatted">Formatted Output</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" id="raw-tab" data-bs-toggle="tab" href="#raw">Raw Output</a>
                            </li>
                        </ul>

                        <div class="tab-content">
                            <div class="tab-pane fade show active" id="formatted">
                                <div class="terminal">
                                    <div class="terminal-actions">
                                        <button id="copyBtn" title="Copy to clipboard">
                                            📋 Copy
                                        </button>
                                        <button id="downloadBtn" title="Download output">
                                            💾 Download
                                        </button>
                                        <button id="clearBtn" title="Clear terminal">
                                            🗑️ Clear
                                        </button>
                                    </div>
                                    <div class="terminal-output" id="terminalOutput">
                                        <div class="text-center py-5" id="loadingOutput">
                                            <div class="spinner" role="status">
                                                <span class="visually-hidden">Loading...</span>
                                            </div>
                                            <p class="mt-3 text-muted">Loading task output...</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="tab-pane fade" id="raw">
                                <div class="terminal">
                                    <div class="terminal-actions">
                                        <button id="rawCopyBtn" title="Copy raw output">
                                            📋 Copy
                                        </button>
                                        <button id="rawDownloadBtn" title="Download raw output">
                                            💾 Download
                                        </button>
                                    </div>
                                    <pre id="rawOutput" class="mb-0">Loading raw output...</pre>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
    <!-- Custom JavaScript to replace Bootstrap functionality -->
    <script>
        // Simple tab functionality
        function setupTabs() {
            const tabLinks = document.querySelectorAll('.nav-link');
            const tabPanes = document.querySelectorAll('.tab-pane');

            tabLinks.forEach(link => {
                link.addEventListener('click', function(e) {
                    e.preventDefault();

                    // Remove active class from all tabs
                    tabLinks.forEach(tab => tab.classList.remove('active'));
                    tabPanes.forEach(pane => pane.classList.remove('active', 'show'));

                    // Add active class to current tab
                    this.classList.add('active');

                    // Show corresponding tab content
                    const target = this.getAttribute('href').substring(1);
                    document.getElementById(target).classList.add('active', 'show');
                });
            });
        }

        // Initialize tabs when DOM is loaded
        document.addEventListener('DOMContentLoaded', function() {
            setupTabs();
        });
    </script>
    <script>
        $(document).ready(function() {
            // Elements
            const taskId = '{{ task_id }}';
            const terminalOutput = $('#terminalOutput');
            const rawOutput = $('#rawOutput');
            const loadingOutput = $('#loadingOutput');
            const taskStatusBadge = $('#taskStatusBadge');
            const environmentDetails = $('#environmentDetails');
            const copyBtn = $('#copyBtn');
            const downloadBtn = $('#downloadBtn');
            const clearBtn = $('#clearBtn');
            const rawCopyBtn = $('#rawCopyBtn');
            const rawDownloadBtn = $('#rawDownloadBtn');
            const cancelBtn = $('#cancelBtn');
            const refreshBtn = $('#refreshBtn');
            const autoScrollToggle = $('#autoScrollToggle');
            const scrollToBottomBtn = $('#scrollToBottomBtn');
            const stepSpinner = $('#stepSpinner');

            // Variables
            let autoScroll = true;
            let ws = null;
            let rawOutputText = '';
            let taskCompleted = false;

            // Format functions
            function formatDuration(seconds) {
                if (!seconds) return 'N/A';

                const minutes = Math.floor(seconds / 60);
                const remainingSeconds = seconds % 60;

                if (minutes === 0) {
                    return `${remainingSeconds}s`;
                } else {
                    return `${minutes}m ${remainingSeconds}s`;
                }
            }

            function formatDate(dateString) {
                if (!dateString) return 'N/A';
                return new Date(dateString).toLocaleString();
            }

            function getStatusBadgeClass(status) {
                if (status.includes('running') || status.includes('starting')) {
                    return 'status-running';
                } else if (status.includes('completed')) {
                    return 'status-completed';
                } else if (status.includes('failed') || status.includes('error')) {
                    return 'status-failed';
                } else if (status.includes('waiting')) {
                    return 'status-waiting';
                } else {
                    return 'bg-secondary';
                }
            }

            // Function to highlight terminal output
            function highlightOutput(text) {
                // Split the output by step headers
                const parts = text.split(/^---\s+Step\s+\d+\s+\([A-Z]+\)\s+---$/gm);
                const stepHeaders = text.match(/^---\s+Step\s+\d+\s+\([A-Z]+\)\s+---$/gm) || [];

                let formattedOutput = '';

                if (parts.length <= 1) {
                    // No step headers found, just return the text with error/success highlighting
                    return highlightErrorsAndSuccess(text);
                }

                // Process each step
                for (let i = 0; i < stepHeaders.length; i++) {
                    const header = stepHeaders[i];
                    const content = parts[i + 1] || '';

                    // Add the step header
                    formattedOutput += `<div class="step-header">${header}</div>`;

                    // Split the step content into code and output
                    const outputIndex = content.indexOf('--- Output ---');

                    if (outputIndex !== -1) {
                        const code = content.substring(0, outputIndex).trim();
                        const output = content.substring(outputIndex + 14).trim();

                        // Add the code section
                        formattedOutput += `<div class="command-line">${escapeHtml(code)}</div>`;

                        // Add the output header
                        formattedOutput += `<div class="output-header">--- Output ---</div>`;

                        // Add the highlighted output
                        formattedOutput += highlightErrorsAndSuccess(output);
                    } else {
                        // If we can't split it, just add the whole content
                        formattedOutput += highlightErrorsAndSuccess(content);
                    }
                }

                return formattedOutput;
            }

            // Function to highlight errors and success messages
            function highlightErrorsAndSuccess(text) {
                let html = escapeHtml(text);

                // Highlight error messages
                html = html.replace(/(?:^|\n)(error|exception|traceback|fail|\[erro?r?\]|❌)/gi,
                                   (match) => `<span class="error-output">${match}</span>`);

                // Highlight success messages
                html = html.replace(/(?:^|\n)(success|completed|done|✅)/gi,
                                   (match) => `<span class="success-output">${match}</span>`);

                // Highlight commands with $
                html = html.replace(/(?:^|\n)\$\s+(.*?)(?=\n|$)/g,
                                   (match, cmd) => `<span class="command-line">$ ${cmd}</span>`);

                return html;
            }

            // Helper function to escape HTML
            function escapeHtml(text) {
                return text
                    .replace(/&/g, '&amp;')
                    .replace(/</g, '&lt;')
                    .replace(/>/g, '&gt;')
                    .replace(/"/g, '&quot;')
                    .replace(/'/g, '&#039;');
            }

            // Function to add blinking cursor if task is still running
            function addBlinkingCursor(element, isRunning) {
                // Remove existing cursor
                element.find('.blinking-cursor').remove();

                // Add new cursor if task is running
                if (isRunning) {
                    element.append('<span class="blinking-cursor"></span>');
                }
            }

            // Function to load task details
            function loadTaskDetails() {
                $.ajax({
                    url: `/api/task/${taskId}`,
                    method: 'GET',
                    success: function(data) {
                        if (data.error) {
                            terminalOutput.html(`<div class="error-output">Error: ${data.error}</div>`);
                            rawOutput.text(`Error: ${data.error}`);
                            loadingOutput.hide();
                            return;
                        }

                        // Update task info
                        $('#taskGoal').text(data.goal || 'N/A');
                        $('#taskStep').text(data.step || '0');
                        $('#currentStep').text(data.step || '0');

                        // Update task status badge
                        const status = data.status || 'unknown';
                        taskStatusBadge.text(status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(status)}`);

                        // Check if task is completed
                        taskCompleted = status.includes('completed') || status.includes('failed');

                        // Update cancel button
                        cancelBtn.prop('disabled', taskCompleted);

                        // Update created time
                        if (data.created) {
                            $('#taskCreated').text(formatDate(data.created));
                        }

                        // Update duration
                        if (data.duration) {
                            $('#taskDuration').text(formatDuration(data.duration));
                        } else if (data.start_time) {
                            const duration = Math.floor(Date.now() / 1000 - data.start_time);
                            $('#taskDuration').text(formatDuration(duration) + ' (running)');
                        }

                        // Show step spinner if task is running
                        if (!taskCompleted && status.includes('running')) {
                            stepSpinner.show();
                        } else {
                            stepSpinner.hide();
                        }

                        // Update terminal output
                        updateTerminalOutput(data.output || '');

                        // Add blinking cursor if task is still running
                        addBlinkingCursor(terminalOutput, !taskCompleted);

                        // Update environment details
                        updateEnvironmentDetails(data.environment);
                    },
                    error: function(xhr, status, error) {
                        terminalOutput.html(`<div class="error-output">Error loading task: ${error}</div>`);
                        rawOutput.text(`Error loading task: ${error}`);
                        loadingOutput.hide();
                    }
                });
            }

            // Function to update terminal output
            function updateTerminalOutput(output) {
                if (!output) {
                    return;
                }

                // Store raw output
                rawOutputText = output;
                rawOutput.text(output);

                // Format and display terminal output
                const highlightedOutput = highlightOutput(output);
                terminalOutput.html(highlightedOutput);
                loadingOutput.hide();

                // Scroll to bottom if auto-scroll is enabled
                if (autoScroll) {
                    const terminal = $('.terminal');
                    terminal.scrollTop(terminal[0].scrollHeight);
                }
            }

            // Function to update environment details
            function updateEnvironmentDetails(env) {
                if (!env) {
                    environmentDetails.html('<div class="text-muted">No environment information available</div>');
                    return;
                }

                let html = '<ul class="list-group list-group-flush">';

                // Process user and root status
                html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                    <span>User</span>
                    <span>${env.user || 'unknown'} ${env.is_root ? '(root)' : ''}</span>
                </li>`;

                // OS info
                if (env.os_info) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>OS</span>
                        <span>${env.os_info}</span>
                    </li>`;
                }

                // Kernel
                if (env.kernel) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Kernel</span>
                        <span>${env.kernel}</span>
                    </li>`;
                }

                // Working directory
                if (env.working_dir) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Working Directory</span>
                        <span>${env.working_dir}</span>
                    </li>`;
                }

                // Docker status
                if (env.docker_status) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Docker</span>
                        <span>${env.docker_status}${env.docker_running ? ` (${env.docker_running})` : ''}</span>
                    </li>`;
                }

                // Available commands
                if (env.available_commands) {
                    html += `<li class="list-group-item">
                        <div>Available Commands</div>
                        <div class="mt-1">`;

                    if (typeof env.available_commands === 'string') {
                        // Handle string format
                        const commands = env.available_commands.split(/\s+/).filter(Boolean);
                        commands.forEach(cmd => {
                            html += `<span class="badge bg-secondary me-1">${cmd}</span>`;
                        });
                    } else if (typeof env.available_commands === 'object') {
                        // Handle object format
                        for (const [cmd, status] of Object.entries(env.available_commands)) {
                            const isAvailable = status === 'available';
                            html += `<span class="badge ${isAvailable ? 'bg-success' : 'bg-danger'} me-1">${cmd}</span>`;
                        }
                    }

                    html += `</div>
                    </li>`;
                }

                // Memory info
                if (env.memory) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Memory</span>
                        <span>${env.memory}</span>
                    </li>`;
                }

                // Free disk space
                if (env.free_disk_space) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Free Disk Space</span>
                        <span>${env.free_disk_space}</span>
                    </li>`;
                }

                html += '</ul>';
                environmentDetails.html(html);
            }

            // Initialize WebSocket connection
            function initWebSocket() {
                ws = new WebSocket(`ws://${window.location.host}/ws`);

                ws.onopen = function() {
                    console.log('WebSocket connected');
                };

                ws.onmessage = function(event) {
                    const data = JSON.parse(event.data);

                    if (data.type === 'task_update' && data.id === taskId) {
                        // Update terminal output
                        updateTerminalOutput(data.output);

                        // Update task status and step
                        taskStatusBadge.text(data.status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(data.status)}`);

                        if (data.step) {
                            $('#taskStep').text(data.step);
                            $('#currentStep').text(data.step);
                            stepSpinner.show();
                        }
                    } else if (data.type === 'task_complete' && data.id === taskId) {
                        // Update terminal output
                        updateTerminalOutput(data.output);

                        // Update task status
                        taskStatusBadge.text(data.status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(data.status)}`);

                        // Update task completion
                        taskCompleted = true;
                        cancelBtn.prop('disabled', true);
                        stepSpinner.hide();

                        // Remove blinking cursor
                        addBlinkingCursor(terminalOutput, false);

                        // Reload task details to get final information
                        loadTaskDetails();
                    }
                };

                ws.onclose = function() {
                    console.log('WebSocket disconnected');
                    // Try to reconnect after a delay if the task is not completed
                    if (!taskCompleted) {
                        setTimeout(initWebSocket, 5000);
                    }
                };

                ws.onerror = function(error) {
                    console.error('WebSocket error:', error);
                };
            }

            // Copy terminal output
            copyBtn.on('click', function() {
                const text = terminalOutput.text();
                navigator.clipboard.writeText(text)
                    .then(() => {
                        const originalText = copyBtn.html();
                        copyBtn.html('<i class="bi bi-check"></i> Copied!');
                        setTimeout(() => {
                            copyBtn.html(originalText);
                        }, 2000);
                    })
                    .catch(err => {
                        alert('Failed to copy: ' + err);
                    });
            });

            // Copy raw output
            rawCopyBtn.on('click', function() {
                navigator.clipboard.writeText(rawOutputText)
                    .then(() => {
                        const originalText = rawCopyBtn.html();
                        rawCopyBtn.html('<i class="bi bi-check"></i> Copied!');
                        setTimeout(() => {
                            rawCopyBtn.html(originalText);
                        }, 2000);
                    })
                    .catch(err => {
                        alert('Failed to copy: ' + err);
                    });
            });

            // Download terminal output
            downloadBtn.on('click', function() {
                const text = terminalOutput.text();
                const blob = new Blob([text], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `task_${taskId}_output.txt`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            });

            // Download raw output
            rawDownloadBtn.on('click', function() {
                const blob = new Blob([rawOutputText], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `task_${taskId}_raw_output.txt`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            });

            // Clear terminal
            clearBtn.on('click', function() {
                if (confirm('Are you sure you want to clear the terminal? This will only clear the view, not the actual task output.')) {
                    terminalOutput.empty();
                }
            });

            // Cancel task
            cancelBtn.on('click', function() {
                if (confirm('Are you sure you want to cancel this task?')) {
                    $.ajax({
                        url: `/api/task/${taskId}/cancel`,
                        method: 'POST',
                        success: function(data) {
                            if (data.success) {
                                alert('Task cancelled successfully');
                                loadTaskDetails();
                            } else {
                                alert(`Failed to cancel task: ${data.error}`);
                            }
                        },
                        error: function(xhr, status, error) {
                            alert(`Error: ${error}`);
                        }
                    });
                }
            });

            // Refresh button
            refreshBtn.on('click', function() {
                loadTaskDetails();
            });

            // Auto-scroll toggle
            autoScrollToggle.on('change', function() {
                autoScroll = $(this).is(':checked');

                if (autoScroll) {
                    const terminal = $('.terminal');
                    terminal.scrollTop(terminal[0].scrollHeight);
                }
            });

            // Scroll to bottom button
            scrollToBottomBtn.on('click', function() {
                const terminal = $('.terminal');
                terminal.scrollTop(terminal[0].scrollHeight);
            });

            // Initialize
            loadTaskDetails();
            initWebSocket();

            // Set up auto-refresh
            const refreshInterval = setInterval(function() {
                if (!taskCompleted) {
                    loadTaskDetails();
                } else {
                    clearInterval(refreshInterval);
                }
            }, 10000);  // Refresh every 10 seconds if task is not completed
        });
    </script>
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
    <title>Infinite AI - Task Details</title>
    <link rel="stylesheet" href="/static/css/bootstrap.min.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/bootstrap-icons.css">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
        }
        
        .navbar {
            background-color: #343a40 !important;
            box-shadow: 0 2px 4px rgba(0,0,0,.1);
        }
        
        .terminal {
            background-color: #212529;
            color: #cccccc;
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
            font-size: 0.9rem;
            padding: 1.5rem;
            border-radius: 0.5rem;
            overflow-y: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
            height: calc(100vh - 250px);
            position: relative;
        }
        
        .terminal-toolbar {
            position: sticky;
            top: 0;
            background-color: #212529;
            padding: 0.5rem;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            margin-bottom: 1rem;
            z-index: 100;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .status-badge {
            font-size: 0.85em;
            padding: 0.4em 0.7em;
            border-radius: 50rem;
        }
        
        .status-running { background-color: #007bff; color: white; }
        .status-completed { background-color: #28a745; color: white; }
        .status-failed { background-color: #dc3545; color: white; }
        .status-waiting { background-color: #ffc107; color: black; }
        
        .info-panel {
            background-color: white;
            border-radius: 0.5rem;
            padding: 1.25rem;
            margin-bottom: 1.25rem;
            box-shadow: 0 0.125rem 0.25rem rgba(0,0,0,.075);
        }
        
        .info-heading {
            font-size: 1.1rem;
            font-weight: 600;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 1px solid #e9ecef;
        }
        
        .info-list {
            list-style-type: none;
            padding-left: 0;
            margin-bottom: 0;
        }
        
        .info-list li {
            display: flex;
            justify-content: space-between;
            padding: 0.5rem 0;
            border-bottom: 1px solid #f8f9fa;
        }
        
        .info-list li:last-child {
            border-bottom: none;
        }
        
        .info-label {
            font-weight: 500;
            color: #495057;
        }
        
        .info-value {
            color: #212529;
            text-align: right;
            word-break: break-word;
        }
        
        .step-indicator {
            font-size: 0.85rem;
            margin-top: 0.25rem;
            color: #6c757d;
        }
        
        .env-list {
            max-height: 300px;
            overflow-y: auto;
        }
        
        .task-logs-btn {
            text-decoration: none !important;
        }
        
        .task-logs-btn i {
            margin-right: 0.5rem;
        }
        
        .terminal-output {
            line-height: 1.5;
        }
        
        .command-line {
            color: #50fa7b;
            font-weight: bold;
        }
        
        .error-output {
            color: #ff5555;
        }
        
        .success-output {
            color: #50fa7b;
        }
        
        .step-header {
            color: #bd93f9;
            font-weight: bold;
            margin-top: 1rem;
            margin-bottom: 0.5rem;
            border-bottom: 1px solid #44475a;
            padding-bottom: 0.25rem;
        }
        
        .output-header {
            color: #8be9fd;
            font-weight: bold;
            margin-top: 0.5rem;
            margin-bottom: 0.5rem;
        }
        
        .terminal-actions {
            position: absolute;
            top: 1rem;
            right: 1rem;
            display: flex;
            gap: 0.5rem;
            z-index: 10;
        }
        
        .terminal-actions button {
            background-color: rgba(255, 255, 255, 0.1);
            border: none;
            color: #ccc;
            border-radius: 0.25rem;
            padding: 0.25rem 0.5rem;
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .terminal-actions button:hover {
            background-color: rgba(255, 255, 255, 0.2);
        }
        
        .spinner-container {
            display: inline-block;
            margin-left: 0.5rem;
        }
        
        #environmentDetails .command-available {
            color: #50fa7b;
        }
        
        #environmentDetails .command-unavailable {
            color: #ff5555;
        }
        
        .task-controls-row {
            margin-bottom: 1rem;
        }
        
        .terminal-mode-tabs {
            margin-bottom: 1rem;
        }
        
        .terminal-mode-tabs .nav-link {
            padding: 0.25rem 0.75rem;
            font-size: 0.85rem;
        }
        
        .terminal-mode-tabs .nav-link.active {
            font-weight: 600;
        }
        
        .blinking-cursor::after {
            content: '▋';
            color: #ccc;
            animation: blink 1s step-end infinite;
        }
        
        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0; }
        }
    </style>
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="container-fluid">
            <a class="navbar-brand" href="/">
                <i class="bi bi-robot"></i> Infinite AI Agent
            </a>
            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                <span class="navbar-toggler-icon"></span>
            </button>
            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav">
                    <li class="nav-item">
                        <a class="nav-link" href="/"><i class="bi bi-house-fill"></i> Home</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/logs"><i class="bi bi-journal-text"></i> System Logs</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/history"><i class="bi bi-clock-history"></i> Task History</a>
                    </li>
                </ul>
            </div>
        </div>
    </nav>

    <div class="container-fluid py-4">
        <div class="row">
            <!-- Left Column - Task Info -->
            <div class="col-lg-3">
                <div class="d-flex justify-content-between align-items-center mb-3">
                    <h2 class="mb-0">Task Details</h2>
                </div>
                
                <div class="task-controls-row">
                    <a href="/" class="btn btn-outline-secondary">
                        <i class="bi bi-arrow-left"></i> Back to Dashboard
                    </a>
                    <a href="/task_logs/{{ task_id }}" class="btn btn-outline-primary ms-2 task-logs-btn">
                        <i class="bi bi-file-text"></i> View Detailed Logs
                    </a>
                </div>
                
                <div class="info-panel">
                    <div class="info-heading d-flex justify-content-between align-items-center">
                        <span><i class="bi bi-info-circle"></i> Task Information</span>
                        <span class="badge status-badge" id="taskStatusBadge">Loading...</span>
                    </div>
                    <ul class="info-list">
                        <li>
                            <span class="info-label">ID:</span>
                            <span class="info-value" id="taskId">{{ task_id }}</span>
                        </li>
                        <li>
                            <span class="info-label">Goal:</span>
                            <span class="info-value" id="taskGoal">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Created:</span>
                            <span class="info-value" id="taskCreated">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Duration:</span>
                            <span class="info-value" id="taskDuration">Loading...</span>
                        </li>
                        <li>
                            <span class="info-label">Current Step:</span>
                            <span class="info-value">
                                <span id="taskStep">Loading...</span>
                                <div class="spinner-container" id="stepSpinner" style="display: none;">
                                    <div class="spinner-border spinner-border-sm text-primary" role="status">
                                        <span class="visually-hidden">Loading...</span>
                                    </div>
                                </div>
                            </span>
                        </li>
                    </ul>
                </div>
                
                <div class="info-panel">
                    <div class="info-heading">
                        <i class="bi bi-hdd-rack"></i> Environment
                    </div>
                    <div id="environmentDetails" class="env-list">
                        <div class="text-center py-3">
                            <div class="spinner-border text-primary" role="status">
                                <span class="visually-hidden">Loading...</span>
                            </div>
                            <p class="mt-2 text-muted">Loading environment details...</p>
                        </div>
                    </div>
                </div>
                
                <div class="info-panel">
                    <div class="info-heading">
                        <i class="bi bi-tools"></i> Task Controls
                    </div>
                    <div class="d-grid gap-2">
                        <button class="btn btn-outline-primary" id="refreshBtn">
                            <i class="bi bi-arrow-clockwise"></i> Refresh Status
                        </button>
                        <button class="btn btn-outline-danger" id="cancelBtn" disabled>
                            <i class="bi bi-x-circle"></i> Cancel Task
                        </button>
                    </div>
                </div>
            </div>
            
            <!-- Right Column - Terminal Output -->
            <div class="col-lg-9">
                <div class="card border-0 shadow-sm">
                    <div class="card-header bg-dark text-white d-flex justify-content-between align-items-center py-3">
                        <h5 class="mb-0">
                            <i class="bi bi-terminal"></i> Task Execution
                            <span class="step-indicator" id="taskProgress">Step <span id="currentStep">0</span> of <span id="totalSteps">?</span></span>
                        </h5>
                        <div>
                            <div class="form-check form-switch d-inline-block me-2">
                                <input class="form-check-input" type="checkbox" id="autoScrollToggle" checked>
                                <label class="form-check-label text-white" for="autoScrollToggle">Auto-scroll</label>
                            </div>
                            <button class="btn btn-sm btn-outline-light ms-2" id="scrollToBottomBtn">
                                <i class="bi bi-arrow-down"></i>
                            </button>
                        </div>
                    </div>
                    
                    <div class="card-body p-0">
                        <ul class="nav nav-tabs terminal-mode-tabs" id="outputModeTabs">
                            <li class="nav-item">
                                <a class="nav-link active" id="formatted-tab" data-bs-toggle="tab" href="#formatted">Formatted Output</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" id="raw-tab" data-bs-toggle="tab" href="#raw">Raw Output</a>
                            </li>
                        </ul>
                        
                        <div class="tab-content">
                            <div class="tab-pane fade show active" id="formatted">
                                <div class="terminal">
                                    <div class="terminal-actions">
                                        <button id="copyBtn" title="Copy to clipboard">
                                            <i class="bi bi-clipboard"></i> Copy
                                        </button>
                                        <button id="downloadBtn" title="Download output">
                                            <i class="bi bi-download"></i> Download
                                        </button>
                                        <button id="clearBtn" title="Clear terminal">
                                            <i class="bi bi-trash"></i> Clear
                                        </button>
                                    </div>
                                    <div class="terminal-output" id="terminalOutput">
                                        <div class="text-center py-5" id="loadingOutput">
                                            <div class="spinner-border text-light" role="status">
                                                <span class="visually-hidden">Loading...</span>
                                            </div>
                                            <p class="mt-3 text-muted">Loading task output...</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="tab-pane fade" id="raw">
                                <div class="terminal">
                                    <div class="terminal-actions">
                                        <button id="rawCopyBtn" title="Copy raw output">
                                            <i class="bi bi-clipboard"></i> Copy
                                        </button>
                                        <button id="rawDownloadBtn" title="Download raw output">
                                            <i class="bi bi-download"></i> Download
                                        </button>
                                    </div>
                                    <pre id="rawOutput" class="mb-0">Loading raw output...</pre>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
    <script src="/static/js/bootstrap.bundle.min.js"></script>
    <script>
        $(document).ready(function() {
            // Elements
            const taskId = '{{ task_id }}';
            const terminalOutput = $('#terminalOutput');
            const rawOutput = $('#rawOutput');
            const loadingOutput = $('#loadingOutput');
            const taskStatusBadge = $('#taskStatusBadge');
            const environmentDetails = $('#environmentDetails');
            const copyBtn = $('#copyBtn');
            const downloadBtn = $('#downloadBtn');
            const clearBtn = $('#clearBtn');
            const rawCopyBtn = $('#rawCopyBtn');
            const rawDownloadBtn = $('#rawDownloadBtn');
            const cancelBtn = $('#cancelBtn');
            const refreshBtn = $('#refreshBtn');
            const autoScrollToggle = $('#autoScrollToggle');
            const scrollToBottomBtn = $('#scrollToBottomBtn');
            const stepSpinner = $('#stepSpinner');
            
            // Variables
            let autoScroll = true;
            let ws = null;
            let rawOutputText = '';
            let taskCompleted = false;
            
            // Format functions
            function formatDuration(seconds) {
                if (!seconds) return 'N/A';
                
                const minutes = Math.floor(seconds / 60);
                const remainingSeconds = seconds % 60;
                
                if (minutes === 0) {
                    return `${remainingSeconds}s`;
                } else {
                    return `${minutes}m ${remainingSeconds}s`;
                }
            }
            
            function formatDate(dateString) {
                if (!dateString) return 'N/A';
                return new Date(dateString).toLocaleString();
            }
            
            function getStatusBadgeClass(status) {
                if (status.includes('running') || status.includes('starting')) {
                    return 'status-running';
                } else if (status.includes('completed')) {
                    return 'status-completed';
                } else if (status.includes('failed') || status.includes('error')) {
                    return 'status-failed';
                } else if (status.includes('waiting')) {
                    return 'status-waiting';
                } else {
                    return 'bg-secondary';
                }
            }
            
            // Function to highlight terminal output
            function highlightOutput(text) {
                // Split the output by step headers
                const parts = text.split(/^---\s+Step\s+\d+\s+\([A-Z]+\)\s+---$/gm);
                const stepHeaders = text.match(/^---\s+Step\s+\d+\s+\([A-Z]+\)\s+---$/gm) || [];
                
                let formattedOutput = '';
                
                if (parts.length <= 1) {
                    // No step headers found, just return the text with error/success highlighting
                    return highlightErrorsAndSuccess(text);
                }
                
                // Process each step
                for (let i = 0; i < stepHeaders.length; i++) {
                    const header = stepHeaders[i];
                    const content = parts[i + 1] || '';
                    
                    // Add the step header
                    formattedOutput += `<div class="step-header">${header}</div>`;
                    
                    // Split the step content into code and output
                    const outputIndex = content.indexOf('--- Output ---');
                    
                    if (outputIndex !== -1) {
                        const code = content.substring(0, outputIndex).trim();
                        const output = content.substring(outputIndex + 14).trim();
                        
                        // Add the code section
                        formattedOutput += `<div class="command-line">${escapeHtml(code)}</div>`;
                        
                        // Add the output header
                        formattedOutput += `<div class="output-header">--- Output ---</div>`;
                        
                        // Add the highlighted output
                        formattedOutput += highlightErrorsAndSuccess(output);
                    } else {
                        // If we can't split it, just add the whole content
                        formattedOutput += highlightErrorsAndSuccess(content);
                    }
                }
                
                return formattedOutput;
            }
            
            // Function to highlight errors and success messages
            function highlightErrorsAndSuccess(text) {
                let html = escapeHtml(text);
                
                // Highlight error messages
                html = html.replace(/(?:^|\n)(error|exception|traceback|fail|\[erro?r?\]|❌)/gi, 
                                   (match) => `<span class="error-output">${match}</span>`);
                
                // Highlight success messages
                html = html.replace(/(?:^|\n)(success|completed|done|✅)/gi, 
                                   (match) => `<span class="success-output">${match}</span>`);
                
                // Highlight commands with $
                html = html.replace(/(?:^|\n)\$\s+(.*?)(?=\n|$)/g, 
                                   (match, cmd) => `<span class="command-line">$ ${cmd}</span>`);
                
                return html;
            }
            
            // Helper function to escape HTML
            function escapeHtml(text) {
                return text
                    .replace(/&/g, '&amp;')
                    .replace(/</g, '&lt;')
                    .replace(/>/g, '&gt;')
                    .replace(/"/g, '&quot;')
                    .replace(/'/g, '&#039;');
            }
            
            // Function to add blinking cursor if task is still running
            function addBlinkingCursor(element, isRunning) {
                // Remove existing cursor
                element.find('.blinking-cursor').remove();
                
                // Add new cursor if task is running
                if (isRunning) {
                    element.append('<span class="blinking-cursor"></span>');
                }
            }
            
            // Function to load task details
            function loadTaskDetails() {
                $.ajax({
                    url: `/api/task/${taskId}`,
                    method: 'GET',
                    success: function(data) {
                        if (data.error) {
                            terminalOutput.html(`<div class="error-output">Error: ${data.error}</div>`);
                            rawOutput.text(`Error: ${data.error}`);
                            loadingOutput.hide();
                            return;
                        }
                        
                        // Update task info
                        $('#taskGoal').text(data.goal || 'N/A');
                        $('#taskStep').text(data.step || '0');
                        $('#currentStep').text(data.step || '0');
                        
                        // Update task status badge
                        const status = data.status || 'unknown';
                        taskStatusBadge.text(status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(status)}`);
                        
                        // Check if task is completed
                        taskCompleted = status.includes('completed') || status.includes('failed');
                        
                        // Update cancel button
                        cancelBtn.prop('disabled', taskCompleted);
                        
                        // Update created time
                        if (data.created) {
                            $('#taskCreated').text(formatDate(data.created));
                        }
                        
                        // Update duration
                        if (data.duration) {
                            $('#taskDuration').text(formatDuration(data.duration));
                        } else if (data.start_time) {
                            const duration = Math.floor(Date.now() / 1000 - data.start_time);
                            $('#taskDuration').text(formatDuration(duration) + ' (running)');
                        }
                        
                        // Show step spinner if task is running
                        if (!taskCompleted && status.includes('running')) {
                            stepSpinner.show();
                        } else {
                            stepSpinner.hide();
                        }
                        
                        // Update terminal output
                        updateTerminalOutput(data.output || '');
                        
                        // Add blinking cursor if task is still running
                        addBlinkingCursor(terminalOutput, !taskCompleted);
                        
                        // Update environment details
                        updateEnvironmentDetails(data.environment);
                    },
                    error: function(xhr, status, error) {
                        terminalOutput.html(`<div class="error-output">Error loading task: ${error}</div>`);
                        rawOutput.text(`Error loading task: ${error}`);
                        loadingOutput.hide();
                    }
                });
            }
            
            // Function to update terminal output
            function updateTerminalOutput(output) {
                if (!output) {
                    return;
                }
                
                // Store raw output
                rawOutputText = output;
                rawOutput.text(output);
                
                // Format and display terminal output
                const highlightedOutput = highlightOutput(output);
                terminalOutput.html(highlightedOutput);
                loadingOutput.hide();
                
                // Scroll to bottom if auto-scroll is enabled
                if (autoScroll) {
                    const terminal = $('.terminal');
                    terminal.scrollTop(terminal[0].scrollHeight);
                }
            }
            
            // Function to update environment details
            function updateEnvironmentDetails(env) {
                if (!env) {
                    environmentDetails.html('<div class="text-muted">No environment information available</div>');
                    return;
                }
                
                let html = '<ul class="list-group list-group-flush">';
                
                // Process user and root status
                html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                    <span>User</span>
                    <span>${env.user || 'unknown'} ${env.is_root ? '(root)' : ''}</span>
                </li>`;
                
                // OS info
                if (env.os_info) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>OS</span>
                        <span>${env.os_info}</span>
                    </li>`;
                }
                
                // Kernel
                if (env.kernel) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Kernel</span>
                        <span>${env.kernel}</span>
                    </li>`;
                }
                
                // Working directory
                if (env.working_dir) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Working Directory</span>
                        <span>${env.working_dir}</span>
                    </li>`;
                }
                
                // Docker status
                if (env.docker_status) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Docker</span>
                        <span>${env.docker_status}${env.docker_running ? ` (${env.docker_running})` : ''}</span>
                    </li>`;
                }
                
                // Available commands
                if (env.available_commands) {
                    html += `<li class="list-group-item">
                        <div>Available Commands</div>
                        <div class="mt-1">`;
                    
                    if (typeof env.available_commands === 'string') {
                        // Handle string format
                        const commands = env.available_commands.split(/\s+/).filter(Boolean);
                        commands.forEach(cmd => {
                            html += `<span class="badge bg-secondary me-1">${cmd}</span>`;
                        });
                    } else if (typeof env.available_commands === 'object') {
                        // Handle object format
                        for (const [cmd, status] of Object.entries(env.available_commands)) {
                            const isAvailable = status === 'available';
                            html += `<span class="badge ${isAvailable ? 'bg-success' : 'bg-danger'} me-1">${cmd}</span>`;
                        }
                    }
                    
                    html += `</div>
                    </li>`;
                }
                
                // Memory info
                if (env.memory) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Memory</span>
                        <span>${env.memory}</span>
                    </li>`;
                }
                
                // Free disk space
                if (env.free_disk_space) {
                    html += `<li class="list-group-item d-flex justify-content-between align-items-center">
                        <span>Free Disk Space</span>
                        <span>${env.free_disk_space}</span>
                    </li>`;
                }
                
                html += '</ul>';
                environmentDetails.html(html);
            }
            
            // Initialize WebSocket connection
            function initWebSocket() {
                ws = new WebSocket(`ws://${window.location.host}/ws`);
                
                ws.onopen = function() {
                    console.log('WebSocket connected');
                };
                
                ws.onmessage = function(event) {
                    const data = JSON.parse(event.data);
                    
                    if (data.type === 'task_update' && data.id === taskId) {
                        // Update terminal output
                        updateTerminalOutput(data.output);
                        
                        // Update task status and step
                        taskStatusBadge.text(data.status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(data.status)}`);
                        
                        if (data.step) {
                            $('#taskStep').text(data.step);
                            $('#currentStep').text(data.step);
                            stepSpinner.show();
                        }
                    } else if (data.type === 'task_complete' && data.id === taskId) {
                        // Update terminal output
                        updateTerminalOutput(data.output);
                        
                        // Update task status
                        taskStatusBadge.text(data.status);
                        taskStatusBadge.removeClass().addClass(`badge status-badge ${getStatusBadgeClass(data.status)}`);
                        
                        // Update task completion
                        taskCompleted = true;
                        cancelBtn.prop('disabled', true);
                        stepSpinner.hide();
                        
                        // Remove blinking cursor
                        addBlinkingCursor(terminalOutput, false);
                        
                        // Reload task details to get final information
                        loadTaskDetails();
                    }
                };
                
                ws.onclose = function() {
                    console.log('WebSocket disconnected');
                    // Try to reconnect after a delay if the task is not completed
                    if (!taskCompleted) {
                        setTimeout(initWebSocket, 5000);
                    }
                };
                
                ws.onerror = function(error) {
                    console.error('WebSocket error:', error);
                };
            }
            
            // Copy terminal output
            copyBtn.on('click', function() {
                const text = terminalOutput.text();
                navigator.clipboard.writeText(text)
                    .then(() => {
                        const originalText = copyBtn.html();
                        copyBtn.html('<i class="bi bi-check"></i> Copied!');
                        setTimeout(() => {
                            copyBtn.html(originalText);
                        }, 2000);
                    })
                    .catch(err => {
                        alert('Failed to copy: ' + err);
                    });
            });
            
            // Copy raw output
            rawCopyBtn.on('click', function() {
                navigator.clipboard.writeText(rawOutputText)
                    .then(() => {
                        const originalText = rawCopyBtn.html();
                        rawCopyBtn.html('<i class="bi bi-check"></i> Copied!');
                        setTimeout(() => {
                            rawCopyBtn.html(originalText);
                        }, 2000);
                    })
                    .catch(err => {
                        alert('Failed to copy: ' + err);
                    });
            });
            
            // Download terminal output
            downloadBtn.on('click', function() {
                const text = terminalOutput.text();
                const blob = new Blob([text], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `task_${taskId}_output.txt`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            });
            
            // Download raw output
            rawDownloadBtn.on('click', function() {
                const blob = new Blob([rawOutputText], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `task_${taskId}_raw_output.txt`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            });
            
            // Clear terminal
            clearBtn.on('click', function() {
                if (confirm('Are you sure you want to clear the terminal? This will only clear the view, not the actual task output.')) {
                    terminalOutput.empty();
                }
            });
            
            // Cancel task
            cancelBtn.on('click', function() {
                if (confirm('Are you sure you want to cancel this task?')) {
                    $.ajax({
                        url: `/api/task/${taskId}/cancel`,
                        method: 'POST',
                        success: function(data) {
                            if (data.success) {
                                alert('Task cancelled successfully');
                                loadTaskDetails();
                            } else {
                                alert(`Failed to cancel task: ${data.error}`);
                            }
                        },
                        error: function(xhr, status, error) {
                            alert(`Error: ${error}`);
                        }
                    });
                }
            });
            
            // Refresh button
            refreshBtn.on('click', function() {
                loadTaskDetails();
            });
            
            // Auto-scroll toggle
            autoScrollToggle.on('change', function() {
                autoScroll = $(this).is(':checked');
                
                if (autoScroll) {
                    const terminal = $('.terminal');
                    terminal.scrollTop(terminal[0].scrollHeight);
                }
            });
            
            // Scroll to bottom button
            scrollToBottomBtn.on('click', function() {
                const terminal = $('.terminal');
                terminal.scrollTop(terminal[0].scrollHeight);
            });
            
            // Initialize
            loadTaskDetails();
            initWebSocket();
            
            // Set up auto-refresh
            const refreshInterval = setInterval(function() {
                if (!taskCompleted) {
                    loadTaskDetails();
                } else {
                    clearInterval(refreshInterval);
                }
            }, 10000);  // Refresh every 10 seconds if task is not completed
        });
    </script>
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
