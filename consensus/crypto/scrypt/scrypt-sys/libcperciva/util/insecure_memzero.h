#ifndef _INSECURE_MEMZERO_H_
#define _INSECURE_MEMZERO_H_

#include <stddef.h>

/* Pointer to memory-zeroing function. */
extern void (* volatile insecure_memzero_ptr)(volatile void *, size_t);

/**
 * insecure_memzero(buf, len):
 * Attempt to zero ${len} bytes at ${buf} in spite of optimizing compilers'
 * best (standards-compliant) attempts to remove the buffer-zeroing.  In
 * particular, to avoid performing the zeroing, a compiler would need to
 * use optimistic devirtualization; recognize that non-volatile objects do not
 * need to be treated as volatile, even if they are accessed via volatile
 * qualified pointers; and perform link-time optimization; in addition to the
 * dead-code elimination which often causes buffer-zeroing to be elided.
 *
 * Note however that zeroing a buffer does not guarantee that the data held
 * in the buffer is not stored elsewhere; in particular, there may be copies
 * held in CPU registers or in anonymous allocations on the stack, even if
 * every named variable is successfully sanitized.  Solving the "wipe data
 * from the system" problem will require a C language extension which does not
 * yet exist.
 *
 * For more information, see:
 * http://www.daemonology.net/blog/2014-09-04-how-to-zero-a-buffer.html
 * http://www.daemonology.net/blog/2014-09-06-zeroing-buffers-is-insufficient.html
 */
static inline void
insecure_memzero(volatile void * buf, size_t len)
{

	(insecure_memzero_ptr)(buf, len);
}

#endif /* !_INSECURE_MEMZERO_H_ */
