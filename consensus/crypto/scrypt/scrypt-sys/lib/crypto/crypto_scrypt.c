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
#include "platform.h"

#include <sys/types.h>
#include <sys/mman.h>

#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "cpusupport.h"
#include "sha256.h"
#include "warnp.h"

#include "crypto_scrypt_smix.h"
#include "crypto_scrypt_smix_sse2.h"

#include "crypto_scrypt.h"

static void (*smix_func)(uint8_t *, size_t, uint64_t, void *, void *) = NULL;

/**
 * _crypto_scrypt(passwd, passwdlen, salt, saltlen, N, r, p, buf, buflen, smix):
 * Perform the requested scrypt computation, using ${smix} as the smix routine.
 */
static int
_crypto_scrypt(const uint8_t * passwd, size_t passwdlen,
    const uint8_t * salt, size_t saltlen, uint64_t N, uint32_t _r, uint32_t _p,
    uint8_t * buf, size_t buflen,
    void (*smix)(uint8_t *, size_t, uint64_t, void *, void *))
{
	void * B0, * V0, * XY0;
	uint8_t * B;
	uint32_t * V;
	uint32_t * XY;
	size_t r = _r, p = _p;
	uint32_t i;

	/* Sanity-check parameters. */
	if ((r == 0) || (p == 0)) {
		errno = EINVAL;
		goto err0;
	}
#if SIZE_MAX > UINT32_MAX
	if (buflen > (((uint64_t)(1) << 32) - 1) * 32) {
		errno = EFBIG;
		goto err0;
	}
#endif
	if ((uint64_t)(r) * (uint64_t)(p) >= (1 << 30)) {
		errno = EFBIG;
		goto err0;
	}
	if (((N & (N - 1)) != 0) || (N < 2)) {
		errno = EINVAL;
		goto err0;
	}
	if ((r > SIZE_MAX / 128 / p) ||
#if SIZE_MAX / 256 <= UINT32_MAX
	    (r > (SIZE_MAX - 64) / 256) ||
#endif
	    (N > SIZE_MAX / 128 / r)) {
		errno = ENOMEM;
		goto err0;
	}

	/* Allocate memory. */
#ifdef HAVE_POSIX_MEMALIGN
	if ((errno = posix_memalign(&B0, 64, 128 * r * p)) != 0)
		goto err0;
	B = (uint8_t *)(B0);
	if ((errno = posix_memalign(&XY0, 64, 256 * r + 64)) != 0)
		goto err1;
	XY = (uint32_t *)(XY0);
#if !defined(MAP_ANON) || !defined(HAVE_MMAP)
	if ((errno = posix_memalign(&V0, 64, (size_t)(128 * r * N))) != 0)
		goto err2;
	V = (uint32_t *)(V0);
#endif
#else
	if ((B0 = malloc(128 * r * p + 63)) == NULL)
		goto err0;
	B = (uint8_t *)(((uintptr_t)(B0) + 63) & ~ (uintptr_t)(63));
	if ((XY0 = malloc(256 * r + 64 + 63)) == NULL)
		goto err1;
	XY = (uint32_t *)(((uintptr_t)(XY0) + 63) & ~ (uintptr_t)(63));
#if !defined(MAP_ANON) || !defined(HAVE_MMAP)
	if ((V0 = malloc(128 * r * N + 63)) == NULL)
		goto err2;
	V = (uint32_t *)(((uintptr_t)(V0) + 63) & ~ (uintptr_t)(63));
#endif
#endif
#if defined(MAP_ANON) && defined(HAVE_MMAP)
	if ((V0 = mmap(NULL, (size_t)(128 * r * N), PROT_READ | PROT_WRITE,
#ifdef MAP_NOCORE
	    MAP_ANON | MAP_PRIVATE | MAP_NOCORE,
#else
	    MAP_ANON | MAP_PRIVATE,
#endif
	    -1, 0)) == MAP_FAILED)
		goto err2;
	V = (uint32_t *)(V0);
#endif

	/* 1: (B_0 ... B_{p-1}) <-- PBKDF2(P, S, 1, p * MFLen) */
	PBKDF2_SHA256(passwd, passwdlen, salt, saltlen, 1, B, p * 128 * r);

	/* 2: for i = 0 to p - 1 do */
	for (i = 0; i < p; i++) {
		/* 3: B_i <-- MF(B_i, N) */
		(smix)(&B[i * 128 * r], r, N, V, XY);
	}

	/* 5: DK <-- PBKDF2(P, B, 1, dkLen) */
	PBKDF2_SHA256(passwd, passwdlen, B, p * 128 * r, 1, buf, buflen);

	/* Free memory. */
#if defined(MAP_ANON) && defined(HAVE_MMAP)
	if (munmap(V0, (size_t)(128 * r * N)))
		goto err2;
#else
	free(V0);
#endif
	free(XY0);
	free(B0);

	/* Success! */
	return (0);

err2:
	free(XY0);
err1:
	free(B0);
err0:
	/* Failure! */
	return (-1);
}

