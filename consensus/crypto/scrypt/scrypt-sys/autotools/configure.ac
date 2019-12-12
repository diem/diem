AC_PREREQ([2.69])
AC_INIT([scrypt],1.3.0,
  [https://github.com/Tarsnap/scrypt],[scrypt],[http://www.tarsnap.com/scrypt/])
AC_CONFIG_SRCDIR([lib/scryptenc/scryptenc.c])
AC_CONFIG_AUX_DIR([config.aux])
AC_CONFIG_MACRO_DIR([m4])
AM_INIT_AUTOMAKE([foreign subdir-objects -Wall])

# Checks for programs.
AC_PROG_CC_C99
AM_PROG_AR
LT_INIT

# Don't rebuild the configure script.  I'm distributing a perfectly good one.
AM_MAINTAINER_MODE

# Checks for libraries.

AC_SEARCH_LIBS([AES_encrypt], [crypto],,
    AC_MSG_ERROR([function AES_ENCRYPT not found]))

# Check for a linuxy sysinfo syscall; and while we're doing that, check if
# struct sysinfo is the old version (total RAM == totalmem) or the new
# version (total RAM == totalmem * mem_unit).
AC_CHECK_HEADERS([sys/sysinfo.h])
AC_CHECK_FUNCS([sysinfo])
AC_CHECK_TYPES([struct sysinfo], [], [], [[#include <sys/sysinfo.h>]])
AC_CHECK_MEMBERS([struct sysinfo.totalram, struct sysinfo.mem_unit], [], [],
    [[#include <sys/sysinfo.h>]])

# Check if we have <sys/param.h>, since some systems require it for sysctl
# to work.
AC_CHECK_HEADERS([sys/param.h])

# Check if we have <openssl/aes.h>, to fix the make failing when configure
# passes.
AC_CHECK_HEADERS([openssl/aes.h],,
  [AC_MSG_ERROR([Unable to find the openssl/aes.h header])])

# Check for <sys/sysctl.h>.  If it exists and it defines HW_USERMEM
# and/or HW_MEMSIZE, we'll try using those as memory limits.
AC_CHECK_HEADERS([sys/sysctl.h])

# Check for posix_memalign
AC_CHECK_FUNCS([posix_memalign])

# Check for mmap so we can work around its absence on Minix
AC_CHECK_FUNCS([mmap])

AC_SYS_LARGEFILE

# Check whether the user has requested to disable compiler warnings
AC_MSG_CHECKING([compiler_warnings])
AC_ARG_ENABLE(compiler_warnings,
   AS_HELP_STRING([--disable-compiler-warnings],
       [Do not request compiler warnings. @<:@default=enabled@:>@]),
   [ac_compiler_warnings=$enableval],
   [ac_compiler_warnings=yes])
AC_MSG_RESULT([${ac_compiler_warnings}])
AS_IF([test x${ac_compiler_warnings} = xyes],
   [AX_CFLAGS_WARN_ALL])

# Check libcperciva POSIX compatibility
AC_MSG_NOTICE([checking POSIX compatibility...])
LIBCPERCIVA_DIR=${srcdir}/libcperciva
LDADD_POSIX=`export CC="${CC}"; command -p sh ${LIBCPERCIVA_DIR}/POSIX/posix-l.sh "$PATH"`
CFLAGS_POSIX=`export CC="${CC}"; command -p sh ${LIBCPERCIVA_DIR}/POSIX/posix-cflags.sh "$PATH"`
AC_SUBST([LDADD_POSIX])
AC_SUBST([CFLAGS_POSIX])
AC_MSG_RESULT([... done checking POSIX compatibility])

# Check whether the user wants to install libscrypt-kdf
AC_ARG_ENABLE(libscrypt-kdf, AS_HELP_STRING([--enable-libscrypt-kdf],
    [Install libscrypt-kdf and development headers.]))
AM_CONDITIONAL([LIBSCRYPT_KDF], [test "x${enable_libscrypt_kdf}" = "xyes"])

AC_CONFIG_HEADERS([config.h])
AC_CONFIG_FILES([Makefile])
AC_OUTPUT
