#!/usr/bin/env bash
set -e

mkdir -p /tmp/archive
echo "secret data" > /tmp/archive/secret.txt
echo "junk" > /tmp/archive/junk.txt
tar -czf /data.tar.gz -C /tmp archive
rm -rf /tmp/archive
