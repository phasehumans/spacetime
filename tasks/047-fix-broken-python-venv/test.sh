#!/usr/bin/env bash
set -e

if [ ! -f /opt/app-venv/bin/python3 ]; then
    echo "/opt/app-venv/bin/python3 missing"
    exit 1
fi

OUT=$(/opt/app-venv/bin/python3 -c "import sys; print(f'venv_ok_{sys.version_info[0]}')" 2>&1)
if [[ "$OUT" != *"venv_ok_3"* ]]; then
    echo "Python venv binary execution failed: $OUT"
    exit 1
fi

echo "Python virtual environment repair verified"
exit 0
