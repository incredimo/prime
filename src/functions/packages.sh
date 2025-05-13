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
