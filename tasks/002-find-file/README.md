# Find and Move Hidden Config (002-find-file)

## Description
The agent needs to find a hidden configuration file and move it to a specific location.

## Prompt Given to Agent
```text
A secret configuration file named '.secret.cfg' is hidden somewhere inside /var/lib/app. Please find it and move it to /etc/app/config.cfg."
```

## Base Image
`alpine:latest`
