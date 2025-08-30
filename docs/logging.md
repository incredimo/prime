# File-First Logging & Replay

**Principles**
- Everything important is on disk, no DB.
- Log outbound *before send*; inbound *verbatim* before parsing.
- Deterministic replay via content-addressed cache keys.

**Layout**

.prime/
memory/
index.bleve/          # Bleve on-disk index (managed by library)
docs/                 # optional: human notes snapshots
sessions/\<session\_id>/
caps.json
clg.json
model.md
runs/<ts>\_<slug>/
env.json            # exe path, backend/model, tool palette, allowlisted env
0001\_USER.md
0002\_PLAN.md
0003\_COMMAND\_\_g1\_get.md
0004\_RESULT\_\_g1.md
0005\_COMMAND\_\_r1\_run.md
0006\_STREAM\_\_r1.stdout.log
0007\_RESULT\_\_r1.md
0008\_WARN.md
index.md
artifacts/
retrieval/
0003\_rag.query.txt
0003\_rag.topk.json
0003\_rag.sources.md

```
**Caching (content-addressed)**
- `get`: hash(verb, attrs#id-less, locator, range, version)
- `run`: hash(verb, attrs#id-less, toolchain version, body, env allowlist, exec_cwd_abs_hash)
- `set`: hash(target, bytes, append)

**Field definition:** `exec_cwd_abs_hash` is the SHA-256 of the absolute working directory of the executed process (usually the per-run sandbox root). This prevents cross-directory cache collisions.

**Replay**
- Replays `STREAM__*`; serves cached `result` when `cache_hit=true`.
- Missing entries → fail fast; write `0008_WARN.md`.
- **Errors still produce results:** failed actions MUST emit a `result` fence with `status:"error"` and a structured `error` object. These are cached with keys the same as successful results.
- **Execution semantics (v1):** strictly **serial**; filenames remain numerically ordered. Parallel/DAG execution is deferred to v2.

**What we don’t store**
- Transport envelopes
- Prompt frames (only final message content)
- Binary blobs inside markdown (store in `artifacts/` and reference)
