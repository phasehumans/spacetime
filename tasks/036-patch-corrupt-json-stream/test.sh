#!/usr/bin/env bash
set -e

if [ ! -f /root/stream.json ]; then
    echo "/root/stream.json missing"
    exit 1
fi

COUNT=$(jq 'length' /root/stream.json 2>/dev/null || echo "0")
if [ "$COUNT" -ne 3 ]; then
    echo "Invalid JSON or incorrect element count: expected 3, got $COUNT"
    exit 1
fi

echo "JSON repair verified"
exit 0
