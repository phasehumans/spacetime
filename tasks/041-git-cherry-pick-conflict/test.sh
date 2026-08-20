#!/usr/bin/env bash
set -e

cd /root/project

# Check if cherry-pick is complete
if [ -d .git/sequencer ] || [ -f .git/CHERRY_PICK_HEAD ]; then
    echo "Cherry-pick still in progress"
    exit 1
fi

STATUS=$(git status --porcelain)
if [ -n "$STATUS" ]; then
    echo "Working tree is not clean: $STATUS"
    exit 1
fi

if ! grep -q "ENABLE_METRICS = True" config.py || ! grep -q "TIMEOUT = 60" config.py; then
    echo "config.py missing required settings after merge"
    exit 1
fi

echo "Git cherry-pick conflict resolution verified"
exit 0
