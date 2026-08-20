#!/usr/bin/env bash
set -e

cat <<'EOF' > /var/log/application.log
2026-08-20 10:00:01 [INFO] Worker started
2026-08-20 10:00:02 [INFO] Processing batch 1
2026-08-20 10:00:03 [ERROR] Unhandled exception occurred:
Traceback (most recent call last):
  File "app.py", line 42, in run
    result = compute_data(None)
  File "calc.py", line 12, in compute_data
    return data.value * 2
AttributeError: 'NoneType' object has no attribute 'value'
2026-08-20 10:00:04 [INFO] Retrying job
2026-08-20 10:00:05 [ERROR] Connection timed out:
Traceback (most recent call last):
  File "client.py", line 88, in connect
    sock.connect(("10.0.0.1", 5432))
ConnectionRefusedError: [Errno 111] Connection refused
2026-08-20 10:00:06 [INFO] Worker shutting down
EOF
rm -f /root/errors.log
