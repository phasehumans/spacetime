#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq tmux
rm -f /usr/local/bin/start-worker.sh
