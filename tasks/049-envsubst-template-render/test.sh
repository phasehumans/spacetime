#!/usr/bin/env bash
set -e

CONFIG="/etc/app/config.yaml"
if [ ! -f "$CONFIG" ]; then
    echo "$CONFIG missing"
    exit 1
fi

if grep -q '\${' "$CONFIG"; then
    echo "Unrendered variables still present in $CONFIG"
    exit 1
fi

if ! grep -q "host: postgres.internal" "$CONFIG" || ! grep -q "super_secret_jwt_key_9911" "$CONFIG"; then
    echo "Rendered values missing from $CONFIG"
    exit 1
fi

echo "Configuration template rendering verified"
exit 0
