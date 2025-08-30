Here’s a single, drop-in file you can include at the repo root.

---

# instruction\_for\_developer.md

**Purpose:** this document tells you exactly how to take the existing spec + scaffolding and implement Prime 2.0 in Go to a production standard — without ambiguity.

---

## 0) Core philosophies (build these into every decision)

* **File-first, replayable**: everything important lands on disk under `.prime/` and can be replayed deterministically.
* **Single-model, adaptive cognition**: profile the connected model (SCP) → govern cognitive load (CLG) so small models succeed via smaller, verified steps.
* **UCM v1.2 contract**: one action per fence (`get|run|set|result`). **100%** actions must emit a `result` (≥95% with `"status":"ok"`).
* **Tool-first, not CoT-first**: short reasoning, strong bias to tools, code, and RAG vs. long free-form chains.
* **Safety by default**: read-only unless explicitly elevated, destructive ops confirmed via HITL, structured errors everywhere.
* **No external DBs**: Bleve provides embedded full-text search; no vector/embedding stack in v1.
* **Serial v1**: no parallel execution/DAG in v1; keep ordering simple and logs deterministic.
* **Low dependency runtime**: OS-level sandboxing, not containers, for v1.

---

## 1) Flat project structure (you will implement)

Prime must run with `go run .` from the repo root. Keep the structure shallow:

```
.
├── main.go
├── go.mod
├── go.sum
├── config.yaml.example
│
├── agent/          # Orchestrator (MAPE-K control loop)
│   └── agent.go
├── clg/            # Cognitive Load Governor
│   └── clg.go
├── scp/            # Session Capability Profiler (probes + scoring)
│   ├── scp.go
│   └── scoring.go
├── ucm/            # UCM v1.2 data model + parser
│   ├── ucm.go
│   └── parser.go
├── runtime/        # Executors for get/run/set; result emission
│   ├── runtime.go
│   ├── exec_get.go
│   ├── exec_run.go
│   └── exec_set.go
├── logging/        # File-first writer + cache + replay helpers
│   └── logger.go
├── memory/         # Bleve-powered Evidence Locker
│   ├── memory.go
│   ├── indexer.go
│   └── store.go
├── tools/          # Tool Foundry + registry + relevance engine
│   ├── foundry.go
│   ├── registry.go
│   └── relevance.go
├── llm/            # Model adapters (single-model runtime)
│   ├── llm.go
│   ├── gemini.go   # gemini-1.5-flash
│   └── ollama.go   # gemma3
├── tui/            # Bubble Tea UI (no emojis)
│   ├── tui.go
│   ├── view_main.go
│   ├── view_confirm.go
│   └── styles.go
│
├── docs/           # You already have these (do not change semantics)
├── probes/         # SCP prompts
└── tests/          # Parser fixtures; do not change IDs or semantics
```

---

## 2) What each code package must do (bullet-point contract)

### `/main.go`

* Parse config (`config.yaml` with sensible defaults from `config.yaml.example`).
* Initialize logging, Bleve index, model adapter, SCP→CLG, and TUI.
* Start the agent control loop and wire it to the TUI (channel message bus).
* Exit with non-zero code on critical invariant breach (invalid UCM, failed `result` emission).

### `/agent/agent.go` — Orchestrator (MAPE-K)

* **Monitor**: timestamp and measure every stage (LLM call, parse, execute).
* **Analyze**: maintain counters for guardrail hits, FPY/RTY, and promote/demote per CLG rules.
* **Plan**: request plans from LLM using prompt palettes (per Mode), inject tool palette (from `tools/relevance`), and constraints.
* **Execute**: parse UCM; for each action, call `runtime` and enforce HITL when required.
* **Knowledge**: write `.prime/sessions/<id>/runs/<ts>_<slug>/...` including `env.json`.
* **HITL**: when `runtime` flags a confirmable operation, send a TUI confirmation request (Y/N/A). Block until UI reply (or timeout→deny).

### `/clg/clg.go` — Cognitive Load Governor

* Accept Brilliance Score **B ∈ \[0,1]**; map to Mode (M1–M4) using thresholds in `docs/clg.md`.
* Expose:

  * `PlanSizeCaps()` → batch size (M4: 4–8, M3: 2–4, M2: 1–2, M1: 1).
  * `PromptPalette()` → which prompt strategy to use (ReAct, reflection, etc.).
  * `VerificationCadence()` → how aggressively to verify.
  * Promotion after **3 clean turns**, demotion after **2 consecutive** hits or 1 severe breach.

