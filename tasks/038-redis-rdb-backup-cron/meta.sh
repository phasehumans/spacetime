TASK_ID="038-redis-rdb-backup-cron"
TASK_NAME="Create Redis Snapshot Backup Script"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Write an executable backup script /usr/local/bin/backup-redis.sh to copy dump.rdb to /var/backups/redis/."
