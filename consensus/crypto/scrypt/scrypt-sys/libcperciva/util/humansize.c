#include <stdio.h>

#include "asprintf.h"
#include "warnp.h"

#include "humansize.h"

/**
 * humansize(size):
 * Given a size in bytes, allocate and return a string of the form "<N> B"
 * for 0 <= N <= 999 or "<X> <prefix>B" where either 10 <= X <= 999 or
 * 1.0 <= X <= 9.9 and <prefix> is "k", "M", "G", "T", "P", or "E"; and where
 * the value returned is the largest valid value <= the provided size.
 */
char *
humansize(uint64_t size)
{
	char * s;
	char prefix;
	int shiftcnt;
	int rc;

	/* Special-case for size < 1000. */
	if (size < 1000) {
		rc = asprintf(&s, "%d B", (int)size);
	} else {
		/* Keep 10 * size / 1000^(3n) in size. */
		for (size /= 100, shiftcnt = 1; size >= 10000; shiftcnt++)
			size /= 1000;

		/*
		 * Figure out what prefix to use.  Since 1 EB = 10^18 B and
		 * the maximum value of a uint64_t is 2^64 which is roughly
		 * 18.4 * 10^18, this cannot reference beyond the end of the
		 * string.
		 */
		prefix = " kMGTPE"[shiftcnt];

		/* Construct the string. */
		if (size < 100)
			rc = asprintf(&s, "%d.%d %cB", (int)size / 10,
			    (int)size % 10, prefix);
		else
			rc = asprintf(&s, "%d %cB", (int)size / 10, prefix);
	}

	if (rc == -1) {
		warnp("asprintf");
		goto err0;
	}

	/* Success! */
	return (s);

err0:
	/* Failure! */
	return (NULL);
}

/**
 * humansize_parse(s, size):
 * Parse a string matching /[0-9]+ ?[kMGTPE]?B?/ as a size in bytes.
 */
int
humansize_parse(const char * s, uint64_t * size)
{
	int state = 0;
	uint64_t multiplier = 1;

	do {
		switch (state) {
		case -1:
			/* Error state. */
			break;
		case 0:
			/* Initial state. */
			*size = 0;

			/* We must start with at least one digit. */
			if ((*s < '0') || (*s > '9')) {
				state = -1;
				break;
			}

			/* FALLTHROUGH */
		case 1:
			/* We're now processing digits. */
			state = 1;

			/* Digit-parsing state. */
			if (('0' <= *s) && (*s <= '9')) {
				if (*size > UINT64_MAX / 10)
					state = -1;
				else
					*size *= 10;
				if (*size > UINT64_MAX - (uint64_t)(*s - '0'))
					state = -1;
				else
					*size += (uint64_t)(*s - '0');
				break;
			}

			/* FALLTHROUGH */
		case 2:
			/* We move into state 3 after an optional ' '. */
			state = 3;
			if (*s == ' ')
				break;

			/* FALLTHROUGH */
		case 3:
			/* We may have one SI prefix. */
			switch (*s) {
			case 'E':
				multiplier *= 1000;
				/* FALLTHROUGH */
			case 'P':
				multiplier *= 1000;
				/* FALLTHROUGH */
			case 'T':
				multiplier *= 1000;
				/* FALLTHROUGH */
			case 'G':
				multiplier *= 1000;
				/* FALLTHROUGH */
			case 'M':
				multiplier *= 1000;
				/* FALLTHROUGH */
			case 'k':
				multiplier *= 1000;
				break;
			}

			/* We move into state 4 after the optional prefix. */
			state = 4;
			if (multiplier != 1)
				break;

			/* FALLTHROUGH */
		case 4:
			/* We move into state 5 after an optional 'B'. */
			state = 5;
			if (*s == 'B')
				break;

			/* FALLTHROUGH */
		case 5:
			/* We have trailing garbage. */
			state = -1;
			break;
		}

		/* Move on to the next character. */
		s++;
	} while (*s != '\0');

	/* Multiply by multiplier. */
	if (*size > UINT64_MAX / multiplier)
		state = -1;
	else
		*size *= multiplier;

	/* Anything other than state -1 is success. */
	return ((state == -1) ? -1 : 0);
}
