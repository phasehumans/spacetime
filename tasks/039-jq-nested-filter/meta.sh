TASK_ID="039-jq-nested-filter"
TASK_NAME="Extract Nested JSON Cluster Nodes"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Extract healthy cluster node IP addresses from /var/data/cluster.json into /root/healthy_nodes.json using jq."
