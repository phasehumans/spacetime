#!/usr/bin/env bash
set -e

nginx -t
service nginx start
curl -s http://localhost | grep "Welcome to nginx!"
