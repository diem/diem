#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "getopt.h"

/*
 * Standard getopt global variables.  optreset starts as non-zero in order to
 * trigger initialization behaviour.
 */
const char * optarg = NULL;
int optind = 1;
int opterr = 1;
int optreset = 1;

/*
 * Quasi-internal global variables -- these are used via GETOPT macros.
 */
const char * getopt_dummy = "(dummy)";
int getopt_initialized = 0;

/*
 * Internal variables.
 */
static const char * cmdname = NULL;
static struct opt {
	const char * os;
	size_t olen;
	int hasarg;
} * opts = NULL;
static size_t nopts;
static size_t opt_missing;
static size_t opt_default;
static size_t opt_found;
static const char * packedopts;
static char popt[3];
static int atexit_registered = 0;

/* Print a message. */
#define PRINTMSG(...)	do {						\
	if (cmdname != NULL)						\
		fprintf(stderr, "%s: ", cmdname);			\
	fprintf(stderr, __VA_ARGS__);					\
	fprintf(stderr, "\n");						\
} while (0)

/* Print an error message and die. */
#define DIE(...)	do {						\
	PRINTMSG(__VA_ARGS__);						\
	abort();							\
} while (0)

/* Print a warning, if warnings are enabled. */
#define WARN(...)	do {						\
	if (opterr == 0)						\
		break;							\
	if (opt_missing != opt_default)					\
		break;							\
	PRINTMSG(__VA_ARGS__);						\
} while (0)

/* Free allocated options array. */
static void
atexit_handler(void)
{

	free(opts);
	opts = NULL;
}

/* Reset internal state. */
static void
reset(int argc, char * const argv[])
{
	const char * p;

	/* If we have arguments, stash argv[0] for error messages. */
	if (argc > 0) {
		/* Find the basename, without leading directories. */
		for (p = cmdname = argv[0]; *p != '\0'; p++) {
			if (*p == '/')
				cmdname = p + 1;
		}
	}

	/* Discard any registered command-line options. */
	free(opts);
	opts = NULL;

	/* Register atexit handler if we haven't done so already. */
	if (!atexit_registered) {
		atexit(atexit_handler);
		atexit_registered = 1;
	}

	/* We will start scanning from the first option. */
	optind = 1;

	/* We're not in the middle of any packed options. */
	packedopts = NULL;

	/* We haven't found any option yet. */
	opt_found = (size_t)(-1);

	/* We're not initialized yet. */
	getopt_initialized = 0;

	/* Finished resetting state. */
	optreset = 0;
}

/* Search for an option string. */
static size_t
searchopt(const char * os)
{
	size_t i;

	/* Scan the array of options. */
	for (i = 0; i < nopts; i++) {
		/* Is there an option in this slot? */
		if (opts[i].os == NULL)
			continue;

		/* Does this match up to the length of the option string? */
		if (strncmp(opts[i].os, os, opts[i].olen))
			continue;

		/* Do we have <option>\0 or <option>= ? */
		if ((os[opts[i].olen] == '\0') || (os[opts[i].olen] == '='))
			return (i);
	}

	/* Not found. */
	return (opt_default);
}

