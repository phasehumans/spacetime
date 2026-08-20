#!/usr/bin/env bash
set -e

mkdir -p /root/firewall
cat <<'EOF' > /root/firewall/rules.sh
#!/usr/bin/env bash
PORT_8080_ACTION="DROP"
PORT_80_ACTION="ACCEPT"
PORT_443_ACTION="ACCEPT"

echo "Configuring firewall..."
if [ "$PORT_8080_ACTION" = "DROP" ]; then
    echo "Port 8080 is BLOCKED"
    exit 1
fi
echo "Port 8080 is ALLOWED"
exit 0
EOF

chmod +x /root/firewall/rules.sh
