#!/usr/bin/env spacetime
# id: task-013
# name: Create Symlink
# description: Create a symbolic link.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: Create a symbolic link at /opt/app/current that points to /opt/app/v1.0.

# === SETUP ===
mkdir -p /opt/app/v1.0
echo "v1.0" > /opt/app/v1.0/version

# === VALIDATE ===
test -L /opt/app/current
readlink /opt/app/current | grep -q "v1.0"
