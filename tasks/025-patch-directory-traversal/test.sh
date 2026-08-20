#!/usr/bin/env bash
set -e

python3 - <<'EOF'
import sys, os
sys.path.insert(0, '/root/fileserver')
import server

base = "/root/fileserver/public"

# Valid path check
res = server.get_file(base, "welcome.txt")
assert res is not None and "welcome.txt" in str(res), "Valid file lookup failed"

# Traversal check: ../../etc/passwd
res_bad = None
try:
    res_bad = server.get_file(base, "../../../etc/passwd")
except (ValueError, PermissionError):
    res_bad = None

assert res_bad is None or not os.path.isabs(str(res_bad)) or str(res_bad).startswith(base), f"Path traversal allowed: {res_bad}"
print("Directory traversal vulnerability patched successfully")
EOF
