#include "catch2/catch_amalgamated.hpp"

#include <string>

#include "sha256.hpp"
#include "test_helpers.hpp"

using namespace spacetime;

TEST_CASE("sha256 matches the FIPS 180-4 test vectors", "[sha256]") {
    CHECK(sha256_hex("") == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    CHECK(sha256_hex("abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    CHECK(sha256_hex("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq") ==
          "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    CHECK(sha256_hex(std::string(1'000'000, 'a')) ==
          "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
}

TEST_CASE("sha256 handles every padding boundary", "[sha256]") {
    // Lengths around the 56- and 64-byte marks exercise both padding branches.
    CHECK(sha256_hex(std::string(55, 'a')) == "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318");
    CHECK(sha256_hex(std::string(56, 'a')) == "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a");
    CHECK(sha256_hex(std::string(64, 'a')) == "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb");
}

TEST_CASE("sha256 incremental updates equal a one-shot hash", "[sha256]") {
    const std::string data(1000, 'x');
    for (std::size_t step : {1u, 7u, 63u, 64u, 65u, 300u}) {
        Sha256 ctx;
        for (std::size_t pos = 0; pos < data.size(); pos += step) {
            ctx.update(std::string_view(data).substr(pos, step));
        }
        CHECK(ctx.hex_digest() == sha256_hex(data));
    }
}

TEST_CASE("sha256_file hashes file contents", "[sha256]") {
    TempDir dir;
    const std::string content(20'000, 'z'); // spans several read buffers
    dir.write("data.bin", content);

    CHECK(sha256_file(dir.path() + "/data.bin") == sha256_hex(content));
    CHECK_FALSE(sha256_file(dir.path() + "/missing.bin").has_value());
}
