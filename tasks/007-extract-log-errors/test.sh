#!/usr/bin/env bash
set -e

test -f /tmp/errors.log
wc -l < /tmp/errors.log | grep -q "2"
grep -q "Connection failed" /tmp/errors.log
grep -q "Timeout" /tmp/errors.log
! grep -q "INFO" /tmp/errors.log
