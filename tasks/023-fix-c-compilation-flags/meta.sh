TASK_ID="023-fix-c-compilation-flags"
TASK_NAME="Fix C Makefile Linker Flags"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Fix missing linker flags (-pthread, -lm) and build options in C Makefile to compile calculator."
