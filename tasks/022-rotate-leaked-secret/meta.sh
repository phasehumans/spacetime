TASK_ID="022-rotate-leaked-secret"
TASK_NAME="Rotate Leaked Secret & Fix Perms"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Rotate compromised secret token, restrict file permissions to 0600, and update .env.example."
