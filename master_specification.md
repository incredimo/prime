# Prime 2.0 — Master Specification (Self‑Tooling, v1.3)

**Scope:** A terminal AI assistant that auto‑adapts to a single available LLM (tiny → frontier), right‑sizes cognition, executes via **UCM** (Unified Command Markdown), and maintains a **file‑first, replayable audit trail**. New in v1.3: **self‑tooling** — the LLM can design, build, validate, register, and selectively expose **its own tools**.

---

## 1) Project Aim (one line)

Build a **terminal AI assistant** that **measures the model**, **governs cognitive load**, and **wins with tools and memory**, so even small/open models complete complex tasks through **smaller, verified steps** — with a **fully file‑first audit trail**.

---

## 2) Problem Statement (why this exists)

Static prompts waste strong models and overload small ones. Prime must **profile the connected model**, dynamically **govern cognitive load**, and **compensate with tools/memory**. In v1.3, Prime also enables the model to **create its own tools** when gaps are detected, while ensuring safety, testability, and selective prompt exposure.

---

## 3) Success Criteria (v1)

* Parser errors **< 1%** of turns
* **100%** destructive ops confirmed; **0** unconfirmed runs
* **≥ 95%** actions emit a valid `result` fence
* TTFA: **< 2s warm / < 6s cold** on typical local models
* Tool Foundry: ≥ **90%** tool build passes on first try (smoke tests)

---

## 4) Core Architecture

### 4.1 Control Loop (PDCA ⇄ MAPE‑K)

**Monitor → Analyze → Plan → Execute (+ Knowledge).**

* Monitor: instrument every step; collect validator outputs, timings, costs
* Analyze: compare to control limits; detect drift/errors
* Plan: compose/adjust plan; propose tool creation if gaps recur
* Execute: run UCM actions; apply validators & safety gates
* Knowledge: persist evidence, reflections, retrieval traces

### 4.2 Unified Command Markdown (UCM) — v1.2

**Verbs:** `get | run | set | result` (one action per fence; unique `#id`).

* `get` — read‑only retrieval (files/URLs/ranges)
* `run` — execute tools (code, shell, HTTP read‑only by default)
* `set` — write/mutate artifacts and registry entries
* `result` — machine‑consumable outcome (JSON) with `cache_key`, `cache_hit`

> **Invariant:** **100%** of actions must yield a `result` fence. Success target: **≥ 95%** with `status:"ok"`.

### 4.3 Session Capability Profiler (SCP)

Probe signals: Grammar Fidelity, Instruction Following, Decomposition, Grounded Reasoning, Self‑Correction, Tool‑Use Readiness, CoT Sensitivity, Context Discipline.

* Compute **Brilliance Score** **B ∈ \[0,1]**, choose Mode: **M4 ≥0.85**, **M3 0.70–0.84**, **M2 0.55–0.69**, **M1 <0.55**.
* Persist to `.prime/sessions/<id>/caps.json`.

### 4.3.1 Scoring → Brilliance B

We compute **B ∈ [0,1]** as a weighted average of 5 probe scores:

| Signal                | Probe               | Weight | Scoring |
| --------------------- | ------------------- | :----: | ------- |
| Grammar Fidelity      | p1_grammar.md       |  0.30  | 1.0 perfect · 0.5 fixable · 0.0 invalid |
| Instruction Following | p2_instruction.md   |  0.20  | 1.0 exact · 0.0 otherwise |
| Decomposition         | p3_decompose.md     |  0.20  | 0.0–1.0 clarity & step quality |
| Self-Correction       | p4_fix_fence.md     |  0.15  | 1.0 fixed · 0.0 not fixed |
| Tool-Use Readiness    | p5_tool_decision.md |  0.15  | 1.0 correct choice · 0.0 otherwise |

**Formula:** `B = Σ(weight_i * score_i)` → Modes: M4 ≥ 0.85, M3 0.70–0.84, M2 0.55–0.69, M1 < 0.55.

