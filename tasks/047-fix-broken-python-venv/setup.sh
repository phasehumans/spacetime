#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq python3 python3-venv

mkdir -p /opt/app-venv/bin
ln -sf /nonexistent/bin/python3.11 /opt/app-venv/bin/python3
ln -sf /nonexistent/bin/python3.11 /opt/app-venv/bin/python
cat <<'EOF' > /opt/app-venv/pyvenv.cfg
home = /nonexistent/bin
include-system-site-packages = false
version = 3.11.0
EOF
