#!/usr/bin/env spacetime
# id: task-007
# name: Extract Log Errors
# description: Extract lines with ERROR from a log file.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Extract all lines containing the word 'ERROR' from /var/log/app.log and save them to /tmp/errors.log.

# === SETUP ===
echo "INFO: Starting up" > /var/log/app.log
echo "ERROR: Connection failed" >> /var/log/app.log
echo "INFO: Retrying" >> /var/log/app.log
echo "ERROR: Timeout" >> /var/log/app.log
echo "WARN: High memory" >> /var/log/app.log

# === VALIDATE ===
test -f /tmp/errors.log
wc -l < /tmp/errors.log | grep -q "2"
grep -q "Connection failed" /tmp/errors.log
grep -q "Timeout" /tmp/errors.log
! grep -q "INFO" /tmp/errors.log
