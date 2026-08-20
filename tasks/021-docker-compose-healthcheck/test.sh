#!/usr/bin/env bash
set -e

COMPOSE_FILE="/root/app/docker-compose.yml"
if [ ! -f "$COMPOSE_FILE" ]; then
    echo "docker-compose.yml missing"
    exit 1
fi

# Verify healthcheck test is not hardcoded failure
if grep -q "exit 1" "$COMPOSE_FILE"; then
    echo "Healthcheck still failing with exit 1"
    exit 1
fi

# Verify valid healthcheck exists
if ! grep -q "redis-cli" "$COMPOSE_FILE" && ! grep -q "exit 0" "$COMPOSE_FILE" && ! grep -q "ping" "$COMPOSE_FILE"; then
    echo "Valid redis healthcheck command not configured"
    exit 1
fi

echo "docker-compose healthcheck verified"
exit 0
