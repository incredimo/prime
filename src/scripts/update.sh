#!/usr/bin/env bash

# update.sh - Script to update the Infinite AI Agent
# This script removes the current installation and installs the latest version

# Exit on any error
set -e

# Display banner
echo "╔═════════════════════════════════════════════════════════════╗"
echo "║ 🔄 INFINITE AI UPDATER                                      ║"
echo "╚═════════════════════════════════════════════════════════════╝"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Go to parent directory (should be the infinite_agent directory)
cd "$SCRIPT_DIR/.."
PARENT_DIR="$(pwd)"

echo "📋 Current directory: $PARENT_DIR"
echo "🗑️ Removing current installation..."

# Go up one more level
cd ..

# Remove the infinite_agent directory
rm -rf "$PARENT_DIR"
echo "✅ Removed old installation"

# Download and run the latest prime.sh
echo "🔄 Downloading and running latest version..."
curl -s https://raw.githubusercontent.com/incredimo/prime/refs/heads/main/prime.sh | bash

echo "✅ Update completed successfully!"
echo ""
echo "📋 Next steps:"
echo "  cd ~/infinite_ai"
echo "  ./start_agent.sh    # Runs the agent with Web UI and auto-restart"
