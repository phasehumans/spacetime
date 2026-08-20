#!/usr/bin/env bash
set -e

mkdir -p /var/lib/app/data/deeply/nested/structure
echo "secret_key=12345" > /var/lib/app/data/deeply/nested/structure/.secret.cfg
mkdir -p /etc/app/
