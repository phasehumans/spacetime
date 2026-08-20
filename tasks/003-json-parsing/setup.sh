#!/usr/bin/env bash
set -e

apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y jq
echo '{"users": [{"id": 1, "name": "Alice", "email": "alice@example.com"}, {"id": 2, "name": "Bob", "email": "bob@example.com"}]}' > /tmp/data.json
