#!/usr/bin/env bash
set -e

if [ ! -f /backup/site.tar.gz ]; then
    echo "/backup/site.tar.gz missing"
    exit 1
fi

CONTENTS=$(tar -ztvf /backup/site.tar.gz)

if ! echo "$CONTENTS" | grep -q "index.js"; then
    echo "Source file index.js missing from archive"
    exit 1
fi

if echo "$CONTENTS" | grep -q "node_modules"; then
    echo "node_modules was not excluded from archive"
    exit 1
fi

if echo "$CONTENTS" | grep -q "\.git"; then
    echo ".git was not excluded from archive"
    exit 1
fi

if echo "$CONTENTS" | grep -q "cache\.tmp"; then
    echo ".tmp file was not excluded from archive"
    exit 1
fi

echo "Archive with selective exclusions verified"
exit 0
