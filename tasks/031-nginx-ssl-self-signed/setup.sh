#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq nginx openssl curl

mkdir -p /etc/ssl/certs /etc/ssl/private
mkdir -p /var/www/html
echo "<h1>Spacetime HTTPS OK</h1>" > /var/www/html/index.html
