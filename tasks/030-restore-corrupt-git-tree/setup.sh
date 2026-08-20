#!/usr/bin/env bash
set -e

rm -rf /root/repo
mkdir -p /root/repo
cd /root/repo
git init -b main
git config user.name "Spacetime"
git config user.email "test@spacetime.benchmark"

echo "line 1" > file.txt
git add file.txt
git commit -m "initial commit"

git checkout -b hotfix
echo "hotfix patch" >> file.txt
git commit -am "applied hotfix"

git checkout main
# Corrupt git index
> .git/index
