#!/usr/bin/env spacetime
# id: task-011
# name: Regex Replace
# description: Use sed to replace a string across multiple files.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Change 'db_host=localhost' to 'db_host=db.internal' in all .conf files in the /configs directory.

# === SETUP ===
mkdir -p /configs
echo "db_host=localhost" > /configs/db1.conf
echo "db_host=localhost" > /configs/db2.conf
echo "other=localhost" > /configs/other.conf

# === VALIDATE ===
grep -q "db_host=db.internal" /configs/db1.conf
grep -q "db_host=db.internal" /configs/db2.conf
grep -q "other=localhost" /configs/other.conf
