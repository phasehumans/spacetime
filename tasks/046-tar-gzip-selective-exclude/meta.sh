TASK_ID="046-tar-gzip-selective-exclude"
TASK_NAME="Archive Web App With Exclusions"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Create /backup/site.tar.gz archiving /var/www/site while excluding node_modules, .git, and *.tmp files."
