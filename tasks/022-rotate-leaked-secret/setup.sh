#!/usr/bin/env bash
set -e

mkdir -p /root/service
cat <<'EOF' > /root/service/.env
APP_NAME=spacetime-vault
DEBUG=false
SECRET_KEY=COMPROMISED_KEY_999
DATABASE_URL=postgres://user:pass@localhost:5432/db
EOF

touch /root/service/.env.example
chmod 777 /root/service/.env
