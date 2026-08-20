#!/usr/bin/env bash
set -e

grep -q "db_host=db.internal" /configs/db1.conf
grep -q "db_host=db.internal" /configs/db2.conf
grep -q "other=localhost" /configs/other.conf\n
