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
