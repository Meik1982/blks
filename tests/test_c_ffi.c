#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "../include/blks.h"

int main(void) {
    printf("Testing blks C-FFI...\n");

    const char *data = "The quick brown fox jumps over the lazy dog";
    size_t len = strlen(data);

    // 1. One-shot buffer hash
    uint8_t digest[BLKS_DIGEST_BYTES];
    int res = blks_hash_buffer((const uint8_t *)data, len, digest);
    assert(res == 0);

    // 2. Base64 encoding
    char b64[BLKS_BASE64_CHARS + 1];
    res = blks_digest_to_base64(digest, b64, sizeof(b64));
    assert(res == 0);
    assert(strlen(b64) == BLKS_BASE64_CHARS);
    printf("C-FFI One-shot Base64: %s\n", b64);

    // 3. Streaming hasher
    BlksHasher *h = blks_hasher_new();
    assert(h != NULL);
    res = blks_hasher_update(h, (const uint8_t *)data, 10);
    assert(res == 0);
    res = blks_hasher_update(h, (const uint8_t *)data + 10, len - 10);
    assert(res == 0);

    uint8_t stream_digest[BLKS_DIGEST_BYTES];
    res = blks_hasher_finalize(h, stream_digest);
    assert(res == 0);

    char stream_b64[BLKS_BASE64_CHARS + 1];
    res = blks_digest_to_base64(stream_digest, stream_b64, sizeof(stream_b64));
    assert(res == 0);

    assert(memcmp(digest, stream_digest, BLKS_DIGEST_BYTES) == 0);
    assert(strcmp(b64, stream_b64) == 0);

    printf("C-FFI Streaming Base64: %s (bit-exact match!)\n", stream_b64);
    printf("All C-FFI tests passed successfully!\n");
    return 0;
}
