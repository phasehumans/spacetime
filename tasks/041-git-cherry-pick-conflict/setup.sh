#!/usr/bin/env bash
set -e

rm -rf /root/project
mkdir -p /root/project
cd /root/project

git init -b main
git config user.name "Spacetime"
git config user.email "test@spacetime.benchmark"
git config core.editor "true"

echo -e "VERSION = 1\nTIMEOUT = 30" > config.py
git add config.py
git commit -m "initial commit"

# Create feature branch
git checkout -b feature
echo -e "VERSION = 1\nTIMEOUT = 30\nENABLE_METRICS = True" > config.py
git commit -am "feature: add metrics"
FEATURE_COMMIT=$(git rev-parse HEAD)

# Update main
git checkout main
echo -e "VERSION = 2\nTIMEOUT = 60" > config.py
git commit -am "bump version and timeout"

# Start cherry-pick that conflicts
git cherry-pick "$FEATURE_COMMIT" || true
