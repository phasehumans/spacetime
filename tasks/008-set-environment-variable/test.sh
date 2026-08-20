#!/usr/bin/env bash
set -e

/app/run.sh | grep -q "xyz123"
