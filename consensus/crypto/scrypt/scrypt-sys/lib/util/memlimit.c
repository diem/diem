/*-
 * Copyright 2009 Colin Percival
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 * This file was originally written by Colin Percival as part of the Tarsnap
 * online backup system.
 */

/* We use non-POSIX functionality in this file. */
#undef _POSIX_C_SOURCE
#undef _XOPEN_SOURCE

#include "platform.h"

#include <sys/types.h>
#include <sys/resource.h>

#ifdef HAVE_SYS_PARAM_H
#include <sys/param.h>
#endif
#ifdef HAVE_SYS_SYSCTL_H
#include <sys/sysctl.h>
#endif
#ifdef HAVE_SYS_SYSINFO_H
#include <sys/sysinfo.h>
#endif

#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>

#ifdef DEBUG
#include <stdio.h>
#endif

#include "memlimit.h"

/* If we don't have CTL_HW, we can't use HW_USERMEM. */
#ifndef CTL_HW
#undef HW_USERMEM
#endif

#ifdef CTL_HW
static int
memlimit_sysctl_hw(size_t * memlimit, int mibleaf)
{
	int mib[2];
	uint8_t sysctlbuf[8];
	size_t sysctlbuflen = 8;
	uint64_t sysctlval;

	/* Ask the kernel how much RAM we have. */
	mib[0] = CTL_HW;
	mib[1] = mibleaf;
	if (sysctl(mib, 2, sysctlbuf, &sysctlbuflen, NULL, 0))
		return (1);

	/*
	 * If we read 8 bytes out, assume this is a system-endian uint64_t.
	 * If we only read 4 bytes out, the OS is trying to give us a
	 * uint32_t answer -- but given how many systems now have 4GB+ of RAM,
	 * it's probably truncating, and we really can't trust the value we
	 * have returned to us.
	 */
	if (sysctlbuflen == sizeof(uint64_t))
		memcpy(&sysctlval, sysctlbuf, sizeof(uint64_t));
	else if (sysctlbuflen == sizeof(uint32_t))
		sysctlval = SIZE_MAX;
	else
		return (1);

	/* Return the sysctl value, but clamp to SIZE_MAX if necessary. */
#if UINT64_MAX > SIZE_MAX
	if (sysctlval > SIZE_MAX)
		*memlimit = SIZE_MAX;
	else
		*memlimit = (size_t)sysctlval;
#else
	*memlimit = sysctlval;
#endif

	/* Success! */
	return (0);
}
#endif

/* If we don't HAVE_STRUCT_SYSINFO, we can't use sysinfo. */
#ifndef HAVE_STRUCT_SYSINFO
#undef HAVE_SYSINFO
#endif

/* If we don't HAVE_STRUCT_SYSINFO_TOTALRAM, we can't use sysinfo. */
#ifndef HAVE_STRUCT_SYSINFO_TOTALRAM
#undef HAVE_SYSINFO
#endif

#ifdef HAVE_SYSINFO
static int
memlimit_sysinfo(size_t * memlimit)
{
	struct sysinfo info;
	uint64_t totalmem;

	/* Get information from the kernel. */
	if (sysinfo(&info))
		return (1);
	totalmem = info.totalram;

	/* If we're on a modern kernel, adjust based on mem_unit. */
#ifdef HAVE_STRUCT_SYSINFO_MEM_UNIT
	totalmem = totalmem * info.mem_unit;
#endif

	/* Return the value, but clamp to SIZE_MAX if necessary. */
#if UINT64_MAX > SIZE_MAX
	if (totalmem > SIZE_MAX)
		*memlimit = SIZE_MAX;
	else
		*memlimit = (size_t)totalmem;
#else
	*memlimit = totalmem;
#endif

	/* Success! */
	return (0);
}
#endif /* HAVE_SYSINFO */

static int
memlimit_rlimit(size_t * memlimit)
{
	struct rlimit rl;
	uint64_t memrlimit;

	/* Find the least of... */
	memrlimit = (uint64_t)(-1);

	/* ... RLIMIT_AS... */
#ifdef RLIMIT_AS
	if (getrlimit(RLIMIT_AS, &rl))
		return (1);
	if ((rl.rlim_cur != RLIM_INFINITY) &&
	    ((uint64_t)rl.rlim_cur < memrlimit))
		memrlimit = (uint64_t)rl.rlim_cur;
#endif

#ifndef HAVE_MMAP
	/* ... RLIMIT_DATA (if we're not using mmap)... */
	if (getrlimit(RLIMIT_DATA, &rl))
		return (1);
	if ((rl.rlim_cur != RLIM_INFINITY) &&
	    ((uint64_t)rl.rlim_cur < memrlimit))
		memrlimit = (uint64_t)rl.rlim_cur;
#endif

	/* ... and RLIMIT_RSS. */
#ifdef RLIMIT_RSS
	if (getrlimit(RLIMIT_RSS, &rl))
		return (1);
	if ((rl.rlim_cur != RLIM_INFINITY) &&
	    ((uint64_t)rl.rlim_cur < memrlimit))
		memrlimit = (uint64_t)rl.rlim_cur;
#endif

	/* Return the value, but clamp to SIZE_MAX if necessary. */
#if UINT64_MAX > SIZE_MAX
	if (memrlimit > SIZE_MAX)
		*memlimit = SIZE_MAX;
	else
		*memlimit = (size_t)memrlimit;
#else
	*memlimit = memrlimit;
#endif

	/* Success! */
	return (0);
}

