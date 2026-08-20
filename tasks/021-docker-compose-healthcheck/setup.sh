#!/usr/bin/env bash
set -e

mkdir -p /root/app
cat <<'EOF' > /root/app/docker-compose.yml
version: '3.8'

services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    environment:
      - REDIS_HOST=cache_db
      - REDIS_PORT=6380
    depends_on:
      redis_service:
        condition: service_healthy

  redis_service:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD-SHELL", "exit 1"]
      interval: 5s
      timeout: 3s
      retries: 3
EOF

cat <<'EOF' > /root/app/.env
REDIS_HOST=cache_db
REDIS_PORT=6380
EOF
