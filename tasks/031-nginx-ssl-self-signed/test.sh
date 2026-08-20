#!/usr/bin/env bash
set -e

if [ ! -f /etc/ssl/certs/server.crt ] || [ ! -f /etc/ssl/private/server.key ]; then
    echo "SSL certificate or key missing"
    exit 1
fi

RES=$(curl -k -s https://localhost:443/ || true)
if [[ "$RES" != *"Spacetime HTTPS OK"* ]]; then
    echo "Nginx HTTPS request failed: $RES"
    exit 1
fi

echo "Nginx HTTPS configuration verified"
exit 0
