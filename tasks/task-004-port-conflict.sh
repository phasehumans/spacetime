#!/usr/bin/env spacetime
# id: task-004
# name: Resolve Port Conflict
# description: The agent must find the process listening on port 8080 and kill it.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: A rogue process is listening on port 8080, preventing our application from starting. Find the process and terminate it so the port is free.

# === SETUP ===
apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y netcat-openbsd net-tools psmisc
nc -l 8080 >/dev/null 2>&1 &

# === VALIDATE ===
! nc -z localhost 8080
