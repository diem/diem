#ifndef _CRYPTO_SCRYPT_SMIX_SSE2_H_
#define _CRYPTO_SCRYPT_SMIX_SSE2_H_

#include <stddef.h>
#include <stdint.h>

/**
 * crypto_scrypt_smix_sse2(B, r, N, V, XY):
 * Compute B = SMix_r(B, N).  The input B must be 128r bytes in length;
 * the temporary storage V must be 128rN bytes in length; the temporary
 * storage XY must be 256r + 64 bytes in length.  The value N must be a
 * power of 2 greater than 1.  The arrays B, V, and XY must be aligned to a
 * multiple of 64 bytes.
 *
 * Use SSE2 instructions.
 */
void crypto_scrypt_smix_sse2(uint8_t *, size_t, uint64_t, void *, void *);

#endif /* !_CRYPTO_SCRYPT_SMIX_SSE2_H_ */
