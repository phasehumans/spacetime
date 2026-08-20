#!/usr/bin/env bash
set -e

grep -q "banana" /diff.txt
! grep -q "apple" /diff.txt
