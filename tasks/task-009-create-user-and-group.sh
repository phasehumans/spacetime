#!/usr/bin/env spacetime
# id: task-009
# name: Create User and Group
# description: Create a user and assign to a group.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Create a new user named 'alice' and add her to the 'developers' group.

# === SETUP ===
groupadd developers

# === VALIDATE ===
id alice | grep -q "developers"
