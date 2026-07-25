#!/usr/bin/env spacetime
# id: task-008
# name: Set Environment Variable
# description: Modify a script to use an environment variable.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: Modify the system so that when /app/run.sh is executed, it outputs 'The secret is xyz123'. You can modify the script itself or its environment.

# === SETUP ===
mkdir -p /app
echo '#!/bin/sh' > /app/run.sh
echo 'echo "The secret is ${SECRET_TOKEN}"' >> /app/run.sh
chmod +x /app/run.sh

# === VALIDATE ===
/app/run.sh | grep -q "xyz123"
