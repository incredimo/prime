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
