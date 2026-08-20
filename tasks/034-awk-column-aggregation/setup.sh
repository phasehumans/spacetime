#!/usr/bin/env bash
set -e

cat <<'EOF' > /var/log/traffic.csv
timestamp,ip,status_code,bytes_sent,endpoint
2026-08-20T10:00:00Z,192.168.1.10,200,1024,/api/v1/users
2026-08-20T10:00:01Z,192.168.1.11,404,256,/api/v1/missing
2026-08-20T10:00:02Z,192.168.1.12,200,2048,/api/v1/items
2026-08-20T10:00:03Z,192.168.1.13,500,512,/api/v1/error
2026-08-20T10:00:04Z,192.168.1.14,200,4096,/api/v1/checkout
2026-08-20T10:00:05Z,192.168.1.15,301,128,/redirect
2026-08-20T10:00:06Z,192.168.1.16,200,8192,/api/v1/download
EOF
