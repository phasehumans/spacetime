#!/usr/bin/env spacetime
# id: task-020
# name: Cron Job
# description: Create a simple cron job.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Add a cron job for the root user that runs the command '/backup.sh' every day at 2:00 AM.

# === SETUP ===
apt-get update && apt-get install -y cron

# === VALIDATE ===
crontab -l | grep -q "0 2 \* \* \* /backup.sh" || crontab -l | grep -q "0 2 \* \* \* root /backup.sh"
