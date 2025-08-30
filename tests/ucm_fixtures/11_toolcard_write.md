```ucm set#sT1 path=".tools/loggrep/toolcard.json" mode="write"
{"name":"loggrep","version":"1.0.0","summary":"parse logs",
 "inputs":{"$schema":"..."},
 "outputs":{"$schema":"..."},
 "runner":{"type":"sh","entry":"impl.sh","timeout_s":5,"stdin":false},
 "resources":{"network":false,"filesystem":"read-only","cpu_limit":1.0,"mem_mb":128},
 "relevance":{"triggers":[{"if_task_contains":["log"]}],"search_keywords":["logs","log parsing","grep"]},
 "tags":["log"],"owner":"prime","created":"2025-08-30","changelog":[{"version":"1.0.0","notes":"initial"}]}
```