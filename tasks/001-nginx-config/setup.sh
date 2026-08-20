#!/usr/bin/env bash
set -e

apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y nginx curl
mkdir -p /var/www/html
echo "<h1>Welcome to nginx!</h1>" > /var/www/html/index.html
echo "server { listen 80; root /var/www/html; index index.html; error_page 404 /404.html; " > /etc/nginx/sites-available/default
