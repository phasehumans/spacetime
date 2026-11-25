#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>

namespace spacetime {

// Incremental SHA-256 (FIPS 180-4).
class Sha256 {
public:
    using Digest = std::array<std::uint8_t, 32>;

    Sha256();

    void update(const std::uint8_t* data, std::size_t len);
    void update(std::string_view data);

    // Finalizes the hash. The object must not be updated afterwards.
    Digest finish();

    // Finalizes the hash and returns it as 64 lowercase hex characters.
    std::string hex_digest();

private:
    void transform(const std::uint8_t* block);

    std::array<std::uint32_t, 8> state_;
    std::array<std::uint8_t, 64> buffer_{};
    std::uint64_t count_ = 0;
};

std::string sha256_hex(std::string_view data);

// Returns nullopt if the file cannot be opened.
std::optional<std::string> sha256_file(const std::string& path);

} // namespace spacetime