const char *
getopt(int argc, char * const argv[])
{
	const char * os = NULL;
	const char * canonical_os = NULL;

	/* No argument yet. */
	optarg = NULL;

	/* Reset the getopt state if needed. */
	if (optreset)
		reset(argc, argv);

	/* If not initialized, return dummy option. */
	if (!getopt_initialized)
		return (GETOPT_DUMMY);

	/* If we've run out of arguments, we're done. */
	if (optind >= argc)
		return (NULL);

	/*
	 * If we're not already in the middle of a packed single-character
	 * options, see if we should start.
	 */
	if ((packedopts == NULL) && (argv[optind][0] == '-') &&
	    (argv[optind][1] != '-') && (argv[optind][1] != '\0')) {
		/* We have one or more single-character options. */
		packedopts = &argv[optind][1];
	}

	/* If we're processing single-character options, fish one out. */
	if (packedopts != NULL) {
		/* Construct the option string. */
		popt[0] = '-';
		popt[1] = *packedopts;
		popt[2] = '\0';
		os = popt;

		/* We've done this character. */
		packedopts++;

		/* Are we done with this string? */
		if (*packedopts == '\0') {
			packedopts = NULL;
			optind++;
		}
	}

	/* If we don't have an option yet, do we have dash-dash? */
	if ((os == NULL) && (argv[optind][0] == '-') &&
	    (argv[optind][1] == '-')) {
		/* If this is not "--\0", it's an option. */
		if (argv[optind][2] != '\0')
			os = argv[optind];

		/* Either way, we want to eat the string. */
		optind++;
	}

	/* If we have found nothing which looks like an option, we're done. */
	if (os == NULL)
		return (NULL);

	/* Search for the potential option. */
	opt_found = searchopt(os);

	/* If the option is not registered, give up now. */
	if (opt_found == opt_default) {
		WARN("unknown option: %s", os);
		return (os);
	}

	/* The canonical option string is the one registered. */
	canonical_os = opts[opt_found].os;

	/* Does the option take an argument? */
	if (opts[opt_found].hasarg) {
		/*
		 * If we're processing packed single-character options, the
		 * rest of the string is the argument to this option.
		 */
		if (packedopts != NULL) {
			optarg = packedopts;
			packedopts = NULL;
			optind++;
		}

		/*
		 * If the option string is <option>=<value>, extract that
		 * value as the option argument.
		 */
		if (os[opts[opt_found].olen] == '=')
			optarg = &os[opts[opt_found].olen + 1];

		/*
		 * If we don't have an argument yet, take one from the
		 * remaining command line.
		 */
		if ((optarg == NULL) && (optind < argc))
			optarg = argv[optind++];

		/* If we still have no option, declare it MIA. */
		if (optarg == NULL) {
			WARN("option requires an argument: %s",
			    opts[opt_found].os);
			opt_found = opt_missing;
		}
	} else {
		/* If we have --foo=bar, something went wrong. */
		if (os[opts[opt_found].olen] == '=') {
			WARN("option doesn't take an argument: %s",
			    opts[opt_found].os);
			opt_found = opt_default;
		}
	}

	/* Return the canonical option string. */
	return (canonical_os);
}

size_t
getopt_lookup(const char * os)
{

	/*
	 * We only take this parameter so that we can assert that it's the
	 * same option string as we returned from getopt(); as such, it is
	 * unused in the absence of assertions.
	 */
	(void)os; /* UNUSED */

	/* Can't reset here. */
	if (optreset)
		DIE("Can't reset in the middle of getopt loop");

	/* We should only be called after initialization is complete. */
	assert(getopt_initialized);

	/* GETOPT_DUMMY should never get passed back to us. */
	assert(os != GETOPT_DUMMY);

	/*
	 * Make sure the option passed back to us corresponds to the one we
	 * found earlier.
	 */
	assert((opt_found == opt_missing) || (opt_found == opt_default) ||
	    ((opt_found < nopts) && (strcmp(os, opts[opt_found].os) == 0)));

	/* Return the option number we identified earlier. */
	return (opt_found);
}

void
getopt_register_opt(const char * os, size_t ln, int hasarg)
{

	/* Can't reset here. */
	if (optreset)
		DIE("Can't reset in the middle of getopt loop");

	/* We should only be called during initialization. */
	assert(!getopt_initialized);

	/* We should have space allocated for registering options. */
	assert(opts != NULL);

	/* We should not have registered an option here yet. */
	assert(opts[ln].os == NULL);

	/* Options should be "-X" or "--foo". */
	if ((os[0] != '-') || (os[1] == '\0') ||
	    ((os[1] == '-') && (os[2] == '\0')) ||
	    ((os[1] != '-') && (os[2] != '\0')))
		DIE("Not a valid command-line option: %s", os);

	/* Make sure we haven't already registered this option. */
	if (searchopt(os) != opt_default)
		DIE("Command-line option registered twice: %s", os);

	/* Record option. */
	opts[ln].os = os;
	opts[ln].olen = strlen(os);
	opts[ln].hasarg = hasarg;
}

void
getopt_register_missing(size_t ln)
{

	/* Can't reset here. */
	if (optreset)
		DIE("Can't reset in the middle of getopt loop");

	/* We should only be called during initialization. */
	assert(!getopt_initialized);

	/* Record missing-argument value. */
	opt_missing = ln;
}

void
getopt_setrange(size_t ln)
{
	size_t i;

	/* Can't reset here. */
	if (optreset)
		DIE("Can't reset in the middle of getopt loop");

	/* We should only be called during initialization. */
	assert(!getopt_initialized);

	/* Allocate space for options. */
	opts = malloc(ln * sizeof(struct opt));
	if ((ln > 0) && (opts == NULL))
		DIE("Failed to allocate memory in getopt");

	/* Initialize options. */
	for (i = 0; i < ln; i++)
		opts[i].os = NULL;

	/* Record the number of (potential) options. */
	nopts = ln;

	/* Record default missing-argument and no-such-option values. */
	opt_missing = opt_default = ln + 1;
}
