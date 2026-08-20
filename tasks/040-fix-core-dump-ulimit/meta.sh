TASK_ID="040-fix-core-dump-ulimit"
TASK_NAME="Enable Core Dump Limits in Limits Conf"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Configure /etc/security/limits.d/coredump.conf to set soft and hard core limits to unlimited for all users."
