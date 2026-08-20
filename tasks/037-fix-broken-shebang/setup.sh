#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq python3

mkdir -p /opt/scripts
cat <<'EOF' > /opt/scripts/health.py
#!/bin/python
print("health_ok")
EOF

cat <<'EOF' > /opt/scripts/metrics.py
#!/usr/local/bin/python
print("metrics_ok")
EOF

chmod +x /opt/scripts/*.py
