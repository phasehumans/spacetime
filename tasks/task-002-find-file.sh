#!/usr/bin/env spacetime
# id: task-002
# name: Find and Move Hidden Config
# description: The agent needs to find a hidden configuration file and move it to a specific location.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: A secret configuration file named '.secret.cfg' is hidden somewhere inside /var/lib/app. Please find it and move it to /etc/app/config.cfg.

# === SETUP ===
mkdir -p /var/lib/app/data/deeply/nested/structure
echo "secret_key=12345" > /var/lib/app/data/deeply/nested/structure/.secret.cfg
mkdir -p /etc/app/

# === VALIDATE ===
test -f /etc/app/config.cfg
grep -q "secret_key=12345" /etc/app/config.cfg
