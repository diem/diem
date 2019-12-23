#include <stddef.h>
#include <stdint.h>

#include "crypto_verify_bytes.h"

/**
 * crypto_verify_bytes(buf0, buf1, len):
 * Return zero if and only if buf0[0 .. len - 1] and buf1[0 .. len - 1] are
 * identical.  Do not leak any information via timing side channels.
 */
uint8_t
crypto_verify_bytes(const uint8_t * buf0, const uint8_t * buf1, size_t len)
{
	uint8_t rc = 0;
	size_t i;

	for (i = 0; i < len; i++)
		rc = rc | (buf0[i] ^ buf1[i]);

	return (rc);
}