### 4.4 Cognitive Load Governor (CLG)

Mode → limits, prompting style, verification cadence. Promote after **3 clean turns**; demote on repeated guardrail hits.

| Mode | Plan size                | Prompting style                                | Verification          | Tool bias                                | Sampling                                 |
| ---: | ------------------------ | ---------------------------------------------- | --------------------- | ---------------------------------------- | ---------------------------------------- |
|   M4 | macro‑plan (4–8 actions) | concise CoT where needed; structured rationale | spot‑checks           | moderate; RAG/code when faster than text | 1–2 passes                               |
|   M3 | medium (2–4)             | ReAct, brief self‑ask, short CoT               | verify critical steps | prefer RAG/code for math/lookup          | 2–3 passes on hard Qs                    |
|   M2 | small (1–2)              | ReAct + reflection; **short** reasoning only   | verify each step      | strong tool bias                         | **self‑consistency** (3–5 bags)          |
|   M1 | micro (single)           | template‑driven; no free‑form CoT              | verify every step     | tool‑first always                        | self‑consistency (5–7), fallback recipes |

---

## 5) Quality System (how we guarantee outcomes)

### 5.1 Jidoka (stop‑the‑line)

Automatic halts on abnormal conditions: schema violation, unsafe/mutating attempts, budget breach, low confidence.

### 5.2 Poka‑yoke (mistake‑proofing)

* Strict tool schemas (JSON Schema / typed adapters)
* Allow‑lists (files/URLs); parameter ranges; idempotent writes
* Pre‑/post‑conditions per step; compensating plans (no multi-step rollback in v1)

### 5.3 Statistical Process Control (SPC)

* Track FPY/RTY, lead time, error codes by step
* Control charts with 3‑sigma limits; breaches trigger diagnosis and CAPA

### 5.4 Evidence Locker (auditability)

Persist: prompts, fences, tool calls, artifacts, retrievals, validator outputs, reflections, decisions.

---

## 6) Tooling & Memory (how small models win)

* **RAG (read‑only default):** fetch from local docs/web, inject facts; log queries & sources
* **Code execution (PAL/PoT):** short scripts to compute/validate; always return `result` JSON
* **Formal solvers (optional):** constrained solvers exposed via `run`
* **Function/MCP adapters:** represented as `run` with clear attrs; outputs captured in `result`
* **Memory:** short‑term summaries + vector store of prior runs/docs; retrieval steps logged under `retrieval/`
* **Evidence Locker (Memory):** Human-readable markdown memories under `.prime/memory/` are indexed by an embedded full-text search engine (**Bleve**). On write, tags + full text are indexed with tokenization, normalization, stemming, and stop-word removal; queries are ranked via BM25. Index files live entirely on disk (`.prime/memory/index.bleve/`), preserving Prime’s file-first design while enabling fast, high-quality retrieval.

---

## 7) Self‑Tooling System (Tool Foundry)

**Goal:** allow the LLM to **design, implement, test, register, and use** new tools via UCM, with strict safety and selective exposure.

### 7.1 Concepts & Directories

```
.tools/
  <tool-name>/
    toolcard.json       # metadata & contract
    impl.(py|ps1|sh)    # implementation (sandboxed)
    tests/
      smoke.json        # sample inputs/outputs
.prime/registry/
  tools/
    <tool-name>@<semver>.json  # frozen ToolCard snapshot
  index.json            # active tools, versions, tags
```

### 7.2 ToolCard Schema (authoritative)

```json
{
  "name": "string",                   
  "version": "semver",                
  "summary": "one line purpose",      
  "inputs": {"$schema": "json-schema"},
  "outputs": {"$schema": "json-schema"},
  "runner": {
    "type": "python|sh|powershell",
    "entry": "impl.py|impl.sh|impl.ps1",
    "timeout_s": 10,
    "stdin": false
  },
  "resources": {
    "network": false,
    "filesystem": "read-only|sandbox|none",
    "cpu_limit": 1.0,
    "mem_mb": 512
  },
  "relevance": {                       
    "triggers": [
      {"if_task_contains": ["rate limit", "quota"]},
      {"if_file_glob": "**/*.log"}
    ],
    "embedding_hints": ["api policy", "throttling"]
  },
  "risks": ["mutating", "high-cost"],
  "tags": ["text", "parse"],
  "owner": "prime",
  "created": "ISO-8601",
  "changelog": [
    {"version": "1.0.0", "notes": "initial"}
  ]
}
```

