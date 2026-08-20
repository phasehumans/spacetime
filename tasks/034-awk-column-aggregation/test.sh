#!/usr/bin/env bash
set -e

if [ ! -f /root/total_bandwidth.txt ]; then
    echo "/root/total_bandwidth.txt missing"
    exit 1
fi

VAL=$(tr -d '[:space:]' < /root/total_bandwidth.txt)
if [ "$VAL" != "15360" ]; then
    echo "Incorrect bandwidth calculation: expected 15360, got '$VAL'"
    exit 1
fi

echo "Bandwidth aggregation verified"
exit 0
