#!/usr/bin/env bash
set -e

CONF="/etc/resolv.conf"
if ! grep -q "8.8.8.8" "$CONF" && ! grep -q "1.1.1.1" "$CONF"; then
    echo "Missing required nameservers in $CONF"
    exit 1
fi

if grep -q "127.0.0.99" "$CONF"; then
    echo "Broken loopback nameserver still present in $CONF"
    exit 1
fi

echo "DNS resolv.conf verified"
exit 0
