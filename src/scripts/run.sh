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
