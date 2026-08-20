TASK_ID="034-awk-column-aggregation"
TASK_NAME="Aggregate Access Log Bandwidth"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Parse CSV access log at /var/log/traffic.csv, calculate total bytes for status 200, and write sum to /root/total_bandwidth.txt."
