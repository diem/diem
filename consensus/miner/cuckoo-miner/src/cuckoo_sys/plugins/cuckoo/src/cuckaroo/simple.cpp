// Cuck(at)oo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp

#include "cuckaroo.hpp"
#include "graph.hpp"
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <unistd.h>
#include <set>

#define NNODES (2*NEDGES)
#ifndef MAXSOLS
#define MAXSOLS 4
#endif

typedef unsigned char u8;

class cuckoo_ctx {
public:
  siphash_keys sip_keys;
  word_t easiness;
  graph<word_t> cg;

  cuckoo_ctx(const char* header, const u32 headerlen, const u32 nonce, word_t easy_ness) : cg(NEDGES, NEDGES, MAXSOLS) {
    easiness = easy_ness;
  }

  ~cuckoo_ctx() { }

  u64 bytes() {
    return cg.bytes();
  }

  void setheadernonce(char* const headernonce, const u32 len, const u32 nonce) {
    ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    setheader(headernonce, len, &sip_keys);
    cg.reset();
  }

  void find_cycles() {
    u64 sips[EDGE_BLOCK_SIZE];
    for (word_t block = 0; block < easiness; block += EDGE_BLOCK_SIZE) {
      sipblock(sip_keys, block, sips);
      for (u32 i = 0; i < EDGE_BLOCK_SIZE; i++) {
        u64 edge = sips[i];
        word_t u = edge & EDGEMASK;
        word_t v = (edge >> 32) & EDGEMASK;
        cg.add_edge(u, v);
#ifdef SHOW
        word_t nonce = block + i;
        printf("%d add (%d,%d)\n", nonce,u,v+NEDGES);
        for (unsigned j=0; j<NNODES; j++) {
          printf("\t%d",j);
          for (int a=cg.adjlist[j]; a!=graph<word_t>::NIL; a=cg.links[a].next) printf(":%d", cg.links[a^1].to);
          if ((j+1)%NEDGES == 0)
          printf("\n");
        }
#endif
      }
    }
    for (u32 s=0; s < cg.nsols; s++) {
      printf("Solution");
      // qsort(&cg.sols[s], PROOFSIZE, sizeof(word_t), cg.nonce_cmp);
      for (u32 j=0; j < PROOFSIZE; j++) {
        word_t nonce = cg.sols[s][j];
        // u64 edge = sipblock(sip_keys, nonce, sips);
        // printf(" (%x,%x)", edge & EDGEMASK, (edge >> 32) & EDGEMASK);
        printf(" %x", nonce);
      }
      printf("\n");
      int pow_rc = verify(cg.sols[s], sip_keys);
      if (pow_rc == POW_OK) {
        printf("Verified with cyclehash ");
        unsigned char cyclehash[32];
        blake2b((void *)cyclehash, sizeof(cyclehash), (const void *)cg.sols[s], sizeof(cg.sols[0]), 0, 0);
        for (int i=0; i<32; i++)
          printf("%02x", cyclehash[i]);
        printf("\n");
      } else {
        printf("FAILED due to %s\n", errstr[pow_rc]);
      }

    }
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
  printf("Looking for %d-cycle on cuckaroo%d(\"%s\",%d", PROOFSIZE, EDGEBITS, header, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with %d%% edges, ", easipct);
  word_t easiness = easipct * (uint64_t)NNODES / 100;
  cuckoo_ctx ctx(header, sizeof(header), nonce, easiness);
  u64 bytes = ctx.bytes();
  int unit;
  for (unit=0; bytes >= 10240; bytes>>=10,unit++) ;
  printf("using %d%cB memory\n", (u32)bytes, " KMGT"[unit]);

  for (u32 r = 0; r < range; r++) {
    time0 = timestamp();
    ctx.setheadernonce(header, sizeof(header), nonce + r);
    printf("nonce %d k0 k1 k2 k3 %llx %llx %llx %llx\n", nonce+r, ctx.sip_keys.k0, ctx.sip_keys.k1, ctx.sip_keys.k2, ctx.sip_keys.k3);
    ctx.find_cycles();
    time1 = timestamp(); timems = (time1 - time0) / 1000000;
    printf("Time: %d ms\n", timems);
  }
}
