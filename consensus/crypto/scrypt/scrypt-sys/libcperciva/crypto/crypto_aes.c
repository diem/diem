#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include <openssl/aes.h>

#include "cpusupport.h"
#include "crypto_aes_aesni.h"
#include "insecure_memzero.h"
#include "warnp.h"

#include "crypto_aes.h"

/**
 * This represents either an AES_KEY or a struct crypto_aes_key_aesni; we
 * know which it is based on whether we're using AESNI code or not.  As such,
 * it's just an opaque pointer; but declaring it as a named structure type
 * prevents type-mismatch bugs in upstream code.
 */
struct crypto_aes_key;

#ifdef CPUSUPPORT_X86_AESNI
/* Test whether OpenSSL and AESNI code produce the same AES ciphertext. */
static int
aesnitest(uint8_t ptext[16], uint8_t * key, size_t len)
{
	AES_KEY kexp_openssl;
	void * kexp_aesni;
	uint8_t ctext_openssl[16];
	uint8_t ctext_aesni[16];

	/* Sanity-check. */
	assert((len == 16) || (len == 32));

	/* Expand the key. */
	AES_set_encrypt_key(key, (int)(len * 8), &kexp_openssl);
	if ((kexp_aesni = crypto_aes_key_expand_aesni(key, len)) == NULL)
		goto err0;

	/* Encrypt the block. */
	AES_encrypt(ptext, ctext_openssl, &kexp_openssl);
	crypto_aes_encrypt_block_aesni(ptext, ctext_aesni, kexp_aesni);

	/* Free the AESNI expanded key. */
	crypto_aes_key_free_aesni(kexp_aesni);

	/* Do the outputs match? */
	return (memcmp(ctext_openssl, ctext_aesni, 16));

err0:
	/* Failure! */
	return (-1);
}

/* Should we use AESNI? */
static int
useaesni(void)
{
	static int aesnigood = -1;
	uint8_t key[32];
	uint8_t ptext[16];
	size_t i;

	/* If we haven't decided which code to use yet, decide now. */
	while (aesnigood == -1) {
		/* Default to OpenSSL. */
		aesnigood = 0;

		/* If the CPU doesn't claim to support AESNI, stop here. */
		if (!cpusupport_x86_aesni())
			break;

		/* Test cases: key is 0x00010203..., ptext is 0x00112233... */
		for (i = 0; i < 16; i++)
			ptext[i] = (0x11 * i) & 0xff;
		for (i = 0; i < 32; i++)
			key[i] = i & 0xff;

		/* Test that AESNI and OpenSSL produce the same results. */
		if (aesnitest(ptext, key, 16) || aesnitest(ptext, key, 32)) {
			warn0("Disabling AESNI due to failed self-test");
			break;
		}

		/* AESNI works; use it. */
		aesnigood = 1;
	}

	return (aesnigood);
}
#endif /* CPUSUPPORT_X86_AESNI */

/**
 * crypto_aes_key_expand(key, len):
 * Expand the ${len}-byte AES key ${key} into a structure which can be passed
 * to crypto_aes_encrypt_block.  The length must be 16 or 32.
 */
struct crypto_aes_key *
crypto_aes_key_expand(const uint8_t * key, size_t len)
{
	AES_KEY * kexp;

	/* Sanity-check. */
	assert((len == 16) || (len == 32));

#ifdef CPUSUPPORT_X86_AESNI
	/* Use AESNI if we can. */
	if (useaesni())
		return (crypto_aes_key_expand_aesni(key, len));
#endif

	/* Allocate structure. */
	if ((kexp = malloc(sizeof(AES_KEY))) == NULL)
		goto err0;

	/* Expand the key. */
	AES_set_encrypt_key(key, (int)(len * 8), kexp);

	/* Success! */
	return ((void *)kexp);

err0:
	/* Failure! */
	return (NULL);
}

/**
 * crypto_aes_encrypt_block(in, out, key):
 * Using the expanded AES key ${key}, encrypt the block ${in} and write the
 * resulting ciphertext to ${out}.
 */
void
crypto_aes_encrypt_block(const uint8_t * in, uint8_t * out,
    const struct crypto_aes_key * key)
{

#ifdef CPUSUPPORT_X86_AESNI
	if (useaesni()) {
		crypto_aes_encrypt_block_aesni(in, out, (const void *)key);
		return;
	}
#endif

	/* Get AES to do the work. */
	AES_encrypt(in, out, (const void *)key);
}

/**
 * crypto_aes_key_free(key):
 * Free the expanded AES key ${key}.
 */
void
crypto_aes_key_free(struct crypto_aes_key * key)
{

#ifdef CPUSUPPORT_X86_AESNI
	if (useaesni()) {
		crypto_aes_key_free_aesni((void *)key);
		return;
	}
#endif

	/* Behave consistently with free(NULL). */
	if (key == NULL)
		return;

	/* Attempt to zero the expanded key. */
	insecure_memzero(key, sizeof(AES_KEY));

	/* Free the key. */
	free(key);
}
