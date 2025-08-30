```ucm run#r3 lang="python" timeout="1ms"
print("too slow")
```

```ucm result#r3 for="r3"
{"status":"error","error":{"code":"EXEC_TIMEOUT","message":"Command timed out"},"cache_key":"sha256:z","cache_hit":false}