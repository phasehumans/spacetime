#!/usr/bin/env bash
set -e

SCRIPT="/usr/local/bin/setup-firewall.sh"
if [ ! -x "$SCRIPT" ]; then
    echo "$SCRIPT missing or not executable"
    exit 1
fi

CONTENT=$(cat "$SCRIPT")
if ! echo "$CONTENT" | grep -q "ESTABLISHED,RELATED"; then
    echo "Missing ESTABLISHED,RELATED state rule"
    exit 1
fi

if ! echo "$CONTENT" | grep -q "\-\-limit 1/s" && ! echo "$CONTENT" | grep -q "\-\-limit 1/second"; then
    echo "Missing ICMP 1/s rate limit rule"
    exit 1
fi

if ! echo "$CONTENT" | grep -q "echo-request.*DROP" && ! echo "$CONTENT" | grep -q "icmp.*DROP"; then
    echo "Missing ICMP drop excess rule"
    exit 1
fi

echo "Iptables firewall configuration verified"
exit 0
