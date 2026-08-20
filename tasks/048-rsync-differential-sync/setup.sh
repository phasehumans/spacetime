#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq rsync

mkdir -p /srv/data /srv/backup
echo "data 1" > /srv/data/file1.txt
echo "data 2" > /srv/data/file2.txt

echo "old data" > /srv/backup/file1.txt
echo "obsolete data" > /srv/backup/old_file.txt
