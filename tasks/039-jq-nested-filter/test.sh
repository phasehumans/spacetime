#!/usr/bin/env bash
set -e

if [ ! -f /root/healthy_nodes.json ]; then
    echo "/root/healthy_nodes.json missing"
    exit 1
fi

COUNT=$(jq 'length' /root/healthy_nodes.json 2>/dev/null || echo "0")
if [ "$COUNT" -ne 2 ]; then
    echo "Expected 2 healthy nodes, got $COUNT"
    exit 1
fi

FIRST=$(jq -r '.[0]' /root/healthy_nodes.json)
SECOND=$(jq -r '.[1]' /root/healthy_nodes.json)

if [ "$FIRST" != "10.0.1.15" ] || [ "$SECOND" != "10.0.1.18" ]; then
    echo "Incorrect IPs or ordering: first='$FIRST', second='$SECOND'"
    exit 1
fi

echo "JSON filter and extraction verified"
exit 0
