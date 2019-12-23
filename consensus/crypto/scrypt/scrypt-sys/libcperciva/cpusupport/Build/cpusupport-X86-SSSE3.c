#include <emmintrin.h>
#include <tmmintrin.h>

static char a[16];

int
main(void)
{
	__m128i x;

	x = _mm_loadu_si128((__m128i *)a);
	x = _mm_alignr_epi8(x, x, 8);
	_mm_storeu_si128((__m128i *)a, x);
	return (a[0]);
}
