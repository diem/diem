#ifndef _CRYPTO_ENTROPY_H_
#define _CRYPTO_ENTROPY_H_

#include <stddef.h>
#include <stdint.h>

/**
 * crypto_entropy_read(buf, buflen):
 * Fill the buffer with unpredictable bits.
 */
int crypto_entropy_read(uint8_t *, size_t);

#endif /* !_CRYPTO_ENTROPY_H_ */
