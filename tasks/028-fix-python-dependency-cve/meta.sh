TASK_ID="028-fix-python-dependency-cve"
TASK_NAME="Remediate Python Dependency CVEs"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Upgrade vulnerable pinned packages in requirements.txt and create dependency audit summary."
