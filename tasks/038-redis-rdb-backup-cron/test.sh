#!/usr/bin/env bash
set -e

if [ ! -x /usr/local/bin/backup-redis.sh ]; then
    echo "Backup script missing or not executable"
    exit 1
fi

BACKUP="/var/backups/redis/dump_backup.rdb.gz"
if [ ! -f "$BACKUP" ]; then
    echo "Backup file missing at $BACKUP"
    exit 1
fi

PERMS=$(stat -c "%a" "$BACKUP" 2>/dev/null || echo "000")
if [ "$PERMS" != "600" ]; then
    echo "Permissions not 600: got $PERMS"
    exit 1
fi

# Verify gzip decompression
gzip -t "$BACKUP"

echo "Redis backup script verified"
exit 0