#define TESTLEN 64
static struct scrypt_test {
	const char * passwd;
	const char * salt;
	uint64_t N;
	uint32_t r;
	uint32_t p;
	uint8_t result[TESTLEN];
} testcase = {
	.passwd = "pleaseletmein",
	.salt = "SodiumChloride",
	.N = 16,
	.r = 8,
	.p = 1,
	.result = {
		0x25, 0xa9, 0xfa, 0x20, 0x7f, 0x87, 0xca, 0x09,
		0xa4, 0xef, 0x8b, 0x9f, 0x77, 0x7a, 0xca, 0x16,
		0xbe, 0xb7, 0x84, 0xae, 0x18, 0x30, 0xbf, 0xbf,
		0xd3, 0x83, 0x25, 0xaa, 0xbb, 0x93, 0x77, 0xdf,
		0x1b, 0xa7, 0x84, 0xd7, 0x46, 0xea, 0x27, 0x3b,
		0xf5, 0x16, 0xa4, 0x6f, 0xbf, 0xac, 0xf5, 0x11,
		0xc5, 0xbe, 0xba, 0x4c, 0x4a, 0xb3, 0xac, 0xc7,
		0xfa, 0x6f, 0x46, 0x0b, 0x6c, 0x0f, 0x47, 0x7b,
	}
};

static int
testsmix(void (*smix)(uint8_t *, size_t, uint64_t, void *, void *))
{
	uint8_t hbuf[TESTLEN];

	/* Perform the computation. */
	if (_crypto_scrypt(
	    (const uint8_t *)testcase.passwd, strlen(testcase.passwd),
	    (const uint8_t *)testcase.salt, strlen(testcase.salt),
	    testcase.N, testcase.r, testcase.p, hbuf, TESTLEN, smix))
		return (-1);

	/* Does it match? */
	return (memcmp(testcase.result, hbuf, TESTLEN));
}

static void
selectsmix(void)
{

#ifdef CPUSUPPORT_X86_SSE2
	/* If we're running on an SSE2-capable CPU, try that code. */
	if (cpusupport_x86_sse2()) {
		/* If SSE2ized smix works, use it. */
		if (!testsmix(crypto_scrypt_smix_sse2)) {
			smix_func = crypto_scrypt_smix_sse2;
			return;
		}
		warn0("Disabling broken SSE2 scrypt support - please report bug!");
	}
#endif

	/* If generic smix works, use it. */
	if (!testsmix(crypto_scrypt_smix)) {
		smix_func = crypto_scrypt_smix;
		return;
	}
	warn0("Generic scrypt code is broken - please report bug!");

	/* If we get here, something really bad happened. */
	abort();
}

/**
 * crypto_scrypt(passwd, passwdlen, salt, saltlen, N, r, p, buf, buflen):
 * Compute scrypt(passwd[0 .. passwdlen - 1], salt[0 .. saltlen - 1], N, r,
 * p, buflen) and write the result into buf.  The parameters r, p, and buflen
 * must satisfy 0 < r * p < 2^30 and buflen <= (2^32 - 1) * 32.  The parameter
 * N must be a power of 2 greater than 1.
 *
 * Return 0 on success; or -1 on error.
 */
int
crypto_scrypt(const uint8_t * passwd, size_t passwdlen,
    const uint8_t * salt, size_t saltlen, uint64_t N, uint32_t _r, uint32_t _p,
    uint8_t * buf, size_t buflen)
{

	if (smix_func == NULL)
		selectsmix();

	return (_crypto_scrypt(passwd, passwdlen, salt, saltlen, N, _r, _p,
	    buf, buflen, smix_func));
}
