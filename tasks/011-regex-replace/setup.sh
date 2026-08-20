#!/usr/bin/env bash
set -e

mkdir -p /configs
echo "db_host=localhost" > /configs/db1.conf
echo "db_host=localhost" > /configs/db2.conf
echo "other=localhost" > /configs/other.conf
