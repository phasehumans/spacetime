#!/usr/bin/env bash
set -e

cd /repo
grep -q "line 1 modified by master" file.txt
! grep -q "<<<<<<<" file.txt
git status | grep "nothing to commit"
