TASK_ID="044-chroot-jail-setup"
TASK_NAME="Set Up Minimal Chroot Jail"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Create minimal chroot jail at /jail with /bin/sh and required shared libraries so chroot /jail /bin/sh works."
