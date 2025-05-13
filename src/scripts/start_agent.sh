# ------------------------------------------------------------
# 12. Create handy shortcut scripts
# ------------------------------------------------------------
cat > start_agent.sh <<'SH'
#!/usr/bin/env bash
cd "$(dirname "$0")" || exit 1
./run.sh
SH
chmod +x start_agent.sh
