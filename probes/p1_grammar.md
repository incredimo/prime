Emit a valid UCM GET + RESULT pair that reads the first 3 lines of `./README.md`. 
- Use `get#g1 path="./README.md" range="1..3"`
- Then emit `result#g1 for="g1"` with a JSON payload that includes `cache_key` and `cache_hit`.