### `/scp/scp.go`, `/scp/scoring.go`

* Run the **5 probes** from `probes/` against the active LLM.
* Score each probe and compute **B** with the weights in `docs/scp.md`.
* Persist results to `.prime/sessions/<id>/caps.json` and `.prime/sessions/<id>/clg.json`.

### `/ucm/ucm.go`, `/ucm/parser.go`

* Implement UCM v1.2:

  * Fences: `get|run|set|result`, unique `#id` per run.
  * **Every** action must pair with a `result` (success or error).
  * `result` must include `cache_key`, `cache_hit`; error form includes `error.code`, `error.message`.
  * `#` at beginning of a line **inside a fence body** is a comment and must be ignored by the runtime.
  * Default HTTP: `GET|HEAD` only; mutation requires `mutate=true` + rationale.
  * `stdin` attr accepts `on|off` (default `off`).
* Parse fixtures under `tests/ucm_fixtures`. Your parser must pass **all** existing cases as-is.

### `/runtime/runtime.go`, `exec_get.go`, `exec_run.go`, `exec_set.go`

* Execute parsed actions **serially** (v1).
* **get**: read files / URLs (HTTP read-only by default). Honor ranges. Compute cache key:

  * `hash(verb, attrs#id-less, locator, range, version)`.
* **run**: execute code/shell/HTTP tools. Stream stdout to `0006_STREAM__*.log`. Cache key:

  * `hash(verb, attrs#id-less, toolchain version, body, env allowlist, exec_cwd_abs_hash)`.
  * **exec\_cwd\_abs\_hash** = SHA-256 of the **absolute** working directory of the executed process (usually per-run sandbox root).
* **set**: write/mutate artifacts. Gate destructive operations behind HITL confirmation.
* Always emit a `result` fence (success or error). On failure:

  * `{"status":"error","error":{"code":"...","message":"..."},"cache_key":"...","cache_hit":false}`.
* Sandbox (v1):

  * OS process with timeouts and resource hints (e.g., `exec.CommandContext`).
  * Network **off** by default for tools; file writes restricted to per-run sandbox dir.
  * No containers in v1.

### `/logging/logger.go`

* Implement the exact layout in `docs/logging.md`:

  * `.prime/sessions/<session_id>/runs/<ts>_<slug>/...`
  * Write outbound before send; inbound verbatim before parsing.
  * Include `env.json` per run (exe path, backend/model, tool palette, allowlisted env).
  * Caching: expose helpers to compute keys and mark `cache_hit`.
  * Fast-fail missing cache entries on replay; write `0008_WARN.md`.

### `/memory/*` — Bleve Evidence Locker

* Dependencies: `github.com/blevesearch/bleve`.
* `indexer.go`: open/create index at `.prime/memory/index.bleve/`; index new/updated markdown files (front-matter + body), delete on removal.
* Analyzer: English (tokenize, lowercase, stop-words, **stemming**).
* Ranking: BM25 default.
* `memory.go`: facade with `Save(doc)→Index`, `Search(query, k)→[]Hit{path, score, snippet}`.
* `store.go`: read/write files under `.prime/memory/docs` or other memory paths.
* On indexing errors: return structured error; system continues in degraded mode (as per `docs/memory.md`).

### `/tools/*` — Tool Foundry, Registry, Relevance

* **Registry (`registry.go`)**

  * Read/write `.prime/registry/tools/<name>@<version>.json` (frozen ToolCards).
  * Maintain `.prime/registry/index.json` with `active` versions.
  * Gate writes via agent HITL + policy.
* **Foundry (`foundry.go`)**

  * Pattern: `set` ToolCard → `set` implementation → run smoke tests (no network, RO FS) → register snapshot → update index.
  * Produce SPC metrics (first-pass yield, latency, defect codes).
* **Relevance Engine (`relevance.go`)** (Bleve-centric, no embeddings)

  * Determine **eligible** tools per turn by:

    * Hard triggers (task string, file glob, MIME).
    * **Bleve query** combining: current task text (boosted) + ToolCard `search_keywords` + `summary` + `tags`.
    * Mode-aware caps: M1–M2 → top **3–5**; M3–M4 → **6–8**.
    * Apply policy filters: network/mutating risk, etc.
  * Return a **prompt palette** with concise tool descriptions and input/output schema tips.

