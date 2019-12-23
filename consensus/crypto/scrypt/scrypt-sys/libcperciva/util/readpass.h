#ifndef _READPASS_H_
#define _READPASS_H_

/* Avoid namespace collisions with other "readpass" functions. */
#ifdef readpass
#undef readpass
#endif
#define readpass libcperciva_readpass

/**
 * readpass(passwd, prompt, confirmprompt, devtty):
 * If ${devtty} is non-zero, read a password from /dev/tty if possible; if
 * not, read from stdin.  If reading from a tty (either /dev/tty or stdin),
 * disable echo and prompt the user by printing ${prompt} to stderr.  If
 * ${confirmprompt} is non-NULL, read a second password (prompting if a
 * terminal is being used) and repeat until the user enters the same password
 * twice.  Return the password as a malloced NUL-terminated string via
 * ${passwd}.
 */
int readpass(char **, const char *, const char *, int);

#endif /* !_READPASS_H_ */
