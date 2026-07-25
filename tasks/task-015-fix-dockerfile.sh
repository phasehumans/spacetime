#!/usr/bin/env spacetime
# id: task-015
# name: Fix Dockerfile
# description: Fix a simulated Dockerfile syntax error.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: The Dockerfile at /project/Dockerfile has a typo causing builds to fail. Fix the typo.

# === SETUP ===
mkdir -p /project
echo "FROM alpine" > /project/Dockerfile
echo "RUNN echo hello" >> /project/Dockerfile

# === VALIDATE ===
grep -q "RUN echo hello" /project/Dockerfile
! grep -q "RUNN" /project/Dockerfile