### `/llm/*` — Adapters

* Unified interface: `Generate(ctx, prompt) (text, tokens, cost, err)` and optional `Stream`.
* Implement:

  * **Gemini** (`gemini-2.5-flash`): env var `GEMINI_API_KEY`; keep request/response logging minimal and redacted.
  * **Ollama** (`gemma3`): env var `OLLAMA_HOST` (default `http://localhost:11434`); model name overridable via config.
* SCP probes must run against the selected adapter; persist model metadata to `model.md`.

### `/tui/*` — Bubble Tea UI (professional; **no emojis**)

* Use `github.com/charmbracelet/bubbletea`, `lipgloss`, and `bubbles` as needed.
* **Design rules:**

  * No emojis. Minimal, sharp typography and spacing.
  * Persistent frame with:

    * Mode badge (M1–M4), budgets, model id.
    * Active plan list: dim gray for pending, bright yellow for active, green `|` for completed.
    * Stream pane showing live stdout lines (truncate with graceful fade).
    * Tools palette (eligible tools for this turn) and short hints.
    * Optional **Search** panel (Bleve top-K with snippet).
* **HITL confirm view**:

  * Prompt shows action, risk, and source (Tool Foundry or runtime).
  * Keys: `Y`=Yes, `N`=No, `A`=Always for this session; timeout defaults to **No**.
  * Emit a structured message back to agent.

---

## 3) Terminal UI preferences (strict)

* No emojis, no playful animations.
* Neutral palette, strong contrast for status states.
* Resist noise: truncation > scrolling walls; use fading for overflow lines.
* Always display: current Mode, model id, tool palette (eligible), budgets, and a clear progress indicator.
* Confirmations must show **exact** action attributes so operators know what they approve.

---

## 4) External references (where to read for authoritative details)

* **UCM grammar, constraints, examples** → `docs/ucm.md`.
* **Logging layout, cache keys, replay** → `docs/logging.md`.
* **Bleve Evidence Locker** → `docs/memory.md`.
* **SCP signals, probes, scoring → B → Mode** → `docs/scp.md`.
* **CLG Modes, technique matrix, cadence** → `docs/clg.md`.
* **Master system behavior, Tool Foundry, safety policy, HITL** → `master_specification.md`.
* **SCP probe prompts** → `probes/`.
* **Parser acceptance tests** → `tests/ucm_fixtures/` (run these early).

---

## 5) Config & environment

* **Example file**: `config.yaml.example`

  * `backend: gemini|ollama`
  * `ollama.host: http://localhost:11434`
  * `ollama.model: gemma3`
  * `gemini.model: gemini-2.5-flash`
  * `memory.index_path: .prime/memory/index.bleve`
  * `memory.max_results: 8`
  * `timeouts: { run_default_s: 10, http_default_s: 8 }`
  * `policy: { http_mutation: "require-confirm", fs_write: "allowlist" }`
* **Env vars**

  * `GEMINI_API_KEY` — required for Gemini adapter.
  * `OLLAMA_HOST` — optional; overrides config.

---

## 6) Execution & logging invariants (must not break)

* **Serial execution** in v1. No concurrent action runs.
* Each action produces a **single** `result` fence file with cache metadata.
* `env.json` is **per run**, not per session.
* Cache keys:

  * `get`: `hash(verb, attrs#id-less, locator, range, version)`
  * `run`: `hash(verb, attrs#id-less, toolchain version, body, env allowlist, exec_cwd_abs_hash)`
  * `set`: `hash(target, bytes, append)`
* Error results use structured form; they **must** be cached, too.
* Replay: missing cache → write `0008_WARN.md` and fail fast.

---

## 7) Tool Foundry specifics (developer checklist)

* **ToolCard schema**: include `relevance.triggers[]` and `relevance.search_keywords[]` — **no embeddings**.
* Registry writes:

  * Write frozen snapshot to `.prime/registry/tools/<name>@<semver>.json`.
  * Update `.prime/registry/index.json`.
  * Require HITL if risk ≥ medium.
* Relevance engine uses a **weighted Bleve query** combining: task text (boost), `search_keywords`, `summary`, `tags`.
* Smoke tests: no network, read-only filesystem; timeout per ToolCard.

---

## 8) Typical request flow (step-by-step)

