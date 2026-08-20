#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq jq

cat <<'EOF' > /root/stream.json
[
  {"id": 1, "name": "alice", "active": true},
  {"id": 2, "name": "bob", "active": false},
  {"id": 3, "name": "charlie", "active": true,
EOF
