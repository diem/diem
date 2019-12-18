#ifndef _ASPRINTF_H_
#define _ASPRINTF_H_

/* Avoid namespace collisions with BSD/GNU asprintf. */
#ifdef asprintf
#undef asprintf
#endif
#define asprintf libcperciva_asprintf

/**
 * asprintf(ret, format, ...):
 * Do asprintf(3) like GNU and BSD do.
 */
int asprintf(char **, const char *, ...);

#endif /* !_ASPRINTF_H_ */
