#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#include "asprintf.h"

/**
 * asprintf(ret, format, ...):
 * Do asprintf(3) like GNU and BSD do.
 */
int
asprintf(char ** ret, const char * format, ...)
{
	va_list ap;
	int len;
	size_t buflen;

	/* Figure out how long the string needs to be. */
	va_start(ap, format);
	len = vsnprintf(NULL, 0, format, ap);
	va_end(ap);

	/* Did we fail? */
	if (len < 0)
		goto err0;
	buflen = (size_t)(len) + 1;

	/* Allocate memory. */
	if ((*ret = malloc(buflen)) == NULL)
		goto err0;

	/* Actually generate the string. */
	va_start(ap, format);
	len = vsnprintf(*ret, buflen, format, ap);
	va_end(ap);

	/* Did we fail? */
	if (len < 0)
		goto err1;

	/* Success! */
	return (len);

err1:
	free(*ret);
err0:
	/* Failure! */
	return (-1);
}
