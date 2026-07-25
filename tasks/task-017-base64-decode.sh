#!/usr/bin/env spacetime
# id: task-017
# name: Base64 Decode
# description: Decode a base64 string.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: Decode the base64 string in /tmp/encoded.txt and save the decoded text to /tmp/decoded.txt.

# === SETUP ===
echo "aGVsbG8gd29ybGQ=" > /tmp/encoded.txt

# === VALIDATE ===
grep -q "hello world" /tmp/decoded.txt
