#!/usr/bin/env bash
set -e

mkdir -p /data/nested/dirs
dd if=/dev/urandom of=/data/file1.bin bs=1M count=1
dd if=/dev/urandom of=/data/nested/file2.bin bs=1M count=2
dd if=/dev/urandom of=/data/nested/dirs/file3.bin bs=1M count=5
