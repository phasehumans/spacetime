#!/usr/bin/env bash
set -e

cat <<'EOF' > /etc/resolv.conf
nameserver 127.0.0.99
nameserver 0.0.0.0
EOF
