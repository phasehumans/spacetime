#!/usr/bin/env bash
set -e

test -L /opt/app/current
readlink /opt/app/current | grep -q "v1.0"\n
