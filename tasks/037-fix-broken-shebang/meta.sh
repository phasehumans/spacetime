TASK_ID="037-fix-broken-shebang"
TASK_NAME="Fix Broken Script Shebang Lines"
BASE_IMAGE="ubuntu:22.04"
MAX_TURNS=15
TIMEOUT_SECS=60
DESCRIPTION="Fix obsolete /usr/bin/python shebangs in /opt/scripts to #!/usr/bin/env python3 and ensure scripts run."
