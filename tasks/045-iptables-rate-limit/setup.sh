#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq iptables
rm -f /usr/local/bin/setup-firewall.sh
