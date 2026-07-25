#!/usr/bin/env spacetime
# id: task-016
# name: SQLite Query
# description: Query a SQLite database.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Query the SQLite database at /data.db to find the username with id=1, and save the username to /tmp/admin.txt.

# === SETUP ===
apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3
sqlite3 /data.db "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT); INSERT INTO users (username) VALUES ('admin'), ('guest');"

# === VALIDATE ===
grep -q "admin" /tmp/admin.txt
