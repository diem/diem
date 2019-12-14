#ifndef _HUMANSIZE_H_
#define _HUMANSIZE_H_

#include <stdint.h>

/**
 * humansize(size):
 * Given a size in bytes, allocate and return a string of the form "<N> B"
 * for 0 <= N <= 999 or "<X> <prefix>B" where either 10 <= X <= 999 or
 * 1.0 <= X <= 9.9 and <prefix> is "k", "M", "G", "T", "P", or "E"; and where
 * the value returned is the largest valid value <= the provided size.
 */
char * humansize(uint64_t);

/**
 * humansize_parse(s, size):
 * Parse a string matching /[0-9]+ ?[kMGTPE]?B?/ as a size in bytes.
 */
int humansize_parse(const char *, uint64_t *);

#endif /* !_HUMANSIZE_H_ */
