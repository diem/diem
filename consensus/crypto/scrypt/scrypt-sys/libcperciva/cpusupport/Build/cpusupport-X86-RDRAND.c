#include <immintrin.h>

int
main(void)
{
	unsigned int x;

	return(!_rdrand32_step(&x));
}
