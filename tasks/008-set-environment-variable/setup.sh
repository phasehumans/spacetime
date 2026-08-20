#!/usr/bin/env bash
set -e

mkdir -p /app
echo '#!/bin/sh' > /app/run.sh
echo 'echo "The secret is ${SECRET_TOKEN}"' >> /app/run.sh
chmod +x /app/run.sh
