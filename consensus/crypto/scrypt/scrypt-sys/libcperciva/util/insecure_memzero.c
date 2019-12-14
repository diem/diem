#include <stddef.h>
#include <stdint.h>

#include "insecure_memzero.h"

/* Function which does the zeroing. */
static void
insecure_memzero_func(volatile void * buf, size_t len)
{
	volatile uint8_t * _buf = buf;
	size_t i;

	for (i = 0; i < len; i++)
		_buf[i] = 0;
}

/* Pointer to memory-zeroing function. */
void (* volatile insecure_memzero_ptr)(volatile void *, size_t) =
    insecure_memzero_func;
