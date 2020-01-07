// Cuck(at)oo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp
// The edge-trimming memory optimization is due to Dave Andersen
// http://da-data.blogspot.com/2014/03/a-public-review-of-cuckoo-cycle.html
// The use of prefetching was suggested by Alexander Peslyak (aka Solar Designer)

#ifdef ATOMIC
#include <atomic>
#endif
#include "cuckatoo.h"
#include "../crypto/siphashxN.h"
#include "graph.hpp"
#include <stdio.h>
#include <pthread.h>
#include "../threads/barrier.hpp"
#include <assert.h>

#ifndef MAXSOLS
#define MAXSOLS 4
#endif

typedef uint64_t u64; // save some typing

#ifdef ATOMIC
typedef std::atomic<u32> au32;
typedef std::atomic<u64> au64;
#else
typedef u32 au32;
typedef u64 au64;
#endif

// algorithm/performance parameters; assume EDGEBITS < 31

const u32 NODEBITS = EDGEBITS + 1;

#ifndef PART_BITS
// #bits used to partition edge set processing to save memory
// a value of 0 does no partitioning and is fastest
// a value of 1 partitions in two, making twice_set the
// same size as shrinkingset at about 33% slowdown
// higher values are not that interesting
#define PART_BITS 0
#endif

#ifndef NPREFETCH
// how many prefetches to queue up
// before accessing the memory
// must be a multiple of NSIPHASH
#define NPREFETCH 32
#endif

#ifndef IDXSHIFT
// minimum shift that allows cycle finding data to fit in node bitmap space
// allowing them to share the same memory
#define IDXSHIFT (PART_BITS + 8)
#endif
#define MAXEDGES (NEDGES >> IDXSHIFT)

const u32 PART_MASK = (1 << PART_BITS) - 1;
#define NONPART_BITS (EDGEBITS - PART_BITS)
#if NONPART_BITS == 32
const word_t NONPART_MASK = -1;
#else
const word_t NONPART_MASK = ((word_t)1 << NONPART_BITS) - 1;
#endif

// set that starts out full and gets reset by threads on disjoint words
class shrinkingset {
public:
  bitmap<u64> bmap;
  u64 *cnt;
  u32 nthreads;

  shrinkingset(const u32 nt) : bmap(NEDGES) {
    nthreads = nt;
    cnt  = new u64[nt];
  }
  ~shrinkingset() {
    delete[] cnt;
  }
  void clear() {
    bmap.clear();
    memset(cnt, 0, nthreads * sizeof(u64));
  }
  u64 count() const {
    u64 sum = NEDGES;
    for (u32 i=0; i<nthreads; i++)
      sum -= cnt[i];
    return sum;
  }
  void reset(word_t n, u32 thread) {
    bmap.set(n);
    cnt[thread]++;
  }
  bool test(word_t n) const {
    return !bmap.test(n);
  }
  u64 block(word_t n) const {
    return ~bmap.block(n);
  }
};

class cuckoo_ctx {
public:
  siphash_keys sip_keys;
  shrinkingset alive;
  bitmap<word_t> nonleaf;
  graph<word_t> cg;
  u32 nonce;
  proof *sols;
  u32 nsols;
  u32 nthreads;
  u32 ntrims;
  bool mutatenonce;
  trim_barrier barry;