1. **Start** app: read config, open Bleve, choose adapter, run SCP → compute **B** → select Mode.
2. **User input** received via TUI; agent builds prompt using CLG palette + eligible tools.
3. **LLM response** parsed as UCM plan.
4. **Execute** actions in order:

   * Write command file → compute cache → maybe serve from cache.
   * Stream output when applicable → write `STREAM__*.log`.
   * Emit `result` (ok/error) with `cache_key`, `cache_hit`.
   * If mutation/risky: pause for HITL (Y/N/A; timeout=No).
5. **Log** everything to `.prime/sessions/.../runs/<ts>_<slug>/...`, including `env.json`.
6. **Promote/demote** Mode depending on guardrail hits and clean turns.

---

## 9) Quality bars & acceptance criteria

* **Success metrics** (must meet):

  * Parser errors **< 1%** of turns (measured on fixtures + ad-hoc).
  * **100%** actions yield a `result`; **≥95%** with `"status":"ok"`.
  * TTFA: **< 2s warm / < 6s cold** on typical local models.
  * Tool Foundry smoke tests **≥ 90%** pass on first build.
* **Security/policy**:

  * Read-only HTTP by default; `mutate=true` + rationale to allow.
  * No writing to dotfiles unless explicitly asked.
  * Redact secrets in logs.
* **Non-goals v1**:

  * No parallel execution.
  * No embeddings/vector stores.
  * No containers/OCI isolation.
  * No background daemons.

---

## 10) Coding standards & dependencies

* **Go version**: ≥ 1.22 recommended.
* **Formatting & linting**: `go fmt`, `go vet`, and a linter (e.g., `golangci-lint`) pre-commit.
* **Errors**: wrap with context; prefer sentinel error codes for `runtime` (`EXEC_TIMEOUT`, `INDEX_WRITE_FAILED`, etc.).
* **Context**: pass `context.Context` into all long-running calls (LLM, HTTP, sandboxed processes).
* **Third-party libraries**:

  * `github.com/charmbracelet/bubbletea` (+ `lipgloss`, `bubbles`) for TUI.
  * `github.com/blevesearch/bleve` for search.
  * Native `net/http`, `os/exec` for runtime; avoid heavy frameworks.
* **Cross-platform**: Linux/macOS first; Windows OK where feasible (job objects for timeouts if needed).

---

## 11) Running locally

* ollama is already running (model `gemma3` pulled).
*  `GEMINI_API_KEY` is already available in the environment.
* `go run .` from repo root.
* First run will create `.prime/` and Bleve index at `.prime/memory/index.bleve/`.

---

## 12) Things to watch out for (pitfalls)

* Do **not** leak emojis or ANSI spam into logs; TUI can style but keep logs plain.
* Don’t forget to **ignore** `#` comment lines inside fence bodies when executing.
* Ensure `env.json` is **per run**, not session.
* Keep `exec_cwd_abs_hash` correct to avoid cache poisoning across directories.
* Never include embeddings or vector search — all relevance is **Bleve**.

---

## 13) “Done” checklists (per module)

**UCM Parser**

* [ ] Passes all `tests/ucm_fixtures/*`.
* [ ] Enforces unique `#id` per run.
* [ ] Accepts `stdin=on|off`, default off.
* [ ] Distinguishes HTTP read-only vs mutation.

**Runtime**

* [ ] Serial execution only.
* [ ] Streams `run` stdout to `0006_STREAM__*.log`.
* [ ] Emits `result` for every action; caches ok/errors.
* [ ] HITL gating works and times out to **No**.

**Logging**

* [ ] Writes exact tree structure from `docs/logging.md`.
* [ ] Correct cache key computation (incl. `exec_cwd_abs_hash`).
* [ ] Replay serves cached results; missing cache → `0008_WARN.md`.

**Memory (Bleve)**

* [ ] Indexes front-matter + body; English analyzer; BM25 ranking.
* [ ] Returns top-K with scores + snippets.
* [ ] Index errors don’t crash the run; return structured error.

**Tools**

* [ ] ToolCard schema supports `relevance.triggers`, `relevance.search_keywords`.
* [ ] Registry snapshots + index updates correct.
* [ ] Relevance engine queries Bleve (no embeddings).

**TUI**

* [ ] No emojis. Clear mode badge, progress, tools palette, stream pane.
* [ ] Confirmation view with Y/N/A, timeout default No.
* [ ] Optional Search panel shows Bleve results and snippets.

---

If anything is unclear, consult the referenced doc in **Section 4** first; those files are the source of truth for behavior and layout.
