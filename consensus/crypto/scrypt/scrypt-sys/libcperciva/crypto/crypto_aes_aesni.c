#include "cpusupport.h"
#ifdef CPUSUPPORT_X86_AESNI

#include <stdint.h>
#include <stdlib.h>
#include <wmmintrin.h>

#include "insecure_memzero.h"
#include "warnp.h"

#include "crypto_aes_aesni.h"

/* Expanded-key structure. */
struct crypto_aes_key_aesni {
	uint8_t rkeys_buf[15 * sizeof(__m128i) + (sizeof(__m128i) - 1)];
	__m128i * rkeys;
	size_t nr;
};

/* Compute an AES-128 round key. */
#define MKRKEY128(rkeys, i, rcon) do {				\
	__m128i _s = rkeys[i - 1];				\
	__m128i _t = rkeys[i - 1];				\
	_s = _mm_xor_si128(_s, _mm_slli_si128(_s, 4));		\
	_s = _mm_xor_si128(_s, _mm_slli_si128(_s, 8));		\
	_t = _mm_aeskeygenassist_si128(_t, rcon);		\
	_t = _mm_shuffle_epi32(_t, 0xff);			\
	rkeys[i] = _mm_xor_si128(_s, _t);			\
} while (0)

/**
 * crypto_aes_key_expand_128_aesni(key, rkeys):
 * Expand the 128-bit AES key ${key} into the 11 round keys ${rkeys}.  This
 * implementation uses x86 AESNI instructions, and should only be used if
 * CPUSUPPORT_X86_AESNI is defined and cpusupport_x86_aesni() returns nonzero.
 */
static void
crypto_aes_key_expand_128_aesni(const uint8_t key[16], __m128i rkeys[11])
{

	/* The first round key is just the key. */
	/**
	 * XXX Compiler breakage:
	 * The intrinsic defined by Intel for _mm_loadu_si128 defines it as
	 * taking a (const __m128i *) parameter.  This forces us to write a
	 * bug: The cast to (const __m128i *) is invalid since it increases
	 * the alignment requirement of the pointer.  Alas, until compilers
	 * get fixed intrinsics, all we can do is code the bug and require
	 * that alignment-requirement-increasing compiler warnings get
	 * disabled.
	 */
	rkeys[0] = _mm_loadu_si128((const __m128i *)&key[0]);

	/*
	 * Each of the remaining round keys are computed from the preceding
	 * round key: rotword+subword+rcon (provided as aeskeygenassist) to
	 * compute the 'temp' value, then xor with 1, 2, 3, or all 4 of the
	 * 32-bit words from the preceding round key.  Unfortunately, 'rcon'
	 * is encoded as an immediate value, so we need to write the loop out
	 * ourselves rather than allowing the compiler to expand it.
	 */
	MKRKEY128(rkeys, 1, 0x01);
	MKRKEY128(rkeys, 2, 0x02);
	MKRKEY128(rkeys, 3, 0x04);
	MKRKEY128(rkeys, 4, 0x08);
	MKRKEY128(rkeys, 5, 0x10);
	MKRKEY128(rkeys, 6, 0x20);
	MKRKEY128(rkeys, 7, 0x40);
	MKRKEY128(rkeys, 8, 0x80);
	MKRKEY128(rkeys, 9, 0x1b);
	MKRKEY128(rkeys, 10, 0x36);
}

/* Compute an AES-256 round key. */
#define MKRKEY256(rkeys, i, shuffle, rcon)	do {		\
	__m128i _s = rkeys[i - 2];				\
	__m128i _t = rkeys[i - 1];				\
	_s = _mm_xor_si128(_s, _mm_slli_si128(_s, 4));		\
	_s = _mm_xor_si128(_s, _mm_slli_si128(_s, 8));		\
	_t = _mm_aeskeygenassist_si128(_t, rcon);		\
	_t = _mm_shuffle_epi32(_t, shuffle);			\
	rkeys[i] = _mm_xor_si128(_s, _t);			\
} while (0)

/**
 * crypto_aes_key_expand_256_aesni(key, rkeys):
 * Expand the 256-bit AES key ${key} into the 15 round keys ${rkeys}.  This
 * implementation uses x86 AESNI instructions, and should only be used if
 * CPUSUPPORT_X86_AESNI is defined and cpusupport_x86_aesni() returns nonzero.
 */
