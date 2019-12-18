#ifndef _CRYPTO_VERIFY_BYTES_H_
#define _CRYPTO_VERIFY_BYTES_H_

#include <stddef.h>
#include <stdint.h>

/**
 * crypto_verify_bytes(buf0, buf1, len):
 * Return zero if and only if buf0[0 .. len - 1] and buf1[0 .. len - 1] are
 * identical.  Do not leak any information via timing side channels.
 */
uint8_t crypto_verify_bytes(const uint8_t *, const uint8_t *, size_t);

#endif /* !_CRYPTO_VERIFY_BYTES_H_ */
