#!/usr/bin/env bash
set -e

FILE="/etc/systemd/system/metric-collector.service"
if [ ! -f "$FILE" ]; then
    echo "Unit file missing"
    exit 1
fi

if ! grep -q "^\[Unit\]" "$FILE" || ! grep -q "^\[Service\]" "$FILE" || ! grep -q "^\[Install\]" "$FILE"; then
    echo "Missing standard systemd section headers"
    exit 1
fi

if ! grep -q "ExecStart=/usr/local/bin/collector.sh" "$FILE"; then
    echo "ExecStart does not point to /usr/local/bin/collector.sh"
    exit 1
fi

if [ ! -x /usr/local/bin/collector.sh ]; then
    echo "/usr/local/bin/collector.sh is not executable"
    exit 1
fi

echo "Systemd service syntax verified"
exit 0
