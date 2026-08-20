TASK_ID="033-disk-space-cleanup"
TASK_NAME="Find and Truncate Large Logs"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Find all .log files in /var/log/app larger than 50MB, truncate them, and write removed filenames to /root/truncated.txt."
