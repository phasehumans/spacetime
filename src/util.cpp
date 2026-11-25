#include "util.hpp"

#include <algorithm>
#include <cctype>
#include <fstream>
#include <sstream>

namespace spacetime {

namespace {

bool is_space(char c) { return std::isspace(static_cast<unsigned char>(c)) != 0; }

char lower(char c) { return static_cast<char>(std::tolower(static_cast<unsigned char>(c))); }

bool char_iequals(char a, char b) { return lower(a) == lower(b); }

} // namespace

std::optional<std::string> read_file(const std::string& path) {
    std::ifstream in(path, std::ios::binary);
    if (!in) return std::nullopt;
    std::ostringstream contents;
    contents << in.rdbuf();
    return std::move(contents).str();
}

std::string_view trim(std::string_view s) {
    while (!s.empty() && is_space(s.front())) s.remove_prefix(1);
    while (!s.empty() && is_space(s.back())) s.remove_suffix(1);
    return s;
}

bool iequals(std::string_view a, std::string_view b) {
    return std::equal(a.begin(), a.end(), b.begin(), b.end(), char_iequals);
}

bool icontains(std::string_view haystack, std::string_view needle) {
    return std::search(haystack.begin(), haystack.end(), needle.begin(), needle.end(),
                       char_iequals) != haystack.end() ||
           needle.empty();
}

} // namespace spacetime
