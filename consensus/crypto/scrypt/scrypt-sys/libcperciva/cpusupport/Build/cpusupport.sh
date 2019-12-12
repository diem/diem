# Should be sourced by `command -p sh path/to/cpusupport.sh "$PATH"` from
# within a Makefile.
if ! [ ${PATH} = "$1" ]; then
	echo "WARNING: POSIX violation: $SHELL's command -p resets \$PATH" 1>&2
	PATH=$1
fi
# Standard output should be written to cpusupport-config.h, which is both a
# C header file defining CPUSUPPORT_ARCH_FEATURE macros and sourceable sh
# code which sets CFLAGS_ARCH_FEATURE environment variables.
SRCDIR=`command -p dirname "$0"`

feature() {
	ARCH=$1
	FEATURE=$2
	shift 2;
	if ! [ -f ${SRCDIR}/cpusupport-$ARCH-$FEATURE.c ]; then
		return
	fi
	printf "Checking if compiler supports $ARCH $FEATURE feature..." 1>&2
	for CFLAG in "$@"; do
		if ${CC} ${CFLAGS} -D_POSIX_C_SOURCE=200809L ${CFLAG}	\
		    ${SRCDIR}/cpusupport-$ARCH-$FEATURE.c 2>/dev/null; then
			rm -f a.out
			break;
		fi
		CFLAG=NOTSUPPORTED;
	done
	case $CFLAG in
	NOTSUPPORTED)
		echo " no" 1>&2
		;;
	"")
		echo " yes" 1>&2
		echo "#define CPUSUPPORT_${ARCH}_${FEATURE} 1"
		;;
	*)
		echo " yes, via $CFLAG" 1>&2
		echo "#define CPUSUPPORT_${ARCH}_${FEATURE} 1"
		echo "#ifdef cpusupport_dummy"
		echo "export CFLAGS_${ARCH}_${FEATURE}=\"${CFLAG}\""
		echo "#endif"
		;;
	esac
}

# Detect CPU-detection features
feature X86 CPUID ""
feature X86 CPUID_COUNT ""

# Detect specific features
feature X86 AESNI "" "-maes" "-maes -Wno-cast-align" "-maes -Wno-missing-prototypes -Wno-cast-qual"
feature X86 CRC32 "" "-msse4.2" "-msse4.2 -Wno-cast-align" "-msse4.2 -Wno-cast-align -fno-strict-aliasing"
feature X86 RDRAND "" "-mrdrnd"
feature X86 SHANI "" "-msse2 -msha" "-msse2 -msha -Wno-cast-align"
feature X86 SSE2 "" "-msse2" "-msse2 -Wno-cast-align"
feature X86 SSSE3 "" "-mssse3" "-mssse3 -Wno-cast-align"
