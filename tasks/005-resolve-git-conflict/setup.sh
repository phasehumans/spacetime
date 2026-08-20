#!/usr/bin/env bash
set -e

which git >/dev/null 2>&1 || (apt-get update && apt-get install -y git)
git init /repo
cd /repo
git config user.email "test@example.com"
git config user.name "Test User"
echo "line 1" > file.txt
git add file.txt
git commit -m "initial"
git checkout -b feature
echo "line 1 modified by feature" > file.txt
git commit -am "feature update"
git checkout master
echo "line 1 modified by master" > file.txt
git commit -am "master update"
git merge feature || true
