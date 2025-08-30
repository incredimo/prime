```ucm set#sT2 path=".tools/loggrep/impl.sh" mode="write"
#!/usr/bin/env sh
grep -n "$1" "$2"
```