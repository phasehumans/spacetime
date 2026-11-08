CC ?= gcc
CFLAGS ?= -std=c11 -Wall -Wextra -pedantic -Iinclude -Ivendor -O2
LDFLAGS ?= -lm

SRCS = src/main.c \
       src/agent.c \
       src/docker.c \
       src/eval.c \
       src/report.c \
       src/sha256.c \
       src/task.c \
       src/taskset.c \
       src/tui.c \
       vendor/yyjson/yyjson.c

OBJS = $(SRCS:.c=.o)
TARGET = spacetime

.PHONY: all clean test debug install

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(OBJS) -o $(TARGET) $(LDFLAGS)
	@echo "Built $(TARGET) successfully!"

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

debug: CFLAGS = -std=c11 -Wall -Wextra -pedantic -Iinclude -Ivendor -g -DDEBUG -O0
debug: clean $(TARGET)

test: all
	$(CC) $(CFLAGS) test/test_taskset.c src/task.c src/taskset.c src/sha256.c -o test/test_taskset $(LDFLAGS)
	./test/test_taskset
	@echo "All unit tests passed!"

clean:
	rm -f $(OBJS) $(TARGET) test/test_taskset test/test_docker

install: $(TARGET)
	install -d $(DESTDIR)/usr/local/bin
	install -m 755 $(TARGET) $(DESTDIR)/usr/local/bin/$(TARGET)
