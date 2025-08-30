# Unified Command Markdown (UCM) — v1.2

UCM is a four-verb interface: **get | run | set | result**. One action per fence. Each fence has a unique `#id` within a run.

## Verbs
- **get**: retrieve read-only content (files, URLs, text ranges).
- **run**: execute tools (code, shell, HTTP requests in read-only by default).
- **set**: write or mutate artifacts (requires confirmation for destructive ops).
- **result**: the *only* place an action returns its machine-consumable outcome.

## Fence shape (illustrative)
```ucm get#g1 path="./README.md" range="1..50"
# Body optional for GET when needed (e.g., inline sources)
```

```ucm run#r1 lang="python" timeout="10s"
print("hello"); x=1+1; print(x)
```

```ucm set#s1 path="./out.txt" mode="write"
Hello
```

```ucm result#r1 for="r1"
{
  "status": "ok",
  "data": {"stdout": "hello\n2\n"},
  "cache_key": "sha256:...",
  "cache_hit": false,
  "warnings": []
}
```

```ucm result#r1 for="r1"
{
  "status": "error",
  "error": { "code": "EXEC_TIMEOUT", "message": "Command timed out after 10s" },
  "cache_key": "sha256:...",
  "cache_hit": false
}
```

## Constraints
- One fence per action file; `#id` is unique per run.
- **Every action MUST emit a `result` fence** (success or error).
- Success target: **≥ 95%** of actions end with `status:"ok"`.
- `result` JSON includes `cache_key` and `cache_hit`; on error include a structured `error` object.
- `run sh` streams to `STREAM__*.log`; `result` summarizes final state.
- HTTP is **read-only** unless `mutate=true` and rationale provided.
- No guessing URLs; explicit `url` only.

See **docs/logging.md** for how results are recorded and replayed.
