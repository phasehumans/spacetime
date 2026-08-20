#!/usr/bin/env bash
set -e

mkdir -p /root/apiserver
cat <<'EOF' > /root/apiserver/requirements.txt
requests==2.18.4
urllib3==1.22
flask==0.12.2
EOF

touch /root/apiserver/AUDIT.md
