#!/usr/bin/env bash
set -e

cat <<'EOF' > /usr/local/bin/worker.sh
#!/usr/bin/env bash
PIDFILE="/var/run/worker.pid"
if [ -f "$PIDFILE" ]; then
    OLD_PID=$(cat "$PIDFILE")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo "Worker is already running with PID $OLD_PID"
        exit 1
    fi
fi
echo $$ > "$PIDFILE"
while true; do
    sleep 1
done
EOF
chmod +x /usr/local/bin/worker.sh

/usr/local/bin/worker.sh &
sleep 0.5
