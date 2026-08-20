#!/usr/bin/env bash
set -e

mkdir -p /var/www/site/src
mkdir -p /var/www/site/.git/objects
mkdir -p /var/www/site/node_modules/express
mkdir -p /backup

echo "index code" > /var/www/site/src/index.js
echo "git data" > /var/www/site/.git/config
echo "package data" > /var/www/site/node_modules/express/index.js
echo "temp data" > /var/www/site/cache.tmp

rm -f /backup/site.tar.gz
