#!/usr/bin/env bash
set -e

if [ ! -x /usr/local/bin/start-worker.sh ]; then
    echo "/usr/local/bin/start-worker.sh missing or not executable"
    exit 1
fi

if ! tmux has-session -t worker_pool 2>/dev/null; then
    echo "tmux session worker_pool is not running"
    exit 1
fi

echo "Detached tmux runner verified"
exit 0
