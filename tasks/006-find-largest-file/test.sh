#!/usr/bin/env bash
set -e

! test -f /data/nested/dirs/file3.bin
test -f /data/file1.bin
test -f /data/nested/file2.bin
