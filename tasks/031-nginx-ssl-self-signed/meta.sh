TASK_ID="031-nginx-ssl-self-signed"
TASK_NAME="Configure Nginx Self-Signed SSL"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Generate a self-signed SSL cert at /etc/ssl/certs/server.crt and configure Nginx HTTPS on port 443."
