#!/usr/bin/env bash
set -e

mkdir -p /var/log/app/sub
# Create files > 50MB
truncate -s 60M /var/log/app/huge.log
truncate -s 75M /var/log/app/sub/nested_huge.log

# Create files < 50MB
truncate -s 10M /var/log/app/small.log
truncate -s 2M /var/log/app/app.txt
