#!/usr/bin/env bash
set -e

if [ ! -f /root/bad_commit.txt ]; then
    echo "/root/bad_commit.txt missing"
    exit 1
fi

SUBMITTED=$(tr -d '[:space:]' < /root/bad_commit.txt)
EXPECTED=$(tr -d '[:space:]' < /root/.expected_bad_commit)

if [ "$SUBMITTED" != "$EXPECTED" ]; then
    echo "Incorrect bad commit identified: expected '$EXPECTED', got '$SUBMITTED'"
    exit 1
fi

echo "Git bisect regression identified successfully"
exit 0
