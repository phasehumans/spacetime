#!/usr/bin/env spacetime
# id: task-005
# name: Resolve Git Conflict
# description: Resolve a simulated git merge conflict.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: There is a git repository at /repo with a merge conflict in file.txt. Resolve the conflict by keeping the 'master' version, and complete the merge commit.

# === SETUP ===
apk add --no-cache git
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

# === VALIDATE ===
cd /repo
grep -q "line 1 modified by master" file.txt
! grep -q "<<<<<<<" file.txt
git status | grep -q "nothing to commit"
