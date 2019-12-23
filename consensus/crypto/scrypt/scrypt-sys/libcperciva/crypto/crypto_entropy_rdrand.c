#include "cpusupport.h"
#ifdef CPUSUPPORT_X86_RDRAND

#include <immintrin.h>
#include <stddef.h>

#include "crypto_entropy_rdrand.h"

/**
 * generate_seed_rdrand(buf, len):
 * Fill the ${buf} buffer with values from RDRAND.  This implementation uses
 * the RDRAND instruction, and should only be used if CPUSUPPORT_X86_RDRAND is
 * defined and cpusupport_x86_rdrand() returns nonzero.
 */
int
generate_seed_rdrand(unsigned int * buf, size_t len)
{
	size_t i;

	/* Fill buffer. */
	for (i = 0; i < len; i++) {
		if (!_rdrand32_step(&buf[i]))
			goto err0;
	}

	/* Success! */
	return (0);

err0:
	/* Failure! */
	return (1);
}
#endif /* CPUSUPPORT_X86_RDRAND */
