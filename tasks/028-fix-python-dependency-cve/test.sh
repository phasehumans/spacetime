#!/usr/bin/env bash
set -e

REQ="/root/apiserver/requirements.txt"
AUDIT="/root/apiserver/AUDIT.md"

if grep -q "2.18.4" "$REQ" || grep -q "1.22" "$REQ" || grep -q "0.12.2" "$REQ"; then
    echo "Outdated vulnerable package versions still pinned in requirements.txt"
    exit 1
fi

if ! grep -q "requests" "$REQ" || ! grep -q "urllib3" "$REQ" || ! grep -q "flask" "$REQ"; then
    echo "Required packages missing from requirements.txt"
    exit 1
fi

if [ ! -s "$AUDIT" ]; then
    echo "AUDIT.md is missing or empty"
    exit 1
fi

echo "Python dependency CVE remediation verified"
exit 0
