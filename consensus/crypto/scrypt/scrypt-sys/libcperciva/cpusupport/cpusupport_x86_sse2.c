#include "cpusupport.h"

#ifdef CPUSUPPORT_X86_CPUID
#include <cpuid.h>

#define CPUID_SSE2_BIT (1 << 26)
#endif

CPUSUPPORT_FEATURE_DECL(x86, sse2)
{
#ifdef CPUSUPPORT_X86_CPUID
	unsigned int eax, ebx, ecx, edx;

	/* Check if CPUID supports the level we need. */
	if (!__get_cpuid(0, &eax, &ebx, &ecx, &edx))
		goto unsupported;
	if (eax < 1)
		goto unsupported;

	/* Ask about CPU features. */
	if (!__get_cpuid(1, &eax, &ebx, &ecx, &edx))
		goto unsupported;

	/* Return the relevant feature bit. */
	return ((edx & CPUID_SSE2_BIT) ? 1 : 0);

unsupported:
#endif
	return (0);
}
