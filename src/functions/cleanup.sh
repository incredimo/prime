# Clean old setup if requested
if [[ "$*" == *"--clean"* ]] || [[ "$*" == *"-c"* ]]; then
  echo "🧹 Cleaning old installation..."
  run_elevated pkill -f ollama 2>/dev/null || true
  pkill -f "python.*agent.py" 2>/dev/null || true
  run_elevated rm -rf "$WORKDIR" 2>/dev/null || true
  run_elevated rm -f /etc/sudoers.d/90-$ME-ai 2>/dev/null || true
  echo "✅ Cleanup complete. Starting fresh installation."
fi
