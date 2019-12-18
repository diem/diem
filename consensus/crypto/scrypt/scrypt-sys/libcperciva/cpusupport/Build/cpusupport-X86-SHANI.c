#include <immintrin.h>
#include <stdint.h>

static char a[16];

int
main(void)
{
	__m128i x;

	x = _mm_loadu_si128((const __m128i *)&a[0]);
	x = _mm_sha256msg1_epu32(x, x);
	_mm_storeu_si128((__m128i *)a, x);
	return (a[0]);
}
