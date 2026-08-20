#!/usr/bin/env bash
set -e

OUT1=$(/opt/scripts/health.py 2>&1 || true)
OUT2=$(/opt/scripts/metrics.py 2>&1 || true)

if [ "$OUT1" != "health_ok" ] || [ "$OUT2" != "metrics_ok" ]; then
    echo "Scripts failed to execute properly: out1='$OUT1', out2='$OUT2'"
    exit 1
fi

echo "Shebang correction verified"
exit 0
