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

# Create directories and set proper permissions
mkdir -p "$WORKDIR" "$WORKDIR/bin" "$WORKDIR/logs" "$WORKDIR/ui" "$WORKDIR/tmp"
chmod 755 "$WORKDIR"
chmod 755 "$WORKDIR/bin"

log(){ printf "[%(%F %T)T] %s\n" -1 "$*" | tee -a "$LOG" ; }
