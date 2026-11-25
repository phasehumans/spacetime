#include "sha256.hpp"

#include <algorithm>
#include <bit>
#include <fstream>

namespace spacetime {

namespace {

constexpr std::array<std::uint32_t, 64> K = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
};

constexpr std::uint32_t ch(std::uint32_t x, std::uint32_t y, std::uint32_t z) {
    return (x & y) ^ (~x & z);
}
constexpr std::uint32_t maj(std::uint32_t x, std::uint32_t y, std::uint32_t z) {
    return (x & y) ^ (x & z) ^ (y & z);
}
constexpr std::uint32_t ep0(std::uint32_t x) {
    return std::rotr(x, 2) ^ std::rotr(x, 13) ^ std::rotr(x, 22);
}
constexpr std::uint32_t ep1(std::uint32_t x) {
    return std::rotr(x, 6) ^ std::rotr(x, 11) ^ std::rotr(x, 25);
}
constexpr std::uint32_t sig0(std::uint32_t x) {
    return std::rotr(x, 7) ^ std::rotr(x, 18) ^ (x >> 3);
}
constexpr std::uint32_t sig1(std::uint32_t x) {
    return std::rotr(x, 17) ^ std::rotr(x, 19) ^ (x >> 10);
}

std::string to_hex(const Sha256::Digest& digest) {
    static constexpr char kHex[] = "0123456789abcdef";
    std::string out;
    out.reserve(digest.size() * 2);
    for (std::uint8_t byte : digest) {
        out.push_back(kHex[byte >> 4]);
        out.push_back(kHex[byte & 0x0f]);
    }
    return out;
}

} // namespace

Sha256::Sha256()
    : state_{0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
             0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19} {}

void Sha256::transform(const std::uint8_t* block) {
    std::array<std::uint32_t, 64> m;
    for (std::size_t i = 0; i < 16; ++i) {
        m[i] = (static_cast<std::uint32_t>(block[i * 4]) << 24) |
               (static_cast<std::uint32_t>(block[i * 4 + 1]) << 16) |
               (static_cast<std::uint32_t>(block[i * 4 + 2]) << 8) |
               (static_cast<std::uint32_t>(block[i * 4 + 3]));
    }
    for (std::size_t i = 16; i < 64; ++i) {
        m[i] = sig1(m[i - 2]) + m[i - 7] + sig0(m[i - 15]) + m[i - 16];
    }

    std::uint32_t a = state_[0], b = state_[1], c = state_[2], d = state_[3];
    std::uint32_t e = state_[4], f = state_[5], g = state_[6], h = state_[7];

    for (std::size_t i = 0; i < 64; ++i) {
        const std::uint32_t t1 = h + ep1(e) + ch(e, f, g) + K[i] + m[i];
        const std::uint32_t t2 = ep0(a) + maj(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    state_[0] += a;
    state_[1] += b;
    state_[2] += c;
    state_[3] += d;
    state_[4] += e;
    state_[5] += f;
    state_[6] += g;
    state_[7] += h;
}

void Sha256::update(const std::uint8_t* data, std::size_t len) {
    std::size_t used = static_cast<std::size_t>(count_ % 64);
    count_ += len;

    // Top up a partially filled block first.
    if (used > 0) {
        const std::size_t take = std::min(len, 64 - used);
        std::copy_n(data, take, buffer_.begin() + static_cast<std::ptrdiff_t>(used));
        data += take;
        len -= take;
        used += take;
        if (used < 64) return;
        transform(buffer_.data());
    }

    while (len >= 64) {
        transform(data);
        data += 64;
        len -= 64;
    }

    std::copy_n(data, len, buffer_.begin());
}

void Sha256::update(std::string_view data) {
    update(reinterpret_cast<const std::uint8_t*>(data.data()), data.size());
}

Sha256::Digest Sha256::finish() {
    const std::uint64_t bit_len = count_ * 8;
    std::size_t used = static_cast<std::size_t>(count_ % 64);

    buffer_[used++] = 0x80;
    if (used > 56) {
        std::fill(buffer_.begin() + static_cast<std::ptrdiff_t>(used), buffer_.end(), std::uint8_t{0});
        transform(buffer_.data());
        used = 0;
    }
    std::fill(buffer_.begin() + static_cast<std::ptrdiff_t>(used), buffer_.begin() + 56, std::uint8_t{0});

    for (std::size_t i = 0; i < 8; ++i) {
        buffer_[63 - i] = static_cast<std::uint8_t>(bit_len >> (i * 8));
    }
    transform(buffer_.data());

    Digest digest;
    for (std::size_t i = 0; i < 8; ++i) {
        for (std::size_t j = 0; j < 4; ++j) {
            digest[i * 4 + j] = static_cast<std::uint8_t>(state_[i] >> (24 - j * 8));
        }
    }
    return digest;
}

std::string Sha256::hex_digest() { return to_hex(finish()); }

std::string sha256_hex(std::string_view data) {
    Sha256 ctx;
    ctx.update(data);
    return ctx.hex_digest();
}

std::optional<std::string> sha256_file(const std::string& path) {
    std::ifstream in(path, std::ios::binary);
    if (!in) return std::nullopt;

    Sha256 ctx;
    std::array<char, 8192> buf;
    while (in) {
        in.read(buf.data(), static_cast<std::streamsize>(buf.size()));
        const std::streamsize n = in.gcount();
        if (n <= 0) break;
        ctx.update(std::string_view(buf.data(), static_cast<std::size_t>(n)));
    }
    return ctx.hex_digest();
}

} // namespace spacetime
