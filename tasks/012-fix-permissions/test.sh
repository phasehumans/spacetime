#!/usr/bin/env bash
set -e

test -x /usr/local/bin/runapp
/usr/local/bin/runapp | grep -q "hello"\n
