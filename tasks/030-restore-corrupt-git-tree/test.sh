#!/usr/bin/env bash
set -e

cd /root/repo
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "Not on main branch: $CURRENT_BRANCH"
    exit 1
fi

if ! grep -q "hotfix patch" file.txt; then
    echo "Hotfix not merged into file.txt"
    exit 1
fi

STATUS=$(git status --porcelain)
if [ -n "$STATUS" ]; then
    echo "Working directory is not clean: $STATUS"
    exit 1
fi

echo "Corrupt git tree restoration and hotfix merge verified"
exit 0
