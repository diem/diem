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
 */
#include "platform.h"

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "getopt.h"
#include "humansize.h"
#include "insecure_memzero.h"
#include "parsenum.h"
#include "readpass.h"
#include "scryptenc.h"
#include "warnp.h"

static void
usage(void)
{

	fprintf(stderr,
	    "usage: scrypt {enc | dec | info} [-f] [-M maxmem]"
	    " [-m maxmemfrac]\n"
	    "              [-t maxtime] [-v] [-P] infile [outfile]\n"
	    "       scrypt --version\n");
	exit(1);
}

int
main(int argc, char *argv[])
{
	FILE * infile;
	FILE * outfile = stdout;
	int devtty = 1;
	int dec = 0;
	int info = 0;
	size_t maxmem = 0;
	int force_resources = 0;
	uint64_t maxmem64;
	double maxmemfrac = 0.5;
	double maxtime = 300.0;
	const char * ch;
	const char * infilename;
	const char * outfilename;
	char * passwd;
	int rc;
	int verbose = 0;
	struct scryptdec_file_cookie * C = NULL;

	WARNP_INIT;

	/* We should have "enc", "dec", or "info" first. */
	if (argc < 2)
		usage();
	if (strcmp(argv[1], "enc") == 0) {
		maxmem = 0;
		maxmemfrac = 0.125;
		maxtime = 5.0;
	} else if (strcmp(argv[1], "dec") == 0) {
		dec = 1;
	} else if (strcmp(argv[1], "info") == 0) {
		info = 1;
	} else if (strcmp(argv[1], "--version") == 0) {
		fprintf(stdout, "scrypt %s\n", PACKAGE_VERSION);
		exit(0);
	} else {
		warn0("First argument must be 'enc', 'dec', or 'info'.\n");
		usage();
	}
	argc--;
	argv++;

	/* Parse arguments. */
	while ((ch = GETOPT(argc, argv)) != NULL) {
		GETOPT_SWITCH(ch) {
		GETOPT_OPT("-f"):
			force_resources = 1;
			break;
		GETOPT_OPTARG("-M"):
			if (humansize_parse(optarg, &maxmem64)) {
				warn0("Could not parse the parameter to -M.");
				exit(1);
			}
			if (maxmem64 > SIZE_MAX) {
				warn0("The parameter to -M is too large.");
				exit(1);
			}
			maxmem = (size_t)maxmem64;
			break;
		GETOPT_OPTARG("-m"):
			if (PARSENUM(&maxmemfrac, optarg, 0, 1)) {
				warnp("Invalid option: -m %s", optarg);
				exit(1);
			}
			break;
		GETOPT_OPTARG("-t"):
			if (PARSENUM(&maxtime, optarg, 0, INFINITY)) {
				warnp("Invalid option: -t %s", optarg);
				exit(1);
			}
			break;
		GETOPT_OPT("-v"):
			verbose = 1;
			break;
		GETOPT_OPT("-P"):
			devtty = 0;
			break;
		GETOPT_MISSING_ARG:
			warn0("Missing argument to %s\n", ch);
			usage();
		GETOPT_DEFAULT:
			warn0("illegal option -- %s\n", ch);
			usage();
		}
	}
	argc -= optind;
	argv += optind;

	/* We must have one or two parameters left. */
	if ((argc < 1) || (argc > 2))
		usage();

	/* Set the input filename. */
	if (strcmp(argv[0], "-"))
		infilename = argv[0];
	else
		infilename = NULL;

	/* Set the output filename. */
	if (argc > 1)
		outfilename = argv[1];
	else
		outfilename = NULL;

	/* If the input isn't stdin, open the file. */
	if (infilename != NULL) {
		if ((infile = fopen(infilename, "rb")) == NULL) {
			warnp("Cannot open input file: %s", infilename);
			goto err0;
		}
	} else {
		infile = stdin;

		/* Error if given incompatible options. */
		if (devtty == 0) {
			warn0("Cannot read both passphrase and input file"
			    " from standard input");
			goto err0;
		}
	}

	/* User selected 'info' mode. */
	if (info) {
		/* Print the encryption parameters used for the file. */
		rc = scryptdec_file_printparams(infile);

		/* Clean up. */
		if (infile != stdin)
			fclose(infile);

		/* Finished! */
		goto done;
	}

	/* Prompt for a password. */
	if (readpass(&passwd, "Please enter passphrase",
	    (dec || !devtty) ? NULL : "Please confirm passphrase", devtty))
		goto err1;

	/*-
	 * If we're decrypting, open the input file and process its header;
	 * doing this here allows us to abort without creating an output
	 * file if the input file does not have a valid scrypt header or if
	 * we have the wrong passphrase.
	 *
	 * If successful, we get back a cookie containing the decryption
	 * parameters (which we'll use after we open the output file).
	 */
	if (dec) {
		if ((rc = scryptdec_file_prep(infile, (uint8_t *)passwd,
		    strlen(passwd), maxmem, maxmemfrac, maxtime, verbose,
		    force_resources, &C)) != 0) {
			goto cleanup;
		}
	}

	/* If we have an output file, open it. */
	if (outfilename != NULL) {
		if ((outfile = fopen(outfilename, "wb")) == NULL) {
			warnp("Cannot open output file: %s", outfilename);
			goto err2;
		}
	}

	/* Encrypt or decrypt. */
	if (dec)
		rc = scryptdec_file_copy(C, outfile);
	else
		rc = scryptenc_file(infile, outfile, (uint8_t *)passwd,
		    strlen(passwd), maxmem, maxmemfrac, maxtime, verbose);

cleanup:
	/* Free the decryption cookie, if any. */
	scryptdec_file_cookie_free(C);

	/* Zero and free the password. */
	insecure_memzero(passwd, strlen(passwd));
	free(passwd);

	/* Close any files we opened. */
	if (infile != stdin)
		fclose(infile);
	if (outfile != stdout)
		fclose(outfile);

done:
	/* If we failed, print the right error message and exit. */
	if (rc != 0) {
		switch (rc) {
		case 1:
			warnp("Error determining amount of available memory");
			break;
		case 2:
			warnp("Error reading clocks");
			break;
		case 3:
			warnp("Error computing derived key");
			break;
		case 4:
			warnp("Error reading salt");
			break;
		case 5:
			warnp("OpenSSL error");
			break;
		case 6:
			warnp("Error allocating memory");
			break;
		case 7:
			warn0("Input is not valid scrypt-encrypted block");
			break;
		case 8:
			warn0("Unrecognized scrypt format version");
			break;
		case 9:
			warn0("Decrypting file would require too much memory");
			break;
		case 10:
			warn0("Decrypting file would take too much CPU time");
			break;
		case 11:
			warn0("Passphrase is incorrect");
			break;
		case 12:
			warnp("Error writing file: %s",
			    (outfilename != NULL) ? outfilename
			    : "standard output");
			break;
		case 13:
			warnp("Error reading file: %s",
			    (infilename != NULL) ? infilename
			    : "standard input");
			break;
		}
		goto err0;
	}

	/* Success! */
	return (0);

err2:
	scryptdec_file_cookie_free(C);
	insecure_memzero(passwd, strlen(passwd));
	free(passwd);
err1:
	if (infile != stdin)
		fclose(infile);
err0:
	/* Failure! */
	exit(1);
}
