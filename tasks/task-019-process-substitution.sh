#!/usr/bin/env spacetime
# id: task-019
# name: Process Substitution
# description: Diff two command outputs.
# base_image: alpine:latest
# max_turns: 15
# timeout: 300
# prompt: Find the lines that are in /list1.txt but NOT in /list2.txt, and save the result to /diff.txt.

# === SETUP ===
echo "apple" > /list1.txt
echo "banana" >> /list1.txt
echo "cherry" >> /list1.txt
echo "apple" > /list2.txt
echo "cherry" >> /list2.txt

# === VALIDATE ===
grep -q "banana" /diff.txt
! grep -q "apple" /diff.txt
