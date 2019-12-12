#ifndef _CRYPTO_AES_AESNI_H_
#define _CRYPTO_AES_AESNI_H_

#include <stddef.h>
#include <stdint.h>

/**
 * crypto_aes_key_expand_aesni(key, len):
 * Expand the ${len}-byte AES key ${key} into a structure which can be passed
 * to crypto_aes_encrypt_block_aesni.  The length must be 16 or 32.  This
 * implementation uses x86 AESNI instructions, and should only be used if
 * CPUSUPPORT_X86_AESNI is defined and cpusupport_x86_aesni() returns nonzero.
 */
void * crypto_aes_key_expand_aesni(const uint8_t *, size_t);

/**
 * crypto_aes_encrypt_block_aesni(in, out, key):
 * Using the expanded AES key ${key}, encrypt the block ${in} and write the
 * resulting ciphertext to ${out}.  This implementation uses x86 AESNI
 * instructions, and should only be used if CPUSUPPORT_X86_AESNI is defined
 * and cpusupport_x86_aesni() returns nonzero.
 */
void crypto_aes_encrypt_block_aesni(const uint8_t *, uint8_t *, const void *);

/**
 * crypto_aes_key_free_aesni(key):
 * Free the expanded AES key ${key}.
 */
void crypto_aes_key_free_aesni(void *);

#endif /* !_CRYPTO_AES_AESNI_H_ */
