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
