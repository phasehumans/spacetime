#!/usr/bin/env bash
set -e

test -f /etc/app/config.cfg
grep -q "secret_key=12345" /etc/app/config.cfg
