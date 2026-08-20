#!/usr/bin/env bash
set -e

if [ ! -f /root/errors.log ]; then
    echo "/root/errors.log missing"
    exit 1
fi

if ! grep -q "AttributeError: 'NoneType' object has no attribute 'value'" /root/errors.log; then
    echo "First traceback missing"
    exit 1
fi

if ! grep -q "ConnectionRefusedError: \[Errno 111\] Connection refused" /root/errors.log; then
    echo "Second traceback missing"
    exit 1
fi

if grep -q "Worker started" /root/errors.log; then
    echo "Normal log lines leaked into /root/errors.log"
    exit 1
fi

echo "Traceback extraction verified"
exit 0
