#include <cpuid.h>

int
main(void)
{
	unsigned int a, b, c, d;

	__cpuid_count(7, 0, a, b, c, d);
	return ((int)a);
}
