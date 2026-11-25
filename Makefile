# Thin wrapper around the CMake build (see CMakeLists.txt).

CMAKE      ?= cmake
BUILD_DIR  ?= build
BUILD_TYPE ?= Release
PREFIX     ?= /usr/local
CMAKE_ARGS ?=

TARGET = spacetime

# Keep the nested CMake-driven make quiet.
MAKEFLAGS += --no-print-directory

.PHONY: all configure debug test test-docker clean install

all: configure
	$(CMAKE) --build $(BUILD_DIR) --target $(TARGET) --parallel
	@cp $(BUILD_DIR)/$(TARGET) $(TARGET)
	@echo "Built $(TARGET) successfully!"

configure:
	@command -v $(CMAKE) >/dev/null 2>&1 || { \
		echo "error: cmake not found. Install it first (e.g. 'sudo apt install cmake' or 'brew install cmake')." >&2; \
		exit 1; }
	$(CMAKE) -S . -B $(BUILD_DIR) -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) $(CMAKE_ARGS)

debug: BUILD_TYPE = Debug
debug: all

test: all
	$(CMAKE) --build $(BUILD_DIR) --target spacetime_tests --parallel
	$(BUILD_DIR)/spacetime_tests
	@echo "All unit tests passed!"

# Integration test against a live Docker daemon (needs the ubuntu:24.04 image).
test-docker: test
	$(BUILD_DIR)/spacetime_tests "[docker]"

clean:
	rm -rf $(BUILD_DIR) $(TARGET)

install: all
	$(CMAKE) --install $(BUILD_DIR) --prefix $(PREFIX)
