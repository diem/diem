// Cuck(at)oo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp

#include <stdint.h> // for types uint32_t,uint64_t
#include <string.h> // for functions strlen, memset
#include <stdarg.h>
#include <stdio.h> 
#include <chrono>
#include <ctime>
#include "../crypto/blake2.h"
#include "../crypto/siphash.hpp"

// save some keystrokes since i'm a lazy typer
typedef uint32_t u32;
typedef uint64_t u64;

#ifndef MAX_SOLS
#define MAX_SOLS 4
#endif

// proof-of-work parameters
#ifndef EDGEBITS
// the main parameter is the number of bits in an edge index,
// i.e. the 2-log of the number of edges
#define EDGEBITS 31
#endif
#ifndef PROOFSIZE
// the next most important parameter is the (even) length
// of the cycle to be found. a minimum of 12 is recommended
#define PROOFSIZE 42
#endif

#if EDGEBITS > 32
typedef uint64_t word_t;
#elif EDGEBITS > 16
typedef u32 word_t;
#else // if EDGEBITS <= 16
typedef uint16_t word_t;
#endif

// number of nodes in one partition
#define NNODES1 (1ULL << EDGEBITS)
// used to mask siphash output
#define NODEMASK ((word_t)NNODES1 - 1)
#define NODE1MASK NODEMASK
#define EDGEMASK NODEMASK
// number of edges
#define NEDGES NNODES1

// Common Solver parameters, to return to caller
struct SolverParams {
	u32 nthreads = 0;
	u32 ntrims = 0;
	bool showcycle;
	bool allrounds;
	bool mutate_nonce = 1;
	bool cpuload = 1;

	// Common cuda params
	u32 device = 0;

	// Cuda-lean specific params
	u32 blocks = 0;
	u32 tpb = 0;

	// Cuda-mean specific params
	u32 expand = 0;
	u32 genablocks = 0;
	u32 genatpb = 0;
	u32 genbtpb = 0;
	u32 trimtpb = 0;
	u32 tailtpb = 0;
	u32 recoverblocks = 0;
	u32 recovertpb = 0;
};

// Solutions result structs to be instantiated by caller,
// and filled by solver if desired
struct Solution {
 u64 id = 0;
 u64 nonce = 0;
 u64 proof[PROOFSIZE];
};

struct SolverSolutions {
 u32 edge_bits = 0;
 u32 num_sols = 0;
 Solution sols[MAX_SOLS];
};

#define MAX_NAME_LEN 256

// last error reason, to be picked up by stats
// to be returned to caller
char LAST_ERROR_REASON[MAX_NAME_LEN];

// Solver statistics, to be instantiated by caller
// and filled by solver if desired
struct SolverStats {
	u32 device_id = 0;
	u32 edge_bits = 0;
	char plugin_name[MAX_NAME_LEN]; // will be filled in caller-side
	char device_name[MAX_NAME_LEN];
	bool has_errored = false;
	char error_reason[MAX_NAME_LEN];
	u32 iterations = 0;
	u64 last_start_time = 0;
	u64 last_end_time = 0;
	u64 last_solution_time = 0;
};

// generate edge endpoint in cuck(at)oo graph without partition bit
word_t sipnode(siphash_keys *keys, word_t edge, u32 uorv) {
  return keys->siphash24(2*(u64)edge + uorv) & NODEMASK;
}

enum verify_code { POW_OK, POW_HEADER_LENGTH, POW_TOO_BIG, POW_TOO_SMALL, POW_NON_MATCHING, POW_BRANCH, POW_DEAD_END, POW_SHORT_CYCLE};
const char *errstr[] = { "OK", "wrong header length", "edge too big", "edges not ascending", "endpoints don't match up", "branch in cycle", "cycle dead ends", "cycle too short"};

// verify that edges are ascending and form a cycle in header-generated graph
int verify(word_t edges[PROOFSIZE], siphash_keys *keys) {
  word_t uvs[2*PROOFSIZE], xor0, xor1;
  xor0 = xor1 = (PROOFSIZE/2) & 1;

  for (u32 n = 0; n < PROOFSIZE; n++) {
    if (edges[n] > NODEMASK)
      return POW_TOO_BIG;
    if (n && edges[n] <= edges[n-1])
      return POW_TOO_SMALL;
    xor0 ^= uvs[2*n  ] = sipnode(keys, edges[n], 0);
    xor1 ^= uvs[2*n+1] = sipnode(keys, edges[n], 1);
  }
  if (xor0|xor1)              // optional check for obviously bad proofs
    return POW_NON_MATCHING;
  u32 n = 0, i = 0, j;
  do {                        // follow cycle
    for (u32 k = j = i; (k = (k+2) % (2*PROOFSIZE)) != i; ) {
      if (uvs[k]>>1 == uvs[i]>>1) { // find other edge endpoint matching one at i
        if (j != i)           // already found one before
          return POW_BRANCH;
        j = k;
      }
    }
    if (j == i || uvs[j] == uvs[i])
      return POW_DEAD_END;  // no matching endpoint
    i = j^1;
    n++;
  } while (i != 0);           // must cycle back to start or we would have found branch
  return n == PROOFSIZE ? POW_OK : POW_SHORT_CYCLE;
}

// convenience function for extracting siphash keys from header
void setheader(const char *header, const u32 headerlen, siphash_keys *keys) {
  char hdrkey[32];
  // SHA256((unsigned char *)header, headerlen, (unsigned char *)hdrkey);
  blake2b((void *)hdrkey, sizeof(hdrkey), (const void *)header, headerlen, 0, 0);
  keys->setkeys(hdrkey);
}

u64 timestamp() {
	using namespace std::chrono;
	high_resolution_clock::time_point now = high_resolution_clock::now();
	auto dn = now.time_since_epoch();
	return dn.count();
}

/////////////////////////////////////////////////////////////////
// Declarations to make it easier for callers to link as required
/////////////////////////////////////////////////////////////////

#ifndef C_CALL_CONVENTION
#define C_CALL_CONVENTION 0
#endif

// convention to prepend to called functions
#if C_CALL_CONVENTION
#define CALL_CONVENTION extern "C"
#else
#define CALL_CONVENTION
#endif

// Ability to squash printf output at compile time, if desired
#ifndef SQUASH_OUTPUT
#define SQUASH_OUTPUT 0
#endif

void print_log(const char *fmt, ...) {
	if (SQUASH_OUTPUT) return;
	va_list args;
	va_start(args, fmt);
	vprintf(fmt, args);
	va_end(args);
}
//////////////////////////////////////////////////////////////////
// END caller QOL
//////////////////////////////////////////////////////////////////

