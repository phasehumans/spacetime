#!/usr/bin/env spacetime
# id: task-010
# name: Tar Extraction
# description: Extract a specific file from a tar archive.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: There is an archive at /data.tar.gz. Extract ONLY the file 'secret.txt' from it and place it at /root/secret.txt.

# === SETUP ===
mkdir -p /tmp/archive
echo "secret data" > /tmp/archive/secret.txt
echo "junk" > /tmp/archive/junk.txt
tar -czf /data.tar.gz -C /tmp archive
rm -rf /tmp/archive

# === VALIDATE ===
test -f /root/secret.txt
grep -q "secret data" /root/secret.txt
! test -f /root/junk.txt
