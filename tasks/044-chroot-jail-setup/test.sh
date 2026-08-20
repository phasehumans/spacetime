#!/usr/bin/env bash
set -e

if [ ! -f /jail/bin/sh ]; then
    echo "/jail/bin/sh missing"
    exit 1
fi

# Run command inside chroot
OUTPUT=$(chroot /jail /bin/sh -c "echo 'chroot_jail_ok'" 2>&1)
if [ "$OUTPUT" != "chroot_jail_ok" ]; then
    echo "Chroot execution failed: $OUTPUT"
    exit 1
fi

echo "Minimal chroot jail verified"
exit 0
