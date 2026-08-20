#!/usr/bin/env bash
set -e

DB="/root/ecommerce/store.db"
if [ ! -f "$DB" ]; then
    echo "Database missing"
    exit 1
fi

IDX1=$(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_users_email';")
IDX2=$(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_orders_status';")

if [ "$IDX1" != "idx_users_email" ]; then
    echo "Index idx_users_email not found in sqlite_master"
    exit 1
fi

if [ "$IDX2" != "idx_orders_status" ]; then
    echo "Index idx_orders_status not found in sqlite_master"
    exit 1
fi

echo "SQLite indexes verified"
exit 0
