TASK_ID="043-curl-jwt-auth-header"
TASK_NAME="Fetch Bearer Token and Query API"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="POST credentials to localhost:8000/auth, extract token, and query GET localhost:8000/data saving JSON to /root/data.json."