### 7.3 Tool Synthesis Pipeline (agent‑driven)

1. **Detect need** (Planner): repeated failure/gap → propose tool
2. **Design** (LLM): emit `toolcard.json` + `impl` draft
3. **Build** (Sandbox): run smoke tests from `tests/` (no network, RO FS)
4. **Register** (Gate): write snapshot to `.prime/registry/tools/` and update `index.json`
5. **Expose** (Palette): make eligible via Relevance Engine (below)
6. **Measure**: track FPY/latency/defect codes per tool version (SPC)

### 7.4 UCM Patterns for Self‑Tooling

**Design & Write:**

```ucm set#s1 path=".tools/loggrep/toolcard.json" mode="write"
{ ...ToolCard JSON... }
```

```ucm set#s2 path=".tools/loggrep/impl.py" mode="write"
# impl code here
```

**Build & Test:**

```ucm run#r1 lang="python" timeout="10s"
# execute smoke tests under sandbox; output summary JSON
```

```ucm result#r1 for="r1"
{ "status":"ok", "data": {"tests": {"passed":3,"failed":0}}, "cache_key":"...","cache_hit":false }
```

**Register:**

```ucm set#s3 path=".prime/registry/tools/loggrep@1.0.0.json" mode="write"
{ ...frozen ToolCard... }
```

```ucm set#s4 path=".prime/registry/index.json" mode="update"
{ "add": {"name":"loggrep","version":"1.0.0","active":true,"tags":["log"]} }
```

> All mutating `set` to registry requires a human or policy confirmation if risk level ≥ medium.

### 7.5 Relevance & Visibility Engine (what the model sees)

* **Goal:** only show the **relevant** tools in the model’s prompt to control context size.
* **Eligibility function** (evaluated per turn):

  * Hard triggers (task text, file globs, MIME, repo path)
  * Embedding similarity between task text and tool `summary`/`tags`
  * Mode‑aware caps: M1–M2 → top **3–5** tools; M3–M4 → **6–8**
  * Safety filters (risk, network, mutating) based on user policy

**Prompt Packing (Tool Palette)**

```
[TOOLS]
- loggrep@1.0.0 — parse log files; inputs {...}; outputs {...}; timeout 5s; RO FS
- ratelimit-scan@0.3.0 — extract API limits; inputs {...}; outputs {...}; RO FS
```

(Only eligible tools are included in the message alongside instructions.)

### 7.6 Tool Lifecycle & Governance

* **Propose → Build → Register → Deprecate** (semver; frozen snapshots)
* **SPC metrics** per tool version (FPY, latency, error codes)
* **CAPA**: repeated defects trigger tool fixes or deprecation
* **Supply chain**: signed ToolCards (optional), checksums for impl files

### 7.7 Safety & Sandbox

* Default **no network**, **read‑only FS**, strict CPU/mem/timeouts
* Allowlist interpreters (`python`, `sh`, `powershell`)
* Permission elevation requires explicit ToolCard flags + confirmation

---

## 8) File‑First Logging & Replay (authoritative)

**Principles:** everything on disk; outbound logged before send; inbound logged verbatim before parsing; deterministic replay via content‑addressed cache.

**Layout:**

```
.prime/
  memory/
    index.bleve/        # Bleve index (managed by library)
    docs/               # optional snapshots
  sessions/<session_id>/
    caps.json
    clg.json
    model.md
    env.json
    runs/<ts>_<slug>/
      0001_USER.md
      0002_PLAN.md
      0003_COMMAND__g*_get.md
      0004_RESULT__g*.md
      0005_COMMAND__r*_run.md
      0006_STREAM__r*.stdout.log
      0007_RESULT__r*.md
      0008_WARN.md
      artifacts/
      retrieval/
```

