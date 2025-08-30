# Evidence Locker (Bleve)

Prime keeps human-readable memories (markdown) and a fast on-disk index (Bleve), no external DB.

## Paths

```

.prime/
memory/
index.bleve/        # Bleve index (managed by library)
docs/               # optional snapshots

```

## What is indexed

* Markdown **front-matter** (if present): `tags[]`, `title`, `type`, `run_id`, `source`, `created`.
* Markdown **body** text.
* **Exclusions:** binary artifacts, `artifacts/`, `retrieval/`, large blobs.

## Analyzer & Ranking

* Language: `en` analyzer (tokenize, lowercase, stop-words, **stemming**).
* Ranking: **BM25** (default Bleve similarity).

## Indexing lifecycle

* On write of a new memory file → index immediately (synchronous).
* On update/delete → reindex/delete by docID (relative path).
* On startup → open existing index (no full rescan).
* **Rebuild** on demand (policy/manual) by re-walking `.prime/`.

## Query API (internal)

* Inputs: query string (+ optional must/should tags).
* Output: Top-K paths with scores + snippets for TUI display.
* Safety: deny queries containing secrets; redact before display.

## Failure semantics

* Index errors produce a `result` with `status:"error"` and `error.code:"INDEX_WRITE_FAILED"`; run continues with degraded retrieval.

## TUI integration

* `⇣ Search` pane shows top-K hits; selecting a hit opens the doc and adds it to the current context plan.