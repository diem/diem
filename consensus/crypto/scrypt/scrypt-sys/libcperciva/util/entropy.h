#ifndef _ENTROPY_H_
#define _ENTROPY_H_

#include <stddef.h>
#include <stdint.h>

/**
 * entropy_read(buf, buflen):
 * Fill the given buffer with random bytes provided by the operating system.
 */
int entropy_read(uint8_t *, size_t);

#endif /* !_ENTROPY_H_ */
