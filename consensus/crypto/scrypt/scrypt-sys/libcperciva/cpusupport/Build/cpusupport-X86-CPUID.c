#include <cpuid.h>

int
main(void)
{
	unsigned int a, b, c, d;

	return __get_cpuid(0, &a, &b, &c, &d);
}
