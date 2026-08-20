#!/usr/bin/env bash
set -e

mkdir -p /root/fileserver/public
echo "hello public" > /root/fileserver/public/welcome.txt

cat <<'EOF' > /root/fileserver/server.py
import os

def get_file(base_dir, user_path):
    # Vulnerable implementation
    full_path = os.path.join(base_dir, user_path)
    if os.path.exists(full_path):
        return full_path
    return None
EOF
