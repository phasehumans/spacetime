#pragma once

#include <atomic>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <optional>
#include <string>
#include <string_view>

#include <unistd.h>

// A unique scratch directory that is deleted when it goes out of scope.
class TempDir {
public:
    TempDir() {
        static std::atomic<int> counter{0};
        path_ = std::filesystem::temp_directory_path() /
                ("spacetime-test-" + std::to_string(::getpid()) + "-" + std::to_string(counter++));
        std::filesystem::create_directories(path_);
    }
    ~TempDir() {
        std::error_code ec;
        std::filesystem::remove_all(path_, ec);
    }
    TempDir(const TempDir&) = delete;
    TempDir& operator=(const TempDir&) = delete;

    std::string path() const { return path_.string(); }

    // Writes `content` to a file relative to the directory, creating parents.
    void write(const std::string& relative, std::string_view content) const {
        const std::filesystem::path file = path_ / relative;
        std::filesystem::create_directories(file.parent_path());
        std::ofstream(file, std::ios::binary) << content;
    }

private:
    std::filesystem::path path_;
};

// Sets (or unsets) an environment variable and restores it on scope exit.
class ScopedEnv {
public:
    ScopedEnv(std::string key, const char* value) : key_(std::move(key)) {
        if (const char* old = std::getenv(key_.c_str())) previous_ = old;
        if (value) {
            ::setenv(key_.c_str(), value, 1);
        } else {
            ::unsetenv(key_.c_str());
        }
    }
    ~ScopedEnv() {
        if (previous_) {
            ::setenv(key_.c_str(), previous_->c_str(), 1);
        } else {
            ::unsetenv(key_.c_str());
        }
    }
    ScopedEnv(const ScopedEnv&) = delete;
    ScopedEnv& operator=(const ScopedEnv&) = delete;

private:
    std::string key_;
    std::optional<std::string> previous_;
};