  cuckoo_ctx(u32 n_threads, u32 n_trims, u32 max_sols, bool mutate_nonce) : alive(n_threads), nonleaf(NNODES1 >> PART_BITS),
      cg(MAXEDGES, MAXEDGES, max_sols, IDXSHIFT, (char *)nonleaf.bits), barry(n_threads) {
    print_log("cg.bytes %llu NEDGES/8 %llu\n", cg.bytes(), NEDGES/8);
    assert(cg.bytes() <= NNODES1/8); // check that graph cg can fit in share nonleaf's memory
    nthreads = n_threads;
    ntrims = n_trims;
    sols = new proof[max_sols];
    nsols = 0;
    mutatenonce = mutate_nonce;
  }
  void setheadernonce(char* headernonce, const u32 len, const u32 nce) {
    nonce = nce;
    if (mutatenonce) {
      ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    }
    setheader(headernonce, len, &sip_keys);
    alive.clear(); // set all edges to be alive
    nsols = 0;
  }
  ~cuckoo_ctx() {
    delete[] sols;
  }
  void barrier() {
    barry.wait();
  }
  void abort() {
    barry.abort();
  }
  void prefetch(const u64 *hashes, const u32 part) const {
    for (u32 i=0; i < NSIPHASH; i++) {
      u64 u = hashes[i] & NODEMASK;
      if ((u >> NONPART_BITS) == part) {
        nonleaf.prefetch(u & NONPART_MASK);
      }
    }
  }
  void node_deg(const u64 *hashes, const u32 nsiphash, const u32 part) {
    for (u32 i=0; i < nsiphash; i++) {
      u64 u = hashes[i] & NODEMASK;
      if ((u >> NONPART_BITS) == part) {
        nonleaf.set(u & NONPART_MASK);
      }
    }
  }
  void kill(const u64 *hashes, const u64 *indices, const u32 nsiphash, const u32 part, const u32 id) {
    for (u32 i=0; i < nsiphash; i++) {
      u64 u = hashes[i] & NODEMASK;
      if ((u >> NONPART_BITS) == part && !nonleaf.test((u & NONPART_MASK) ^ 1)) {
        alive.reset(indices[i]/2, id);
      }
    }
  }
  void count_node_deg(const u32 id, const u32 uorv, const u32 part) {
    alignas(64) u64 indices[NSIPHASH];
    alignas(64) u64 hashes[NPREFETCH];
  
    memset(hashes, 0, NPREFETCH * sizeof(u64)); // allow many nonleaf.set(0) to reduce branching
    u32 nidx = 0;
    word_t nloops = NEDGES / 64 / nthreads;
    for (word_t loop = 0; loop < nloops; loop++) {
      word_t block = 64 * (id + loop * nthreads);
      u64 alive64 = alive.block(block);
      for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
        u32 ffs = __builtin_ffsll(alive64);
        nonce += ffs; alive64 >>= ffs;
        indices[nidx++ % NSIPHASH] = 2*(u64)nonce + uorv;
        if (nidx % NSIPHASH == 0) {
          node_deg(hashes+nidx-NSIPHASH, NSIPHASH, part);
          siphash24xN(&sip_keys, indices, hashes+nidx-NSIPHASH);
          prefetch(hashes+nidx-NSIPHASH, part);
          nidx %= NPREFETCH;
        }
        if (ffs & 64) break; // can't shift by 64
      }
    }
    node_deg(hashes, NPREFETCH, part);
    if (nidx % NSIPHASH != 0) {
      siphash24xN(&sip_keys, indices, hashes+(nidx&-NSIPHASH));
      node_deg(hashes+(nidx&-NSIPHASH), nidx%NSIPHASH, part);
    }
  }
  void kill_leaf_edges(const u32 id, const u32 uorv, const u32 part) {
    alignas(64) u64 indices[NPREFETCH];
    alignas(64) u64 hashes[NPREFETCH];
  
    for (int i=0; i < NPREFETCH; i++)
      hashes[i] = 1; // allow many nonleaf.test(0) to reduce branching
    u32 nidx = 0;
    word_t nloops = NEDGES / 64 / nthreads;
    for (word_t loop = 0; loop < nloops; loop++) {
      word_t block = 64 * (id + loop * nthreads);
      u64 alive64 = alive.block(block);
      for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
        u32 ffs = __builtin_ffsll(alive64);
        nonce += ffs; alive64 >>= ffs;
        indices[nidx++] = 2*(u64)nonce + uorv;
        if (nidx % NSIPHASH == 0) {
          siphash24xN(&sip_keys, indices+nidx-NSIPHASH, hashes+nidx-NSIPHASH);
          prefetch(hashes+nidx-NSIPHASH, part);
          nidx %= NPREFETCH;
          kill(hashes+nidx, indices+nidx, NSIPHASH, part, id);
        }
        if (ffs & 64) break; // can't shift by 64
      }
    }
    const u32 pnsip = nidx & -NSIPHASH;
    if (pnsip != nidx) {
      siphash24xN(&sip_keys, indices+pnsip, hashes+pnsip);
    }
    kill(hashes, indices, nidx, part, id);
    const u32 nnsip = pnsip + NSIPHASH;
    kill(hashes+nnsip, indices+nnsip, NPREFETCH-nnsip, part, id);
  }
};

typedef struct {
  u32 id;
  pthread_t thread;
  cuckoo_ctx *ctx;
} thread_ctx;

void *worker(void *vp) {
  thread_ctx *tp = (thread_ctx *)vp;
  cuckoo_ctx *ctx = tp->ctx;

  shrinkingset &alive = ctx->alive;
#ifdef VERBOSE
  if (tp->id == 0) print_log("initial size %llu\n", NEDGES);
#endif
  u32 round;
  for (round=0; round < ctx->ntrims; round++) {
#ifdef VERBOSE
    if (tp->id == 0) print_log("round %2d partition sizes", round);
#endif
    for (u32 part = 0; part <= PART_MASK; part++) {
      if (tp->id == 0)
        ctx->nonleaf.clear(); // clear all counts
      ctx->barrier();
      ctx->count_node_deg(tp->id,round&1,part);
      ctx->barrier();
      ctx->kill_leaf_edges(tp->id,round&1,part);
#ifdef VERBOSE
      if (tp->id == 0) print_log(" %c%d %llu", "UV"[round&1], part, alive.count());
#endif
      ctx->barrier();
    }
#ifdef VERBOSE
    if (tp->id == 0) print_log("\n");
#endif
  }
  if (tp->id != 0)
    pthread_exit(NULL);
  print_log("%d trims completed  %d edges left\n", round-1, alive.count());
  ctx->cg.reset();
  word_t nloops = NEDGES / 64;
  for (word_t loop = 0; loop < nloops; loop++) {
    word_t block = 64 * loop;
    u64 alive64 = alive.block(block);
    for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
      u32 ffs = __builtin_ffsll(alive64);
      nonce += ffs; alive64 >>= ffs;
      word_t u=sipnode(&ctx->sip_keys, nonce, 0), v=sipnode(&ctx->sip_keys, nonce, 1);
      ctx->cg.add_compress_edge(u, v);
      if (ffs & 64) break; // can't shift by 64
    }
  }
  for (u32 s=0; s < ctx->cg.nsols; s++) {
    u32 j = 0, nalive = 0;
    word_t nloops = NEDGES / 64;
    for (word_t loop = 0; loop < nloops; loop++) {
      word_t block = 64 * loop;
      u64 alive64 = alive.block(block);
      for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
        u32 ffs = __builtin_ffsll(alive64);
        nonce += ffs; alive64 >>= ffs;
        if (j < PROOFSIZE && nalive++ == ctx->cg.sols[s][j]) {
          ctx->sols[s][j++] = nonce;
        }
        if (ffs & 64) break; // can't shift by 64
      }
    }
    assert (j == PROOFSIZE);
  }
  ctx->nsols = ctx->cg.nsols;
  pthread_exit(NULL);
  return 0;
}
