TASK_ID="029-deadlock-process-kill"
TASK_NAME="Recover Deadlocked Worker Process"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Inspect stale PID lockfile, terminate deadlocked process, and restart background worker."
