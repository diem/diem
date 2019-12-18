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
#ifndef _SCRYPTENC_H_
#define _SCRYPTENC_H_

#include <stdint.h>
#include <stdio.h>

/**
 * NOTE: This file provides prototypes for routines which encrypt/decrypt data
 * using a key derived from a password by using the scrypt key derivation
 * function.  If you are just trying to "hash" a password for user logins,
 * this is not the code you are looking for.  You want to use the crypto_scrypt
 * function directly.
 */

/**
 * The parameters maxmem, maxmemfrac, and maxtime used by all of these
 * functions are defined as follows:
 * maxmem - maximum number of bytes of storage to use for V array (which is
 *     by far the largest consumer of memory).  If this value is set to 0, no
 *     maximum will be enforced; any other value less than 1 MiB will be
 *     treated as 1 MiB.
 * maxmemfrac - maximum fraction of available storage to use for the V array,
 *     where "available storage" is defined as the minimum out of the
 *     RLIMIT_AS, RLIMIT_DATA. and RLIMIT_RSS resource limits (if any are
 *     set).  If this value is set to 0 or more than 0.5 it will be treated
 *     as 0.5; and this value will never cause a limit of less than 1 MiB to
 *     be enforced.
 * maxtime - maximum amount of CPU time to spend computing the derived keys,
 *     in seconds.  This limit is only approximately enforced; the CPU
 *     performance is estimated and parameter limits are chosen accordingly.
 * For the encryption functions, the parameters to the scrypt key derivation
 * function are chosen to make the key as strong as possible subject to the
 * specified limits; for the decryption functions, the parameters used are
 * compared to the computed limits and an error is returned if decrypting
 * the data would take too much memory or CPU time.
 */
/**
 * Return codes from scrypt(enc|dec)_(buf|file):
 * 0	success
 * 1	getrlimit or sysctl(hw.usermem) failed
 * 2	clock_getres or clock_gettime failed
 * 3	error computing derived key
 * 4	could not read salt from /dev/urandom
 * 5	error in OpenSSL
 * 6	malloc failed
 * 7	data is not a valid scrypt-encrypted block
 * 8	unrecognized scrypt format
 * 9	decrypting file would take too much memory
 * 10	decrypting file would take too long
 * 11	password is incorrect
 * 12	error writing output file
 * 13	error reading input file
 */

struct scryptdec_file_cookie;

/**
 * scryptenc_buf(inbuf, inbuflen, outbuf, passwd, passwdlen,
 *     maxmem, maxmemfrac, maxtime, verbose):
 * Encrypt inbuflen bytes from inbuf, writing the resulting inbuflen + 128
 * bytes to outbuf.
 */
int scryptenc_buf(const uint8_t *, size_t, uint8_t *,
    const uint8_t *, size_t, size_t, double, double, int);

/**
 * scryptdec_buf(inbuf, inbuflen, outbuf, outlen, passwd, passwdlen,
 *     maxmem, maxmemfrac, maxtime, verbose, force):
 * Decrypt inbuflen bytes from inbuf, writing the result into outbuf and the
 * decrypted data length to outlen.  The allocated length of outbuf must
 * be at least inbuflen.  If ${force} is 1, do not check whether
 * decryption will exceed the estimated available memory or time.
 */
int scryptdec_buf(const uint8_t *, size_t, uint8_t *, size_t *,
    const uint8_t *, size_t, size_t, double, double, int, int);

/**
 * scryptenc_file(infile, outfile, passwd, passwdlen,
 *     maxmem, maxmemfrac, maxtime, verbose):
 * Read a stream from infile and encrypt it, writing the resulting stream to
 * outfile.
 */
int scryptenc_file(FILE *, FILE *, const uint8_t *, size_t,
    size_t, double, double, int);

/*
 * scryptdec_file_printparams(infile):
 * Print the encryption parameters (N, r, p) used for the encrypted ${infile}.
 */
int scryptdec_file_printparams(FILE *);

/**
 * scryptdec_file(infile, outfile, passwd, passwdlen,
 *     maxmem, maxmemfrac, maxtime, verbose, force):
 * Read a stream from infile and decrypt it, writing the resulting stream to
 * outfile.  If ${force} is 1, do not check whether decryption
 * will exceed the estimated available memory or time.
 */
int scryptdec_file(FILE *, FILE *, const uint8_t *, size_t,
    size_t, double, double, int, int);

/**
 * scryptdec_file_prep(infile, passwd, passwdlen, maxmem, maxmemfrac,
 *     maxtime, force, cookie):
 * Prepare to decrypt ${infile}, including checking the passphrase.  Allocate
 * a cookie at ${cookie}.  After calling this function, ${infile} should not
 * be modified until the decryption is completed by scryptdec_file_copy.
 */
int scryptdec_file_prep(FILE *, const uint8_t *, size_t, size_t, double,
    double, int, int, struct scryptdec_file_cookie **);

/**
 * scryptdec_file_copy(cookie, outfile):
 * Read a stream from the file that was passed into the ${cookie} by
 * scryptdec_file_prep, decrypt it, and write the resulting stream to
 * ${outfile}.  After this function completes, it is safe to modify/close
 * ${outfile} and the ${infile} which was given to scryptdec_file_prep.
 */
int scryptdec_file_copy(struct scryptdec_file_cookie *, FILE *);

/**
 * scryptdec_file_cookie_free(cookie):
 * Free the ${cookie}.
 */
void scryptdec_file_cookie_free(struct scryptdec_file_cookie *);

#endif /* !_SCRYPTENC_H_ */
