#!/usr/bin/env bash
set -e

mkdir -p /project
echo -e "FROM alpine
RUNN echo hello" > /project/Dockerfile
