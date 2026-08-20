#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq jq

mkdir -p /var/data
cat <<'EOF' > /var/data/cluster.json
{
  "cluster_id": "prod-us-east-1",
  "nodes": [
    {"name": "worker-1", "ip": "10.0.1.15", "status": "Ready", "drain": false},
    {"name": "worker-2", "ip": "10.0.1.16", "status": "NotReady", "drain": false},
    {"name": "worker-3", "ip": "10.0.1.12", "status": "Ready", "drain": true},
    {"name": "worker-4", "ip": "10.0.1.18", "status": "Ready", "drain": false}
  ]
}
EOF
rm -f /root/healthy_nodes.json
