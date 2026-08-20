#!/usr/bin/env bash
set -e

mkdir -p /etc/systemd/system
cat <<'EOF' > /etc/systemd/system/metric-collector.service
Description=System Metric Collector
ExecStart=collector.sh
Restart=on-failure
WantedBy=default.target
EOF

cat <<'EOF' > /usr/local/bin/collector.sh
#!/usr/bin/env bash
echo "metric collection active"
EOF
chmod -x /usr/local/bin/collector.sh
