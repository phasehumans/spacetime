#!/usr/bin/env spacetime
# id: task-018
# name: Find Most Frequent IP
# description: Find the most frequent IP in an access log.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Find the IP address that appears most frequently in /var/log/access.log and write just that IP to /tmp/top_ip.txt.

# === SETUP ===
mkdir -p /var/log
echo "192.168.1.1" > /var/log/access.log
echo "10.0.0.1" >> /var/log/access.log
echo "192.168.1.1" >> /var/log/access.log
echo "192.168.1.1" >> /var/log/access.log
echo "10.0.0.1" >> /var/log/access.log

# === VALIDATE ===
grep -q "192.168.1.1" /tmp/top_ip.txt
