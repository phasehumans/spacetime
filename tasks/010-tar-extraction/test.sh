#!/usr/bin/env bash
set -e

test -f /root/secret.txt
grep -q "secret data" /root/secret.txt
! test -f /root/junk.txt
