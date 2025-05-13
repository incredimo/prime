#
# prime.sh   —   Ollama-powered, gemma3 agent with UI
# --------------------------------------------------------------------
set -euo pipefail
IFS=$'\n\t'
export DEBIAN_FRONTEND=noninteractive

ME="$(whoami)"
WORKDIR="$HOME/prime"
LOG="$WORKDIR/install.log"
