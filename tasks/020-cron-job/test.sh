#!/usr/bin/env bash
set -e

crontab -l | grep -q "0 2 \* \* \* /backup.sh" || crontab -l | grep -q "0 2 \* \* \* root /backup.sh"\n
