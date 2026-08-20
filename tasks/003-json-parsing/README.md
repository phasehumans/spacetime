# Extract User Email (003-json-parsing)

## Description
The agent needs to parse a JSON file and extract a specific value into a new file.

## Prompt Given to Agent
```text
There is a JSON file at /tmp/data.json containing user data. Extract the email address of the user named 'Bob' and save it to a new file at /tmp/bob_email.txt. Only the email address should be in the file."
```

## Base Image
`ubuntu:22.04`
