#ifndef _CPUSUPPORT_H_
#define _CPUSUPPORT_H_

/*
 * To enable support for non-portable CPU features at compile time, one or
 * more CPUSUPPORT_ARCH_FEATURE macros should be defined.  This can be done
 * directly on the compiler command line via -D CPUSUPPORT_ARCH_FEATURE or
 * -D CPUSUPPORT_ARCH_FEATURE=1; or a file can be created with the
 * necessary #define lines and then -D CPUSUPPORT_CONFIG_FILE=cpuconfig.h
 * (or similar) can be provided to include that file here.
 */
#ifdef CPUSUPPORT_CONFIG_FILE
#include CPUSUPPORT_CONFIG_FILE
#endif

/**
 * The CPUSUPPORT_FEATURE macro declares the necessary variables and
 * functions for detecting CPU feature support at run time.  The function
 * defined in the macro acts to cache the result of the ..._detect function
 * using the ..._present and ..._init variables.  The _detect function and the
 * _present and _init variables are turn defined by CPUSUPPORT_FEATURE_DECL in
 * appropriate cpusupport_foo_bar.c file.
 *
 * In order to allow CPUSUPPORT_FEATURE to be used for features which do not
 * have corresponding CPUSUPPORT_FEATURE_DECL blocks in another source file,
 * we abuse the C preprocessor: If CPUSUPPORT_${enabler} is defined to 1, then
 * we access _present_1, _init_1, and _detect_1; but if it is not defined, we
 * access _present_CPUSUPPORT_${enabler} etc., which we define as static, thus
 * preventing the compiler from emitting a reference to an external symbol.
 *
 * In this way, it becomes possible to issue CPUSUPPORT_FEATURE invocations
 * for nonexistent features without running afoul of the requirement that
 * "If an identifier declared with external linkage is used... in the entire
 * program there shall be exactly one external definition" (C99 standard, 6.9
 * paragraph 5).  In practice, this means that users of the cpusupport code
 * can omit build and runtime detection files without changing the framework
 * code.
 */
#define CPUSUPPORT_FEATURE__(arch_feature, enabler, enabled)					\
	static int cpusupport_ ## arch_feature ## _present ## _CPUSUPPORT_ ## enabler;		\
	static int cpusupport_ ## arch_feature ## _init ## _CPUSUPPORT_ ## enabler;		\
	static inline int cpusupport_ ## arch_feature ## _detect ## _CPUSUPPORT_ ## enabler(void) { return (0); }	\
	extern int cpusupport_ ## arch_feature ## _present_ ## enabled;				\
	extern int cpusupport_ ## arch_feature ## _init_ ## enabled;				\
	int cpusupport_ ## arch_feature ## _detect_ ## enabled(void);				\
												\
	static inline int									\
	cpusupport_ ## arch_feature(void)							\
	{											\
												\
		if (cpusupport_ ## arch_feature ## _present_ ## enabled)			\
			return (1);								\
		else if (cpusupport_ ## arch_feature ## _init_ ## enabled)			\
			return (0);								\
		cpusupport_ ## arch_feature ## _present_ ## enabled =				\
		    cpusupport_ ## arch_feature ## _detect_ ## enabled();			\
		cpusupport_ ## arch_feature ## _init_ ## enabled = 1;				\
		return (cpusupport_ ## arch_feature ## _present_ ## enabled);			\
	}											\
	static void (* cpusupport_ ## arch_feature ## _dummyptr)(void);				\
	static inline void									\
	cpusupport_ ## arch_feature ## _dummyfunc(void)						\
	{											\
												\
		(void)cpusupport_ ## arch_feature ## _present ## _CPUSUPPORT_ ## enabler;	\
		(void)cpusupport_ ## arch_feature ## _init ## _CPUSUPPORT_ ## enabler;		\
		(void)cpusupport_ ## arch_feature ## _detect ## _CPUSUPPORT_ ## enabler;	\
		(void)cpusupport_ ## arch_feature ## _present_ ## enabled;			\
		(void)cpusupport_ ## arch_feature ## _init_ ## enabled;				\
		(void)cpusupport_ ## arch_feature ## _detect_ ## enabled;			\
		(void)cpusupport_ ## arch_feature ## _dummyptr;					\
	}											\
	static void (* cpusupport_ ## arch_feature ## _dummyptr)(void) = cpusupport_ ## arch_feature ## _dummyfunc;	\
	struct cpusupport_ ## arch_feature ## _dummy
#define CPUSUPPORT_FEATURE_(arch_feature, enabler, enabled)	\
	CPUSUPPORT_FEATURE__(arch_feature, enabler, enabled)
#define CPUSUPPORT_FEATURE(arch, feature, enabler)				\
	CPUSUPPORT_FEATURE_(arch ## _ ## feature, enabler, CPUSUPPORT_ ## enabler)

/*
 * CPUSUPPORT_FEATURE_DECL(arch, feature):
 * Macro which defines variables and provides a function declaration for
 * detecting the presence of "feature" on the "arch" architecture.  The
 * function body following this macro expansion must return nonzero if the
 * feature is present, or zero if the feature is not present or the detection
 * fails for any reason.
 */
#define CPUSUPPORT_FEATURE_DECL(arch, feature)				\
	extern int cpusupport_ ## arch ## _ ## feature ## _present_1;	\
	extern int cpusupport_ ## arch ## _ ## feature ## _init_1;	\
	int cpusupport_ ## arch ## _ ## feature ## _present_1 = 0;	\
	int cpusupport_ ## arch ## _ ## feature ## _init_1 = 0;		\
	int cpusupport_ ## arch ## _ ## feature ## _detect_1(void); \
	int								\
	cpusupport_ ## arch ## _ ## feature ## _detect_1(void)

/*
 * List of features.  If a feature here is not enabled by the appropriate
 * CPUSUPPORT_ARCH_FEATURE macro being defined, it has no effect; but if the
 * relevant macro may be defined (e.g., by Build/cpusupport.sh successfully
 * compiling Build/cpusupport-ARCH-FEATURE.c) then the C file containing the
 * corresponding run-time detection code (cpusupport_arch_feature.c) must be
 * compiled and linked in.
 */
CPUSUPPORT_FEATURE(x86, aesni, X86_AESNI);
CPUSUPPORT_FEATURE(x86, crc32, X86_CRC32);
CPUSUPPORT_FEATURE(x86, rdrand, X86_RDRAND);
CPUSUPPORT_FEATURE(x86, shani, X86_SHANI);
CPUSUPPORT_FEATURE(x86, sse2, X86_SSE2);
CPUSUPPORT_FEATURE(x86, ssse3, X86_SSSE3);

#endif /* !_CPUSUPPORT_H_ */
