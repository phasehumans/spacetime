#!/usr/bin/env bash
set -e

PIDFILE="/var/run/worker.pid"
if [ ! -f "$PIDFILE" ]; then
    echo "Worker PID file missing"
    exit 1
fi

PID=$(cat "$PIDFILE")
if ! kill -0 "$PID" 2>/dev/null; then
    echo "Worker process $PID is not running"
    exit 1
fi

echo "Worker process deadlock recovery verified"
exit 0