#ifdef _SC_PHYS_PAGES

/* Some systems define _SC_PAGESIZE instead of _SC_PAGE_SIZE. */
#ifndef _SC_PAGE_SIZE
#define _SC_PAGE_SIZE _SC_PAGESIZE
#endif

static int
memlimit_sysconf(size_t * memlimit)
{
	long pagesize;
	long physpages;
	uint64_t totalmem;

	/* Set errno to 0 in order to distinguish "no limit" from "error". */
	errno = 0;

	/* Read the two limits. */
	if (((pagesize = sysconf(_SC_PAGE_SIZE)) == -1) ||
	    ((physpages = sysconf(_SC_PHYS_PAGES)) == -1)) {
		/*
		 * Did an error occur?  OS X may return EINVAL due to not
		 * supporting _SC_PHYS_PAGES in spite of defining it.
		 */
		if (errno != 0 && errno != EINVAL)
			return (1);

		/* If not, there is no limit. */
		totalmem = (uint64_t)(-1);
	} else {
		/* Compute the limit. */
		totalmem = (uint64_t)(pagesize) * (uint64_t)(physpages);
	}

	/* Return the value, but clamp to SIZE_MAX if necessary. */
#if UINT64_MAX > SIZE_MAX
	if (totalmem > SIZE_MAX)
		*memlimit = SIZE_MAX;
	else
		*memlimit = (size_t)totalmem;
#else
	*memlimit = totalmem;
#endif

	/* Success! */
	return (0);
}
#endif

/**
 * memtouse(maxmem, maxmemfrac, memlimit):
 * Examine the system and return via memlimit the amount of RAM which should
 * be used -- the specified fraction of the available RAM, but no more than
 * maxmem, and no less than 1MiB.
 */
int
memtouse(size_t maxmem, double maxmemfrac, size_t * memlimit)
{
	size_t usermem_memlimit, memsize_memlimit;
	size_t sysinfo_memlimit, rlimit_memlimit;
	size_t sysconf_memlimit;
	size_t memlimit_min;
	size_t memavail;

	/* Get memory limits. */
#ifdef HW_USERMEM
	if (memlimit_sysctl_hw(&usermem_memlimit, HW_USERMEM))
		return (1);
#else
	usermem_memlimit = SIZE_MAX;
#endif
#ifdef HW_MEMSIZE
	if (memlimit_sysctl_hw(&memsize_memlimit, HW_MEMSIZE))
		return (1);
#else
	memsize_memlimit = SIZE_MAX;
#endif
#ifdef HAVE_SYSINFO
	if (memlimit_sysinfo(&sysinfo_memlimit))
		return (1);
#else
	sysinfo_memlimit = SIZE_MAX;
#endif
	if (memlimit_rlimit(&rlimit_memlimit))
		return (1);
#ifdef _SC_PHYS_PAGES
	if (memlimit_sysconf(&sysconf_memlimit))
		return (1);
#else
	sysconf_memlimit = SIZE_MAX;
#endif

#ifdef DEBUG
	/* rlimit has two '\t' so that they line up. */
	fprintf(stderr, "Memory limits are:\n\tusermem:\t%zu\n"
	    "\tmemsize:\t%zu\n\tsysinfo:\t%zu\n\trlimit:\t\t%zu\n"
	    "\tsysconf:\t%zu\n", usermem_memlimit, memsize_memlimit,
	    sysinfo_memlimit, rlimit_memlimit, sysconf_memlimit);
#endif

	/*
	 * Some systems return bogus values for hw.usermem due to ZFS making
	 * use of wired pages.  Assume that at least 50% of physical pages
	 * are available to userland on demand.
	 */
	if (sysconf_memlimit != SIZE_MAX) {
		if (usermem_memlimit < sysconf_memlimit / 2)
			usermem_memlimit = sysconf_memlimit / 2;
	}

	/* Find the smallest of them. */
	memlimit_min = SIZE_MAX;
	if (memlimit_min > usermem_memlimit)
		memlimit_min = usermem_memlimit;
	if (memlimit_min > memsize_memlimit)
		memlimit_min = memsize_memlimit;
	if (memlimit_min > sysinfo_memlimit)
		memlimit_min = sysinfo_memlimit;
	if (memlimit_min > rlimit_memlimit)
		memlimit_min = rlimit_memlimit;
	if (memlimit_min > sysconf_memlimit)
		memlimit_min = sysconf_memlimit;

	/* Only use the specified fraction of the available memory. */
	if ((maxmemfrac > 0.5) || (maxmemfrac == 0.0))
		maxmemfrac = 0.5;
	memavail = (size_t)(maxmemfrac * memlimit_min);

	/* Don't use more than the specified maximum. */
	if ((maxmem > 0) && (memavail > maxmem))
		memavail = maxmem;

	/* But always allow at least 1 MiB. */
	if (memavail < 1048576)
		memavail = 1048576;

#ifdef DEBUG
	fprintf(stderr, "Allowing up to %zu memory to be used\n", memavail);
#endif

	/* Return limit via the provided pointer. */
	*memlimit = memavail;
	return (0);
}