static void
crypto_aes_key_expand_256_aesni(const uint8_t key[32], __m128i rkeys[15])
{

	/* The first two round keys are just the key. */
	/**
	 * XXX Compiler breakage:
	 * The intrinsic defined by Intel for _mm_loadu_si128 defines it as
	 * taking a (const __m128i *) parameter.  This forces us to write a
	 * bug: The cast to (const __m128i *) is invalid since it increases
	 * the alignment requirement of the pointer.  Alas, until compilers
	 * get fixed intrinsics, all we can do is code the bug and require
	 * that alignment-requirement-increasing compiler warnings get
	 * disabled.
	 */
	rkeys[0] = _mm_loadu_si128((const __m128i *)&key[0]);
	rkeys[1] = _mm_loadu_si128((const __m128i *)&key[16]);

	/*
	 * Each of the remaining round keys are computed from the preceding
	 * pair of keys.  Even rounds use rotword+subword+rcon, while odd
	 * rounds just use subword; the aeskeygenassist instruction computes
	 * both, and we use 0xff or 0xaa to select the one we need.  The rcon
	 * value used is irrelevant for odd rounds since we ignore the value
	 * which it feeds into.  Unfortunately, the 'shuffle' and 'rcon'
	 * values are encoded into the instructions as immediates, so we need
	 * to write the loop out ourselves rather than allowing the compiler
	 * to expand it.
	 */
	MKRKEY256(rkeys, 2, 0xff, 0x01);
	MKRKEY256(rkeys, 3, 0xaa, 0x00);
	MKRKEY256(rkeys, 4, 0xff, 0x02);
	MKRKEY256(rkeys, 5, 0xaa, 0x00);
	MKRKEY256(rkeys, 6, 0xff, 0x04);
	MKRKEY256(rkeys, 7, 0xaa, 0x00);
	MKRKEY256(rkeys, 8, 0xff, 0x08);
	MKRKEY256(rkeys, 9, 0xaa, 0x00);
	MKRKEY256(rkeys, 10, 0xff, 0x10);
	MKRKEY256(rkeys, 11, 0xaa, 0x00);
	MKRKEY256(rkeys, 12, 0xff, 0x20);
	MKRKEY256(rkeys, 13, 0xaa, 0x00);
	MKRKEY256(rkeys, 14, 0xff, 0x40);
}

/**
 * crypto_aes_key_expand_aesni(key, len):
 * Expand the ${len}-byte AES key ${key} into a structure which can be passed
 * to crypto_aes_encrypt_block_aesni.  The length must be 16 or 32.  This
 * implementation uses x86 AESNI instructions, and should only be used if
 * CPUSUPPORT_X86_AESNI is defined and cpusupport_x86_aesni() returns nonzero.
 */
void *
crypto_aes_key_expand_aesni(const uint8_t * key, size_t len)
{
	struct crypto_aes_key_aesni * kexp;
	size_t rkey_offset;

	/* Allocate structure. */
	if ((kexp = malloc(sizeof(struct crypto_aes_key_aesni))) == NULL)
		goto err0;

	/* Figure out where to put the round keys. */
	rkey_offset = (uintptr_t)(&kexp->rkeys_buf[0]) % sizeof(__m128i);
	rkey_offset = (sizeof(__m128i) - rkey_offset) % sizeof(__m128i);
	kexp->rkeys = (void *)&kexp->rkeys_buf[rkey_offset];

	/* Compute round keys. */
	if (len == 16) {
		kexp->nr = 10;
		crypto_aes_key_expand_128_aesni(key, kexp->rkeys);
	} else if (len == 32) {
		kexp->nr = 14;
		crypto_aes_key_expand_256_aesni(key, kexp->rkeys);
	} else {
		warn0("Unsupported AES key length: %zu bytes", len);
		goto err1;
	}

	/* Success! */
	return (kexp);

err1:
	free(kexp);
err0:
	/* Failure! */
	return (NULL);
}

/**
 * crypto_aes_encrypt_block_aesni(in, out, key):
 * Using the expanded AES key ${key}, encrypt the block ${in} and write the
 * resulting ciphertext to ${out}.  This implementation uses x86 AESNI
 * instructions, and should only be used if CPUSUPPORT_X86_AESNI is defined
 * and cpusupport_x86_aesni() returns nonzero.
 */
void
crypto_aes_encrypt_block_aesni(const uint8_t * in, uint8_t * out,
    const void * key)
{
	const struct crypto_aes_key_aesni * _key = key;
	const __m128i * aes_key = _key->rkeys;
	__m128i aes_state;
	size_t nr = _key->nr;

	aes_state = _mm_loadu_si128((const __m128i *)in);
	aes_state = _mm_xor_si128(aes_state, aes_key[0]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[1]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[2]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[3]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[4]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[5]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[6]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[7]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[8]);
	aes_state = _mm_aesenc_si128(aes_state, aes_key[9]);
	if (nr > 10) {
		aes_state = _mm_aesenc_si128(aes_state, aes_key[10]);
		aes_state = _mm_aesenc_si128(aes_state, aes_key[11]);

		if (nr > 12) {
			aes_state = _mm_aesenc_si128(aes_state, aes_key[12]);
			aes_state = _mm_aesenc_si128(aes_state, aes_key[13]);
		}
	}

	aes_state = _mm_aesenclast_si128(aes_state, aes_key[nr]);
	_mm_storeu_si128((__m128i *)out, aes_state);
}

/**
 * crypto_aes_key_free_aesni(key):
 * Free the expanded AES key ${key}.
 */
void
crypto_aes_key_free_aesni(void * key)
{

	/* Behave consistently with free(NULL). */
	if (key == NULL)
		return;

	/* Attempt to zero the expanded key. */
	insecure_memzero(key, sizeof(struct crypto_aes_key_aesni));

	/* Free the key. */
	free(key);
}

#endif /* CPUSUPPORT_X86_AESNI */
