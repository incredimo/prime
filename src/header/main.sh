#
# prime.sh   —  WSL-aware, Ollama-powered, gemma3 agent with UI
# --------------------------------------------------------------------
set -euo pipefail
IFS=$'\n\t'
export DEBIAN_FRONTEND=noninteractive

ME="$(whoami)"
WORKDIR="$HOME/infinite_ai"
LOG="$WORKDIR/install.log"
