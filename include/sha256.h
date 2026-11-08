#ifndef SPACETIME_SHA256_H
#define SPACETIME_SHA256_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint32_t state[8];
    uint64_t count;
    uint8_t buffer[64];
} Sha256Context;

void sha256_init(Sha256Context *ctx);
void sha256_update(Sha256Context *ctx, const uint8_t *data, size_t len);
void sha256_final(Sha256Context *ctx, uint8_t hash[32]);

void sha256_bytes(const uint8_t *data, size_t len, char hex_out[65]);
int sha256_file(const char *filepath, char hex_out[65]);

#endif /* SPACETIME_SHA256_H */
