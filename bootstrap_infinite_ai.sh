#!/usr/bin/env bash
#
# bootstrap_infinite_ai.sh   —  WSL-aware, Ollama-powered, Qwen3 agent
# --------------------------------------------------------------------
set -euo pipefail
IFS=$'\n\t'
export DEBIAN_FRONTEND=noninteractive

ME="$(whoami)"
WORKDIR="$HOME/infinite_ai"
LOG="$WORKDIR/install.log"
mkdir -p "$WORKDIR"

log(){ printf "[%(%F %T)T] %s\n" -1 "$*" | tee -a "$LOG" ; }

# ------------------------------------------------------------
# 1.  password-less sudo so the agent can apt-install later
# ------------------------------------------------------------
log "Configuring password-less sudo for $ME…"
sudo bash -c "echo '$ME ALL=(ALL) NOPASSWD:ALL' >/etc/sudoers.d/90-$ME-ai && chmod 0440 /etc/sudoers.d/90-$ME-ai"

# ------------------------------------------------------------
# 2.  base system packages
# ------------------------------------------------------------
log "Installing system prerequisites…"
sudo apt-get update -y
sudo apt-get upgrade -y
sudo apt-get install -y --no-install-recommends \
     python3 python3-venv python3-pip git curl wget build-essential \
     sqlite3 jq unzip net-tools htop tmux

# ------------------------------------------------------------
# 3.  detect / install Ollama
# ------------------------------------------------------------
OLLAMA_ENDPOINT="http://127.0.0.1:11434"
if curl -s --max-time 3 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
  log "Ollama service detected on Windows-side or local host."
else
  if command -v ollama >/dev/null 2>&1; then
      log "Ollama CLI present but daemon not running → starting it."
      nohup ollama serve >/dev/null 2>&1 &
      sleep 3
  else
      log "Ollama not found — installing inside WSL."
      curl -fsSL https://ollama.com/install.sh | sh
      nohup ollama serve >/dev/null 2>&1 &
      sleep 3
  fi
  # Re-check
  if ! curl -s --max-time 5 "$OLLAMA_ENDPOINT/api/tags" >/dev/null 2>&1; then
      log "FATAL: Ollama daemon still unreachable."; exit 1
  fi
fi

# ------------------------------------------------------------
# 4.  ensure qwen3 model exists
# ------------------------------------------------------------
if ! curl -s "$OLLAMA_ENDPOINT/api/tags" | grep -q '"name":"qwen3"'; then
  log "Pulling qwen3 model… (this may take a while)"
  ollama pull qwen3 || { log "Failed pulling qwen3"; exit 1; }
fi
log "qwen3 model ready."

# ------------------------------------------------------------
# 5.  python environment
# ------------------------------------------------------------
log "Creating workspace at $WORKDIR"
mkdir -p "$WORKDIR"/{skills,logs,infra}
cd "$WORKDIR"

log "Setting up Python venv…"
python3 -m venv venv
source venv/bin/activate

log "Installing Python libs…"
pip install --upgrade pip
pip install fastapi uvicorn duckdb tiktoken watchdog requests

# ------------------------------------------------------------
# 6.  immutable logger (never self-modified)
# ------------------------------------------------------------
cat > infra/logger.py <<'PY'
import datetime, pathlib, sys, os
WORKDIR = pathlib.Path(__file__).resolve().parents[1]
LOG_DIR = WORKDIR / "logs"; LOG_DIR.mkdir(exist_ok=True)
def log(msg: str):
    stamp = datetime.datetime.now().strftime("%F %T")
    line  = f"[{stamp}] {msg}"
    print(line, flush=True)
    with open(LOG_DIR / f"agent_{datetime.date.today()}.log","a") as f:
        f.write(line+"\n")
PY

# ------------------------------------------------------------
# 7.  agent.py  —  now powered by Ollama
# ------------------------------------------------------------
cat > agent.py <<'PY'
#!/usr/bin/env python3
import os, sys, subprocess, uuid, sqlite3, pathlib, json, importlib.util, datetime, threading, textwrap, re, requests
from fastapi import FastAPI
from pydantic import BaseModel
from infra.logger import log

