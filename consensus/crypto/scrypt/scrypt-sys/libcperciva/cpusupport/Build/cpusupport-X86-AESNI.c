#include <stdint.h>

#include <wmmintrin.h>

static uint8_t a[16];

int
main(void)
{
	__m128i x, y;

	x = _mm_loadu_si128((const __m128i *)&a[0]);
	y = _mm_aesenc_si128(x, x);
	_mm_storeu_si128((__m128i *)&a[0], y);
	return (a[0]);
}
