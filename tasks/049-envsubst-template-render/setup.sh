#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq gettext-base

mkdir -p /etc/app
cat <<'EOF' > /etc/app/config.template.yaml
server:
  port: 8080
database:
  host: ${DATABASE_HOST}
  port: ${DATABASE_PORT}
security:
  secret_key: "${API_SECRET}"
EOF

cat <<'EOF' > /etc/app/.env
DATABASE_HOST=postgres.internal
DATABASE_PORT=5432
API_SECRET=super_secret_jwt_key_9911
EOF

rm -f /etc/app/config.yaml
