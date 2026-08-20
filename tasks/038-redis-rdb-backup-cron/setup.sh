#!/usr/bin/env bash
set -e

mkdir -p /var/lib/redis
echo "REDIS0009_DUMMY_DATA_PAYLOAD_FOR_TESTING" > /var/lib/redis/dump.rdb
rm -rf /var/backups/redis /usr/local/bin/backup-redis.sh
