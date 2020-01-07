// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp
// The edge-trimming memory optimization is due to Dave Andersen
// http://da-data.blogspot.com/2014/03/a-public-review-of-cuckoo-cycle.html

#include "cuckoo.h"
#ifdef __APPLE__
#include "osx_barrier.h"
#endif
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <assert.h>
#include <vector>
#ifdef ATOMIC
#include <atomic>
typedef std::atomic<u32> au32;
typedef std::atomic<u64> au64;
#else
typedef u32 au32;
typedef u64 au64;
#endif
#if NODEBITS <= 32
typedef u32 nonce_t;
typedef u32 node_t;
#else
typedef u64 nonce_t;
typedef u64 node_t;
#endif
#include <set>

// algorithm parameters

#define NNODES (2*NEDGES)
#define NODEBITS (EDGEBITS+1)

#ifndef SAVEMEM_BITS
#define SAVEMEM_BITS 6
#endif

#ifndef IDXSHIFT
#define IDXSHIFT SAVEMEM_BITS
#endif
#ifndef LOGPROOFSIZE
// roughly the binary logarithm of cycle length rounded down
// tweak if necessary to get cuckoo hash load between 45 and 90%
#define LOGPROOFSIZE 5
#endif
#ifndef UPART_BITS
// #bits used to partition vertex set to save memory
#define UPART_BITS (IDXSHIFT+LOGPROOFSIZE)
#endif
#define NUPARTS (1<<UPART_BITS)

#define ONCE_BITS (NEDGES >> UPART_BITS)
#define TWICE_WORDS ((2 * ONCE_BITS) / 32)

class twice_set {
public:
  au32 *bits;

  twice_set() {
    bits = (au32 *)calloc(TWICE_WORDS, sizeof(au32));
    assert(bits != 0);
  }
  void reset() {
    memset(bits, 0, TWICE_WORDS*sizeof(au32));
  }
  void set(node_t u) {
    node_t idx = u/16;
    u32 bit = 1 << (2 * (u%16));
#ifdef ATOMIC
    u32 old = std::atomic_fetch_or_explicit(&bits[idx], bit, std::memory_order_relaxed);
    if (old & bit) std::atomic_fetch_or_explicit(&bits[idx], bit<<1, std::memory_order_relaxed);
  }
  u32 test(node_t u) const {
    return (bits[u/16].load(std::memory_order_relaxed) >> (2 * (u%16))) & 2;
  }
#else
    u32 old = bits[idx];
    bits[idx] = old | (bit + (old & bit));
  }
  u32 test(node_t u) const {
    return bits[u/16] >> (2 * (u%16)) & 2;
  }
#endif
  ~twice_set() {
    free(bits);
  }
};

#define UPART_MASK (NUPARTS - 1)
// grow with cube root of size, hardly affected by trimming
#define MAXPATHLEN (8 << (NODEBITS/3))

#define CUCKOO_SIZE (NNODES >> IDXSHIFT)
#define CUCKOO_MASK (CUCKOO_SIZE - 1)
// number of (least significant) key bits that survives leftshift by NODEBITS
#define KEYBITS (64-NODEBITS)
#define KEYMASK ((1L << KEYBITS) - 1)
#define MAXDRIFT (1L << (KEYBITS - IDXSHIFT))

class cuckoo_hash {
public:
  au64 *cuckoo;
  au32 nstored;

