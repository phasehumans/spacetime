#!/usr/bin/env bash
set -e

mkdir -p /root/calc
cat <<'EOF' > /root/calc/main.c
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <pthread.h>

void* worker(void* arg) {
    double* val = (double*)arg;
    *val = sqrt(*val);
    return NULL;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("Usage: %s <number>\n", argv[0]);
        return 1;
    }
    double input = atof(argv[1]);
    double result = input;
    pthread_t tid;
    pthread_create(&tid, NULL, worker, &result);
    pthread_join(tid, NULL);
    printf("sqrt(%.2f) = %.2f\n", input, result);
    return 0;
}
EOF

cat <<'EOF' > /root/calc/Makefile
CC = gcc
CFLAGS = -Wall
TARGET = calculator

all: $(TARGET)

$(TARGET): main.c
	$(CC) $(CFLAGS) -o $(TARGET) main.c

clean:
	rm -f $(TARGET)
EOF
