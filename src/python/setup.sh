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
