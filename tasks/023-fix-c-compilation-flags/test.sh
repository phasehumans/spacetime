#!/usr/bin/env bash
set -e

cd /root/calc
make clean || true
make
OUTPUT=$(./calculator 16)
echo "$OUTPUT" | grep -q "sqrt(16.00) = 4.00"
echo "C compilation and math/pthread linking verified"
exit 0
