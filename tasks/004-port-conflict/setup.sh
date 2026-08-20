#!/usr/bin/env bash
set -e

apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y netcat-openbsd net-tools psmisc
nc -l 8080 >/dev/null 2>&1 &
