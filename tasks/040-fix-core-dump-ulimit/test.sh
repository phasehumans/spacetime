#!/usr/bin/env bash
set -e

CONF="/etc/security/limits.d/coredump.conf"
if [ ! -f "$CONF" ]; then
    echo "$CONF missing"
    exit 1
fi

if ! grep -qE "^\s*\*\s+(soft|hard|-)\s+core\s+unlimited" "$CONF"; then
    echo "Limits configuration does not properly configure * core unlimited"
    exit 1
fi

echo "Core dump limit configuration verified"
exit 0
