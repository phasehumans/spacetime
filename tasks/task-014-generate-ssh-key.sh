#!/usr/bin/env spacetime
# id: task-014
# name: Generate SSH Key
# description: Generate an SSH keypair.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Generate an RSA SSH keypair at /root/.ssh/id_rsa with no passphrase.

# === SETUP ===
apt-get update && apt-get install -y openssh-client

# === VALIDATE ===
test -f /root/.ssh/id_rsa
test -f /root/.ssh/id_rsa.pub
