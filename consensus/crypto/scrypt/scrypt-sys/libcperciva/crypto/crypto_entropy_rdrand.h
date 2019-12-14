#ifndef _CRYPTO_ENTROPY_RDRAND_H_
#define _CRYPTO_ENTROPY_RDRAND_H_

#include <stddef.h>

/**
 * generate_seed_rdrand(buf, len):
 * Fill the ${buf} buffer with values from RDRAND.  This implementation uses
 * the RDRAND instruction, and should only be used if CPUSUPPORT_X86_RDRAND is
 * defined and cpusupport_x86_rdrand() returns nonzero.
 */
int generate_seed_rdrand(unsigned int *, size_t);

#endif /* !_CRYPTO_ENTROPY_RDRAND_H_ */
