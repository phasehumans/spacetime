#!/usr/bin/env bash
set -e

if [ ! -f /root/truncated.txt ]; then
    echo "/root/truncated.txt missing"
    exit 1
fi

if ! grep -q "/var/log/app/huge.log" /root/truncated.txt || ! grep -q "/var/log/app/sub/nested_huge.log" /root/truncated.txt; then
    echo "Missing expected truncated files in /root/truncated.txt"
    exit 1
fi

SIZE1=$(stat -c%s /var/log/app/huge.log 2>/dev/null || echo "1")
SIZE2=$(stat -c%s /var/log/app/sub/nested_huge.log 2>/dev/null || echo "1")
SIZE_SMALL=$(stat -c%s /var/log/app/small.log 2>/dev/null || echo "0")

if [ "$SIZE1" -ne 0 ] || [ "$SIZE2" -ne 0 ]; then
    echo "Files were not truncated to 0 bytes: size1=$SIZE1, size2=$SIZE2"
    exit 1
fi

if [ "$SIZE_SMALL" -eq 0 ]; then
    echo "Small log file was improperly truncated"
    exit 1
fi

echo "Log cleanup and truncation verified"
exit 0
