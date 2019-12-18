#ifndef _CRYPTO_AES_H_
#define _CRYPTO_AES_H_

#include <stddef.h>
#include <stdint.h>

/* Opaque structure. */
struct crypto_aes_key;

/**
 * crypto_aes_key_expand(key, len):
 * Expand the ${len}-byte AES key ${key} into a structure which can be passed
 * to crypto_aes_encrypt_block.  The length must be 16 or 32.
 */
struct crypto_aes_key * crypto_aes_key_expand(const uint8_t *, size_t);

/**
 * crypto_aes_encrypt_block(in, out, key):
 * Using the expanded AES key ${key}, encrypt the block ${in} and write the
 * resulting ciphertext to ${out}.
 */
void crypto_aes_encrypt_block(const uint8_t *, uint8_t *,
    const struct crypto_aes_key *);

/**
 * crypto_aes_key_free(key):
 * Free the expanded AES key ${key}.
 */
void crypto_aes_key_free(struct crypto_aes_key *);

#endif /* !_CRYPTO_AES_H_ */