  cuckoo_hash() {
    cuckoo = (au64 *)calloc(CUCKOO_SIZE, sizeof(au64));
    assert(cuckoo != 0);
  }
  ~cuckoo_hash() {
    free(cuckoo);
  }
  void clear() {
    memset(cuckoo, 0, CUCKOO_SIZE*sizeof(au64));
    nstored = 0;
  }
  void set(node_t u, node_t v) {
    u64 niew = (u64)u << NODEBITS | v;
    for (node_t ui = u >> IDXSHIFT; ; ui = (ui+1) & CUCKOO_MASK) {
#ifdef ATOMIC
      u64 old = 0;
      if (cuckoo[ui].compare_exchange_strong(old, niew, std::memory_order_relaxed)) {
        std::atomic_fetch_add(&nstored, 1U);
        return;
      }
      if ((old >> NODEBITS) == (u & KEYMASK)) {
        cuckoo[ui].store(niew, std::memory_order_relaxed);
#else
      u64 old = cuckoo[ui];
      if ((old == 0 && ++nstored) || (old >> NODEBITS) == (u & KEYMASK)) {
        cuckoo[ui] = niew;
#endif
        return;
      }
    }
  }
  node_t operator[](node_t u) const {
    for (node_t ui = u >> IDXSHIFT; ; ui = (ui+1) & CUCKOO_MASK) {
#ifdef ATOMIC
      u64 cu = cuckoo[ui].load(std::memory_order_relaxed);
#else
      u64 cu = cuckoo[ui];
#endif
      if (!cu)
        return 0;
      if ((cu >> NODEBITS) == (u & KEYMASK)) {
        assert(((ui - (u >> IDXSHIFT)) & CUCKOO_MASK) < MAXDRIFT);
        return (node_t)(cu & (NNODES-1));
      }
    }
  }
  u32 load() const {
    return (u32)(nstored*100L/CUCKOO_SIZE);
  }
  bool overloaded() const {
    return nstored >= (u32)(CUCKOO_SIZE*9L/10L);
  }
};

class cuckoo_ctx {
public:
  siphash_keys sip_keys;
  cuckoo_hash *cuckoo;
  bool minimalbfs;
  twice_set *nonleaf;
  u32 nparts;
  nonce_t (*sols)[PROOFSIZE];
  u32 nthreads;
  pthread_barrier_t barry;

