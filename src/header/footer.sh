log "Bootstrap completed ✓"
echo "
╔═════════════════════════════════════════════════════════════╗
║ 🚀 PRIME AI BOOTSTRAP COMPLETED                          ║
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

cd "$WORKDIR"  && ./start_agent.sh 
