#!/usr/bin/env bash
set -e

mkdir -p /etc/security/limits.d
rm -f /etc/security/limits.d/coredump.conf
