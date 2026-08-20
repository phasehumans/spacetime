#!/usr/bin/env bash
set -e

apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3
sqlite3 /data.db "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT); INSERT INTO users (username) VALUES ('admin'), ('guest');"
