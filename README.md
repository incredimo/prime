# Prime:  Ollama-powered AI Agent with Web UI

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

Prime is a powerful, self-contained AI agent that runs locally using Ollama models. It features a modern web UI, automatic model management, and is specifically designed to work seamlessly in Windows Subsystem for Linux (WSL) environments.

## 🚀 Features

- **Local AI Execution**: Runs entirely on your machine using Ollama models
- **Web UI**: Modern, responsive interface for interacting with the agent
- **WSL Compatibility**: Designed to work seamlessly in Windows Subsystem for Linux
- **Automatic Model Management**: Handles model downloading and initialization
- **Persistent History**: Keeps track of all your interactions
- **Real-time Logs**: Monitor the agent's activities in real-time
- **Modular Architecture**: Built with a clean, modular codebase for easy maintenance

## 📋 Requirements

- Linux or Windows with WSL
- Python 3.8+
- Ollama (installed automatically if not present)
- 8GB+ RAM (16GB+ recommended for larger models)
- 10GB+ free disk space

## 🔧 Installation

### One-Line Installation

```bash
curl https://raw.githubusercontent.com/incredimo/prime/main/prime.sh | bash
```

### Manual Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/incredimo/prime.git
   cd prime
   ```

2. Run the installation script:
   ```bash
   bash prime.sh
   ```

## 🎮 Usage

After installation, you can start the agent with:

```bash
cd ~/prime
./start_agent.sh
```

### Web Interface

Access the web interface at:
- http://localhost:8080
- http://[your-ip-address]:8080 (for access from other devices on your network)

### API Endpoints

Prime exposes several API endpoints:

- `POST /api/goal`: Submit a new goal for the agent to accomplish
  ```bash
  curl -X POST http://localhost:8000/api/goal -d '{"text":"install docker"}' -H 'Content-Type: application/json'
  ```

- `GET /api/status`: Check the status of the agent and Ollama
  ```bash
  curl http://localhost:8000/api/status
  ```

- `GET /api/tasks`: List all active tasks
- `GET /api/history`: View task history
- `GET /api/logs`: Access agent logs

## 🏗️ Project Structure

The project is organized into a modular structure:

```
prime/
├── build.ps1           # Build script to generate prime.sh
├── bump.ps1            # Version bumping and release script
├── prime.sh            # Main installation script (generated)
├── version.txt         # Current version
└── src/                # Source files
    ├── header/         # Script header files
    ├── functions/      # Utility functions
    ├── ollama/         # Ollama installation and service
    ├── python/         # Python environment and agent code
    ├── ui/             # Web UI files
    │   ├── static/     # CSS and JavaScript
    │   └── templates/  # HTML templates
    └── scripts/        # Helper scripts
```

## 🧠 Supported Models

Prime works with any model supported by Ollama, including:

- Gemma 3 (default)
- Llama 3.2
- Mistral
- Phi-3
- And many more!

To change the default model, edit the `MODEL` variable in `src/python/agent.py`.

## 🔄 Development Workflow

### Building from Source

To build the `prime.sh` script from source:

```powershell
.\build.ps1
```

### Version Bumping

To bump the version, commit changes, and create a tag:

```powershell
# Bump patch version (0.1.0 -> 0.1.1)
.\bump.ps1

# Bump minor version (0.1.0 -> 0.2.0)
.\bump.ps1 -VersionType minor

# Bump major version (0.1.0 -> 1.0.0)
.\bump.ps1 -VersionType major

# Custom commit message
.\bump.ps1 -CommitMessage "Fix Ollama JSON parsing issue"

# Dry run (show what would happen without making changes)
.\bump.ps1 -DryRun
```

## 🔍 Troubleshooting

### Ollama Connection Issues

If the agent can't connect to Ollama:

1. Check if Ollama is running:
   ```bash
   ollama ps
   ```

2. Start Ollama manually:
   ```bash
   ollama serve
   ```

3. For WSL users, try the Windows helper script:
   ```powershell
   powershell.exe -ExecutionPolicy Bypass -File C:/repo/prime/windows_ollama_helper.ps1
   ```

### Model Issues

If you encounter issues with the default model:

1. Try pulling it manually:
   ```bash
   ollama pull gemma3
   ```

2. Try a different model by editing the `MODEL` variable in `agent.py`.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgements

- [Ollama](https://github.com/ollama/ollama) for providing the local LLM runtime
- [FastAPI](https://fastapi.tiangolo.com/) for the API framework
- All the open-source contributors who make projects like this possible