  cuckoo_ctx(u32 n_threads, u32 n_parts, bool minimal_bfs) {
    nthreads = n_threads;
    nparts = n_parts;
    cuckoo = new cuckoo_hash();
    minimalbfs = minimal_bfs;
    if (minimalbfs)
      nonleaf = new twice_set();
    int err = pthread_barrier_init(&barry, NULL, nthreads);
    assert(err == 0);
  }
  void setheadernonce(char* headernonce, const u32 len, const u32 nonce) {
    ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    setheader(headernonce, len, &sip_keys);
  }
  ~cuckoo_ctx() {
    delete cuckoo;
    if (minimalbfs)
      delete nonleaf;
  }
};

typedef struct {
  u32 id;
  pthread_t thread;
  cuckoo_ctx *ctx;
} thread_ctx;

void barrier(pthread_barrier_t *barry) {
  int rc = pthread_barrier_wait(barry);
  if (rc != 0 && rc != PTHREAD_BARRIER_SERIAL_THREAD) {
    printf("Could not wait on barrier\n");
    pthread_exit(NULL);
  }
}

u32 path(cuckoo_hash &cuckoo, node_t u, node_t *us) {
  u32 nu;
  for (nu = 0; u; u = cuckoo[u]) {
    if (++nu >= MAXPATHLEN) {
      while (nu-- && us[nu] != u) ;
      if (!~nu)
        printf("maximum path length exceeded\n");
      else printf("illegal % 4d-cycle %d %d\n", MAXPATHLEN-nu,us[0],us[1]);
      pthread_exit(NULL);
    }
    us[nu] = u;
  }
  return nu;
}

typedef std::pair<node_t,node_t> edge;

void solution(cuckoo_ctx *ctx, node_t *us, u32 nu, node_t *vs, u32 nv) {
  std::set<edge> cycle;
  u32 n;
  cycle.insert(edge(*us, *vs));
  while (nu--)
    cycle.insert(edge(us[(nu+1)&~1], us[nu|1])); // u's in even position; v's in odd
  while (nv--)
    cycle.insert(edge(vs[nv|1], vs[(nv+1)&~1])); // u's in odd position; v's in even
  printf("Solution ");
  for (nonce_t nonce = n = 0; nonce < NEDGES; nonce++) {
    edge e(sipnode_(&ctx->sip_keys, nonce, 0), sipnode_(&ctx->sip_keys, nonce, 1));
    if (cycle.find(e) != cycle.end()) {
      printf("%x%c", nonce, ++n == PROOFSIZE?'\n':' ');
      if (PROOFSIZE > 2)
        cycle.erase(e);
    }
  }
  assert(n==PROOFSIZE);
}

void *worker(void *vp) {
  thread_ctx *tp = (thread_ctx *)vp;
  cuckoo_ctx *ctx = tp->ctx;

  cuckoo_hash &cuckoo = *ctx->cuckoo;
  twice_set *nonleaf = ctx->nonleaf;
  node_t us[MAXPATHLEN], vs[MAXPATHLEN];
  for (node_t upart=0; upart < ctx->nparts; upart++) {
    if (ctx->minimalbfs) {
      for (nonce_t nonce = tp->id; nonce < NEDGES; nonce += ctx->nthreads) {
        node_t u0 = sipnode(&ctx->sip_keys, nonce, 0);
        if (u0 != 0 && (u0 & UPART_MASK) == upart)
            nonleaf->set(u0 >> UPART_BITS);
      }
    }
    barrier(&ctx->barry);
    static int bfsdepth = ctx->minimalbfs ? PROOFSIZE/2 : PROOFSIZE;
    for (int depth=0; depth < bfsdepth; depth++) {
      u32 uorv = depth&1;
      for (nonce_t nonce = tp->id; nonce < NEDGES; nonce += ctx->nthreads) {
        node_t u0 = sipnode_(&ctx->sip_keys, nonce, uorv);
        if (u0 == 0)
          continue;
        if (depth == 0) {
          node_t u1 = u0 >> 1;
          if ((u1 & UPART_MASK) != upart)
            continue;
          if (ctx->minimalbfs && !nonleaf->test(u1 >> UPART_BITS))
            continue;
        }
        node_t u = cuckoo[us[0] = u0];
        if (depth > 0 && u == 0)
          continue;
        node_t v0 = sipnode_(&ctx->sip_keys, nonce, uorv^1);
        if (v0 == 0)
          continue;
        node_t v = cuckoo[vs[0] = v0];
#if PROOFSIZE != 2
        if (u == v0 || v == u0) // duplicate
          continue;
#endif
        u32 nu = path(cuckoo, u, us), nv = path(cuckoo, v, vs);
        if (us[nu] == vs[nv]) {
          u32 min = nu < nv ? nu : nv;
          for (nu -= min, nv -= min; us[nu] != vs[nv]; nu++, nv++) ;
          u32 len = nu + nv + 1;
          printf("% 4d-cycle found at %d:%d\n", len, tp->id, depth);
          if (len == PROOFSIZE) {
            if (depth&1)
              solution(ctx, vs, nv, us, nu);
            else solution(ctx, us, nu, vs, nv);
          }
          continue;
        }
        if (nu < nv) {
          while (nu--)
            cuckoo.set(us[nu+1], us[nu]);
          cuckoo.set(u0, v0);
        } else {
          while (nv--)
            cuckoo.set(vs[nv+1], vs[nv]);
          cuckoo.set(v0, u0);
        }
      }
      barrier(&ctx->barry);
      if (tp->id == 0 && cuckoo.load() >= 90) {
        printf("OVERLOAD !!!!!!!!!!!!!!!!!\n");
        break;
      }
    }
    if (tp->id == 0) {
      printf("upart %d depth %d load %d%%\n", upart, PROOFSIZE/2, cuckoo.load());
      cuckoo.clear();
      if (ctx->minimalbfs)
        ctx->nonleaf->reset();
    }
  }
  pthread_exit(NULL);
}