# --------------------------- CONFIG ---------------------------
WORKDIR    = pathlib.Path(__file__).resolve().parent
SKILL_DIR  = WORKDIR / "skills"
DB_PATH    = WORKDIR / "skills.db"
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://127.0.0.1:11434")
MODEL      = os.getenv("OLLAMA_MODEL", "qwen3")
PORT       = int(os.getenv("INFINITE_AI_PORT", 8000))
# --------------------------------------------------------------

# ---------- memory ----------
conn = sqlite3.connect(DB_PATH)
cur  = conn.cursor()
cur.execute("""CREATE TABLE IF NOT EXISTS convo
               (id INTEGER PRIMARY KEY, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                role TEXT, content TEXT)""")
conn.commit()
def remember(r,c): cur.execute("INSERT INTO convo(role,content) VALUES(?,?)",(r,c)); conn.commit()

# ---------- Ollama wrapper ----------
def ollama_chat(prompt:str)->str:
    remember("user", prompt)
    log(f"→AI PROMPT  {prompt[:120]}…")
    payload = {"model": MODEL,
               "messages":[{"role":"system","content":
                 "You are an autonomous, root-capable agent inside WSL. "
                 "Return exactly one code block starting with #SH or #PY, "
                 "or #DONE when finished, or #SELFUPDATE followed by python code to replace agent.py."},
                {"role":"user","content":prompt}]}
    r = requests.post(f"{OLLAMA_URL}/api/chat", json=payload, timeout=600)
    r.raise_for_status()
    txt = r.json()["message"]["content"].strip()
    remember("assistant", txt)
    log(f"←AI REPLY   {txt[:120]}…")
    return txt

# ---------- execution ----------
def extract(txt:str):
    m = re.search(r"^#(SH|PY)\s*\n(.*)", txt, re.S|re.M)
    return (m.group(1), textwrap.dedent(m.group(2))) if m else (None,None)

def run_sh(code:str)->str:
    log(f"$ bash ‹‹\n{code}\n››")
    p = subprocess.run(code, shell=True, capture_output=True, text=True, timeout=1800)
    out = p.stdout + p.stderr
    log(out); return out

def run_py(code:str)->str:
    tmp = SKILL_DIR / f"tmp_{uuid.uuid4().hex}.py"
    tmp.write_text(code)
    return run_sh(f"python {tmp}")

def iterate(goal:str):
    step = ollama_chat(f"Goal: {goal}")
    while True:
        if "#DONE" in step.upper(): log("Goal complete."); break
        if "#SELFUPDATE" in step.upper():
            new_code = step.split("#SELFUPDATE",1)[1].strip()
            (WORKDIR/"agent.py").write_text(new_code)
            log("Self-updated code. Restarting…")
            os.execv(sys.executable, ["python", "agent.py"])
        kind, code = extract(step)
        if not kind: log("No code detected; abort."); break
        out = run_py(code) if kind=="PY" else run_sh(code)
        step = ollama_chat(f"Output:\n{out}\nNext?")

# ---------- interfaces ----------
app = FastAPI()
class Goal(BaseModel): text:str
@app.post("/goal"); async def g(g:Goal):
    log(f"API goal: {g.text}"); iterate(g.text); return{"ok":True}

def cli():
    while True:
        try: goal=input("\nGoal › ").strip()
        except EOFError: break
        if goal.lower() in {"exit","quit"}: break
        if goal: iterate(goal)

if __name__=="__main__":
    threading.Thread(target=lambda: __import__("uvicorn").run(
        app, host="0.0.0.0", port=PORT, log_level="warning"),daemon=True).start()
    log(f"REST API ready on http://127.0.0.1:{PORT}")
    cli()
PY
chmod +x agent.py

# ------------------------------------------------------------
# 8.  watchdog launcher
# ------------------------------------------------------------
cat > run.sh <<'SH'
#!/usr/bin/env bash
source "$(dirname "$0")/venv/bin/activate"
while true; do
  ./agent.py || true
  echo "[watchdog] agent exited — restarting in 5 s" | tee -a logs/watchdog.log
  sleep 5
done
SH
chmod +x run.sh

log "Bootstrap completed ✓"
echo "
Next steps:
  cd $WORKDIR
  source venv/bin/activate
  ./run.sh        # starts agent, watchdog, REST API
Interact:
  • Type goals in the terminal, or
  • curl -X POST http://localhost:8000/goal -d '{\"text\":\"<your goal>\"}'
"
