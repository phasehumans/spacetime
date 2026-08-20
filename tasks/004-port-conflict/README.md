# Resolve Port Conflict (004-port-conflict)

## Description
The agent must find the process listening on port 8080 and kill it.

## Prompt Given to Agent
```text
A rogue process is listening on port 8080, preventing our application from starting. Find the process and terminate it so the port is free."
```

## Base Image
`ubuntu:22.04`