**Caching keys:**

* `get`: hash(verb, attrs#id‑less, locator, range, version)
* `run`: hash(verb, attrs#id‑less, toolchain version, body, env allowlist, cwd snapshot)
* `set`: hash(target, bytes, append)

---

## 9) Safety & Policy (defensive only)

* Confirm destructive patterns (`rm -rf`, `mkfs`, `dd if=`, mass deletes/renames)
* Read‑only HTTP by default; mutation requires `mutate=true` + rationale
* Redact secrets; never write to dotfiles unless asked
* Registry writes gated by policy; risky ToolCards require manual confirm

---

## 10) Terminal UI (operator ergonomics)

* **Persistent frame & progress bar**; dim gray for pending, bright yellow for active (`⇣`), green for completed (`|`)
* Minimal, sharp, professional TUI; no emojis; overflow text **fades**
* Progressive reveal of tasks; always show current Mode, budgets, and active tools

### HITL Confirmation

* Confirmation prompts: Y/N/A (Approve)
* Timeout: default No (deny)

---

## 11) Worked Examples (multi‑Mode)

**Task:** “Find API rate limits in the docs and summarize.”

**M4 (macro)**

* `get url=docs` → parse → summarize (single pass) → `result`

**M2 (small; tool‑first)**

* `get url` (RAG prefetch) → `result`
* `run lang=python` (extract limits via regex) → `result`
* `run lang=python` (format summary JSON) → `result`

**M1 (micro; self‑consistency)**

* `get url` → `result`
* `run lang=python` (find first limit) → `result`
* repeat per section; bag 5–7 attempts; choose consensus numbers

**Self‑Tooling variant (if regex fails repeatedly):**

* Propose `ratelimit-scan` ToolCard + impl
* Build/tests in sandbox → register v1.0.0 → palette exposure for future runs

---

## 12) Deliverables (near‑term)

* `docs/`: `README.md`, `ucm.md`, `logging.md`, `clg.md`, `scp.md`, **this master spec**
* `.prime/` scaffold with sample run + retrieval traces
* **Golden UCM parser tests** (10 fixtures)
* **SCP probe battery** (5 probes) + scoring script (TBD)
* **Tool Foundry starter**: registry layout, ToolCard schema, sandbox runner stub

---

## 13) Definition of Done (v1)

* Session starts with SCP; Mode banner printed & persisted
* Orchestrator enforces CLG; streaming output; each action yields a `result`
* File‑first trace present; replay reproduces results (cache permitting)
* Self‑tooling path operational: design → build → register → selective exposure
* Parser tests pass; safety confirmations verified; metrics met

---

## 14) Appendices

**A. Prompt Palettes**: ReAct (M2–M4), Self‑Ask (force decomposition), Reflection (M1–M2), Role priming (task‑specific), Self‑consistency (multi‑sample adjudication). Short reasoning bias by default; long CoT only if SCP tolerance is high.

**B. Minimal SCP Probes**: (5) grammar, instruction following, decomposition, fix broken fence, tool decision.

**C. Registry `index.json` shape (suggested)**

```json
{
  "tools": [
    {"name":"ratelimit-scan","version":"1.0.0","active":true,"tags":["docs","api"],"risk":"low"}
  ],
  "updated": "ISO-8601"
}
```

**D. Tool test record (example `smoke.json`)**

```json
{
  "cases": [
    {"name":"simple", "input": {"path":"sample.md"}, "expect": {"limits_found": ">=1"}}
  ]
}
```

**E. Safety policy knobs (YAML)**

```yaml
http_mutation: require-confirm
fs_write: allowlist
max_timeout_s: 20
max_parallel_runs: 2
registry_confirm_risk: [medium, high]
```

— End of Master Specification —
