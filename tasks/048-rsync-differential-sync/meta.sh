TASK_ID="048-rsync-differential-sync"
TASK_NAME="Perform Differential Rsync With Delete"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Synchronize directory /srv/data/ to /srv/backup/ using rsync with deletion of obsolete files."
