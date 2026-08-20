#!/usr/bin/env bash
set -e

test -f /tmp/bob_email.txt
grep -q "bob@example.com" /tmp/bob_email.txt
