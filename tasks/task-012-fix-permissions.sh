#!/usr/bin/env spacetime
# id: task-012
# name: Fix Permissions
# description: Fix file permissions so a script can run.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: The script at /usr/local/bin/runapp is throwing a permission denied error. Fix its permissions so it is executable by anyone.

# === SETUP ===
mkdir -p /usr/local/bin
echo "echo hello" > /usr/local/bin/runapp
chmod 000 /usr/local/bin/runapp

# === VALIDATE ===
test -x /usr/local/bin/runapp
/usr/local/bin/runapp | grep -q "hello"
