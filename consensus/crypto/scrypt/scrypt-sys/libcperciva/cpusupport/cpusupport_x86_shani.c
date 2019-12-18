#include "cpusupport.h"

#ifdef CPUSUPPORT_X86_CPUID_COUNT
#include <cpuid.h>

#define CPUID_SHANI_BIT (1 << 29)
#endif

CPUSUPPORT_FEATURE_DECL(x86, shani)
{
#ifdef CPUSUPPORT_X86_CPUID_COUNT
	unsigned int eax, ebx, ecx, edx;

	/* Check if CPUID supports the level we need. */
	if (!__get_cpuid(0, &eax, &ebx, &ecx, &edx))
		goto unsupported;
	if (eax < 7)
		goto unsupported;

	/*
	 * Ask about extended CPU features.  Note that this macro violates
	 * the principle of being "function-like" by taking the variables
	 * used for holding output registers as named parameters rather than
	 * as pointers (which would be necessary if __cpuid_count were a
	 * function).
	 */
	__cpuid_count(7, 0, eax, ebx, ecx, edx);

	/* Return the relevant feature bit. */
	return ((ebx & CPUID_SHANI_BIT) ? 1 : 0);

unsupported:
#endif
	return (0);
}
