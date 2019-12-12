#ifndef _SHA256_SHANI_H_
#define _SHA256_SHANI_H_

#include <stdint.h>

/**
 * SHA256_Transform_shani(state, block):
 * Compute the SHA256 block compression function, transforming ${state} using
 * the data in ${block}.  This implementation uses x86 SHANI and SSSE3
 * instructions, and should only be used if CPUSUPPORT_X86_SHANI and _SSSE3
 * are defined and cpusupport_x86_shani() and _ssse3() return nonzero.
 */
void
SHA256_Transform_shani(uint32_t[static restrict 8],
    const uint8_t[static restrict 64]);

#endif /* !_SHA256_SHANI_H_ */
