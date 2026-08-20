#!/usr/bin/env bash
set -e

grep -q "RUN echo hello" /project/Dockerfile
! grep -q "RUNN" /project/Dockerfile\n
