TASK_ID="050-git-bisect-regression"
TASK_NAME="Identify Regressing Commit with Git Bisect"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Use git bisect in /root/calculator to locate the commit that broke the test suite and save the commit SHA to /root/bad_commit.txt."
