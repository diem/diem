// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

#include "cuckoo.h"

// assume EDGEBITS < 31
#define NNODES (2 * NEDGES)
#define NCUCKOO NNODES


#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <unistd.h>
#include "cyclebase.hpp"
#include <set>

typedef unsigned char u8;

class cuckoo_ctx {
public:
  siphash_keys sip_keys;
  word_t easiness;
  cyclebase cb;

  cuckoo_ctx(const char* header, const u32 headerlen, const u32 nonce, word_t easy_ness) {
    easiness = easy_ness;
    cb.alloc();
    assert(cb.cuckoo != 0);
  }

  ~cuckoo_ctx() {
    cb.freemem();
  }

  u64 bytes() {
    return (word_t)(1+NNODES) * sizeof(word_t);
  }

  void setheadernonce(char* const headernonce, const u32 len, const u32 nonce) {
    ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    setheader(headernonce, len, &sip_keys);
    cb.reset();
  }

  void cycle_base() {
    for (word_t nonce = 0; nonce < easiness; nonce++) {
      word_t u = sipnode(&sip_keys, nonce, 0);
      word_t v = sipnode(&sip_keys, nonce, 1);
  #ifdef SHOW
      for (unsigned j=1; j<NNODES; j++)
        if (!cb.cuckoo[j]) printf("%2d:   ",j);
        else               printf("%2d:%02d ",j,cb.cuckoo[j]);
      printf(" %x (%d,%d)\n", nonce,2*u,2*v+1);
  #endif
      cb.addedge(u, v);
    }
#ifdef CCSIZE1000
    u32 nlarge = 0;
    for (u32 i=0; i<NNODES; i++) {
      int size = -cb.cuckoo[i];
      if (size >= 1000)
        nlarge += size;
    }
    printf("%u nodes in ccsize >= 1000\n", nlarge);
#endif
  }
};

// arbitrary length of header hashed into siphash key
#define HEADERLEN 80

int main(int argc, char **argv) {
  char header[HEADERLEN];
  memset(header, 0, HEADERLEN);
  int c, easipct = 50;
  u32 nonce = 0;
  u32 range = 1;
  u64 time0, time1;
  u32 timems;

  while ((c = getopt (argc, argv, "e:h:n:r:")) != -1) {
    switch (c) {
      case 'e':
        easipct = atoi(optarg);
        break;
      case 'h':
        memcpy(header, optarg, strlen(optarg));
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
    }
  }
  assert(easipct >= 0 && easipct <= 100);
  printf("Looking for %d-cycle on cuckoo%d(\"%s\",%d", PROOFSIZE, EDGEBITS+1, header, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with %d%% edges, ", easipct);
  word_t easiness = easipct * (word_t)NNODES / 100;
  cuckoo_ctx ctx(header, sizeof(header), nonce, easiness);
  u64 bytes = ctx.bytes();
  int unit;
  for (unit=0; bytes >= 10240; bytes>>=10,unit++) ;
  printf("using %d%cB memory at %llx.\n", (u32)bytes, " KMGT"[unit], (uint64_t)ctx.cb.cuckoo);

  for (u32 r = 0; r < range; r++) {
    time0 = timestamp();
    ctx.setheadernonce(header, sizeof(header), nonce + r);
    printf("nonce %d k0 k1 k2 k3 %llx %llx %llx %llx\n", nonce+r, ctx.sip_keys.k0, ctx.sip_keys.k1, ctx.sip_keys.k2, ctx.sip_keys.k3);
    ctx.cycle_base();
    time1 = timestamp(); timems = (time1 - time0) / 1000000;
    printf("Time: %d ms\n", timems);
  }
}
