#!/usr/bin/env bash
set -e

ENV_FILE="/root/service/.env"
EXAMPLE_FILE="/root/service/.env.example"

if [ ! -f "$ENV_FILE" ]; then
    echo ".env file missing"
    exit 1
fi

PERMS=$(stat -c "%a" "$ENV_FILE")
if [ "$PERMS" != "600" ]; then
    echo "Permissions on .env must be 600, found: $PERMS"
    exit 1
fi

if grep -q "COMPROMISED_KEY_999" "$ENV_FILE"; then
    echo "Compromised key still present"
    exit 1
fi

KEY=$(grep "^SECRET_KEY=" "$ENV_FILE" | cut -d '=' -f2 | tr -d '"' | tr -d "'" | tr -d '[:space:]')
if [ ${#KEY} -ne 32 ]; then
    echo "SECRET_KEY must be exactly 32 hex characters, got length ${#KEY}"
    exit 1
fi

if ! echo "$KEY" | grep -Eq '^[0-9a-fA-F]{32}$'; then
    echo "SECRET_KEY must be hexadecimal"
    exit 1
fi

if ! grep -q "SECRET_KEY=replace_with_32_char_hex" "$EXAMPLE_FILE"; then
    echo ".env.example missing required placeholder"
    exit 1
fi

echo "Secret rotation verified"
exit 0
