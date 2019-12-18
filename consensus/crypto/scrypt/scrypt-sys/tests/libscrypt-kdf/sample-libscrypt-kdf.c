#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "scrypt-kdf.h"

/* Parameters controlling memory usage and CPU time. */
#define N 16384
#define r 8
#define p 1

/* How much data should scrypt return? */
#define OUTPUT_BUFLEN 8

int
main(void)
{
	const char * passwd = "hunter2";
	const char * salt = "DANGER -- this should be a random salt -- DANGER";
	uint8_t output[OUTPUT_BUFLEN];
	int exitcode;

	/* Perform hashing. */
	exitcode = scrypt_kdf((const uint8_t *)passwd, strlen(passwd),
	    (const uint8_t*)salt, strlen(salt), N, r, p,
	    output, OUTPUT_BUFLEN);

	/* Notify user of success / failure. */
	if (exitcode == 0)
		printf("scrypt(): success\n");
	else
		printf("scrypt(): failure %i\n", exitcode);

	return (exitcode);
}
