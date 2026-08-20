#!/usr/bin/env bash
set -e

if [ ! -f /root/data.json ]; then
    echo "/root/data.json missing"
    exit 1
fi

PAYLOAD=$(jq -r '.payload' /root/data.json 2>/dev/null || echo "")
if [ "$PAYLOAD" != "spacetime_authenticated_ok" ]; then
    echo "Incorrect payload: $PAYLOAD"
    exit 1
fi

echo "JWT Auth and API query verified"
exit 0
