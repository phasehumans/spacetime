#!/usr/bin/env bash
set -e

crontab -l | grep -E -q '^[[:space:]]*0[[:space:]]+2[[:space:]]+\*[[:space:]]+\*[[:space:]]+\*[[:space:]]+(root[[:space:]]+)?/backup\.sh'
