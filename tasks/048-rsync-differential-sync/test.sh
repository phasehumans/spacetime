#!/usr/bin/env bash
set -e

if [ ! -f /srv/backup/file1.txt ] || [ ! -f /srv/backup/file2.txt ]; then
    echo "Files missing from backup destination"
    exit 1
fi

if [ -f /srv/backup/old_file.txt ]; then
    echo "Obsolete file was not deleted during rsync sync"
    exit 1
fi

DIFF=$(diff -r /srv/data /srv/backup || true)
if [ -n "$DIFF" ]; then
    echo "Directories differ: $DIFF"
    exit 1
fi

echo "Rsync differential synchronization verified"
exit 0
