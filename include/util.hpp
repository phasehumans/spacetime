#pragma once

#include <cstdio>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace spacetime {

// Reads a whole file as bytes. Returns nullopt if it cannot be opened.
std::optional<std::string> read_file(const std::string& path);

// Strips leading and trailing ASCII whitespace.
std::string_view trim(std::string_view s);

// ASCII case-insensitive comparison and substring search.
bool iequals(std::string_view a, std::string_view b);
bool icontains(std::string_view haystack, std::string_view needle);

// printf-style helpers on top of std::format. They write through C stdio so
// they interleave correctly with the raw agent output streamed by the Docker
// client.
template <typename... Args>
void print(std::format_string<Args...> fmt, Args&&... args) {
    const std::string text = std::format(fmt, std::forward<Args>(args)...);
    std::fwrite(text.data(), 1, text.size(), stdout);
}

template <typename... Args>
void eprint(std::format_string<Args...> fmt, Args&&... args) {
    const std::string text = std::format(fmt, std::forward<Args>(args)...);
    std::fwrite(text.data(), 1, text.size(), stderr);
}

} // namespace spacetime
