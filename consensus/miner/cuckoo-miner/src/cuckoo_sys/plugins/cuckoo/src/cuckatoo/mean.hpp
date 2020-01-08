// Cuckatoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp
// The edge-trimming memory optimization is due to Dave Andersen
// http://da-data.blogspot.com/2014/03/a-public-review-of-cuckoo-cycle.html
// xenoncat demonstrated at https://github.com/xenoncat/cuckoo_pow
// how bucket sorting avoids random memory access latency
// my own cycle finding is run single threaded to avoid losing cycles
// to race conditions (typically takes under 1% of runtime)

#include "cuckatoo.h"
#include "../crypto/siphashxN.h"
#include <stdlib.h>
#include <pthread.h>
#include <unistd.h> // sleep
#include <x86intrin.h>
#include <assert.h>
#include <vector>
#include <bitset>
#include "graph.hpp"
#include "../threads/barrier.hpp"

// algorithm/performance parameters

// EDGEBITS/NEDGES/NODEMASK defined in cuckoo.h

// The node bits are logically split into 3 groups:
// XBITS 'X' bits (most significant), YBITS 'Y' bits, and ZBITS 'Z' bits (least significant)
// Here we have the default XBITS=YBITS=7, ZBITS=15 summing to EDGEBITS=29
// nodebits   XXXXXXX YYYYYYY ZZZZZZZZZZZZZZZ
// bit%10     8765432 1098765 432109876543210
// bit/10     2222222 2211111 111110000000000

// The matrix solver stores all edges in a matrix of NX * NX buckets,
// where NX = 2^XBITS is the number of possible values of the 'X' bits.
// Edge i between nodes ui = siphash24(2*i) and vi = siphash24(2*i+1)
// resides in the bucket at (uiX,viX)
// In each trimming round, either a matrix row or a matrix column (NX buckets)
// is bucket sorted on uY or vY respectively, and then within each bucket
// uZ or vZ values are counted and edges with a count of only one are eliminated,
// while remaining edges are bucket sorted back on vX or uX respectively.
// When sufficiently many edges have been eliminated, a pair of compression
// rounds remap surviving Y,Z values in each row or column into 15 bit
// combined YZ values, allowing the remaining rounds to avoid the sorting on Y,
// and directly count YZ values in a cache friendly 32KB.
// A final pair of compression rounds remap YZ values from 15 into 11 bits.

#ifndef XBITS
// 7 seems to give best performance
#define XBITS 7
#endif

#define YBITS XBITS

// size in bytes of a big bucket entry
#ifndef BIGSIZE
#if EDGEBITS <= 15
#define BIGSIZE 4
// no compression needed
#define COMPRESSROUND 0
#else
#define BIGSIZE 5
// YZ compression round; must be even
#ifndef COMPRESSROUND
#define COMPRESSROUND 14
#endif
#endif
#endif
// size in bytes of a small bucket entry
#define SMALLSIZE BIGSIZE

// initial entries could be smaller at percent or two slowdown
#ifndef BIGSIZE0
#if EDGEBITS < 30 && !defined SAVEEDGES
#define BIGSIZE0 4
#else
#define BIGSIZE0 BIGSIZE
#endif
#endif
// but they may need syncing entries
#if BIGSIZE0 == 4 && EDGEBITS > 27
#define NEEDSYNC
#endif

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

#if EDGEBITS >= 30
typedef u64 offset_t;
#else
typedef u32 offset_t;
#endif

#if BIGSIZE0 > 4
typedef u64 BIGTYPE0;
#else
typedef u32 BIGTYPE0;
#endif

// node bits have two groups of bucketbits (X for big and Y for small) and a remaining group Z of degree bits
const u32 NX        = 1 << XBITS;
const u32 XMASK     = NX - 1;
const u32 NY        = 1 << YBITS;
const u32 YMASK     = NY - 1;
const u32 XYBITS    = XBITS + YBITS;
const u32 NXY       = 1 << XYBITS;
const u32 ZBITS     = EDGEBITS - XYBITS;
const u32 NZ        = 1 << ZBITS;
const u32 ZMASK     = NZ - 1;
const u32 YZBITS    = EDGEBITS - XBITS;
const u32 NYZ       = 1 << YZBITS;
const u32 YZMASK    = NYZ - 1;
const u32 YZ1BITS   = YZBITS < 16 ? YZBITS : 16;  // compressed YZ bits
const u32 NYZ1      = 1 << YZ1BITS;
const u32 YZ1MASK   = NYZ1 - 1;
const u32 Z1BITS    = YZ1BITS - YBITS;
const u32 NZ1       = 1 << Z1BITS;
const u32 Z1MASK    = NZ1 - 1;
const u32 YZ2BITS   = YZBITS < 12 ? YZBITS : 12;  // more compressed YZ bits
const u32 NYZ2      = 1 << YZ2BITS;
const u32 YZ2MASK   = NYZ2 - 1;
const u32 Z2BITS    = YZ2BITS - YBITS;
const u32 NZ2       = 1 << Z2BITS;
const u32 Z2MASK    = NZ2 - 1;
const u32 YZZBITS   = YZBITS + ZBITS;
const u32 YZZ1BITS  = YZ1BITS + ZBITS;

const u32 MAXEDGES = NX * NYZ2;

const u32 BIGSLOTBITS   = BIGSIZE * 8;
const u32 SMALLSLOTBITS = SMALLSIZE * 8;
const u64 BIGSLOTMASK   = (1ULL << BIGSLOTBITS) - 1ULL;
const u64 SMALLSLOTMASK = (1ULL << SMALLSLOTBITS) - 1ULL;
const u32 BIGSLOTBITS0  = BIGSIZE0 * 8;
const u64 BIGSLOTMASK0  = (1ULL << BIGSLOTBITS0) - 1ULL;
const u32 NONYZBITS     = BIGSLOTBITS0 - YZBITS;
const u32 NNONYZ        = 1 << NONYZBITS;

// for p close to 0, Pr(X>=k) < e^{-n*p*eps^2} where k=n*p*(1+eps)
// see https://en.wikipedia.org/wiki/Binomial_distribution#Tail_bounds
// eps should be at least 1/sqrt(n*p/64)
// to give negligible bad odds of e^-64.

// 1/32 reduces odds of overflowing z bucket on 2^30 nodes to 2^14*e^-32
// (less than 1 in a billion) in theory. not so in practice (fails first at mean30 -n 1549)
// 3/64 works well for 29, would need to be enlarged to 1/16 for EDGEBITS=27
#ifndef BIGEPS
#define BIGEPS 3/64
#endif

// 176/256 is safely over 1-e(-1) ~ 0.63 trimming fraction
#ifndef TRIMFRAC256
#define TRIMFRAC256 176
#endif

const u32 NTRIMMEDZ  = NZ * TRIMFRAC256 / 256;

const u32 ZBUCKETSLOTS = NZ + NZ * BIGEPS;
#ifdef SAVEEDGES
const u32 ZBUCKETSIZE = NTRIMMEDZ * (BIGSIZE + sizeof(u32));  // assumes EDGEBITS <= 32
#else
const u32 ZBUCKETSIZE = ZBUCKETSLOTS * BIGSIZE0; 
#endif
const u32 TBUCKETSIZE = ZBUCKETSLOTS * BIGSIZE; 

template<u32 BUCKETSIZE>
struct zbucket {
  u32 size;
  // should avoid different values of RENAMESIZE in different threads of one process
  static const u32 RENAMESIZE = NZ2 + (COMPRESSROUND ? NZ1 : 0);
  union alignas(16) {
    u8 bytes[BUCKETSIZE];
    struct {
#ifdef SAVEEDGES
      u32 words[BUCKETSIZE/sizeof(u32) - RENAMESIZE - NTRIMMEDZ];
#else
      u32 words[BUCKETSIZE/sizeof(u32) - RENAMESIZE];
#endif
      u32 renameu1[NZ2/2];
      u32 renamev1[NZ2/2];
      u32 renameu[COMPRESSROUND ? NZ1/2 : 0];
      u32 renamev[COMPRESSROUND ? NZ1/2 : 0];
#ifdef SAVEEDGES
      u32 edges[NTRIMMEDZ];
#endif
    };
  };
  u32 setsize(u8 const *end) {
    size = end - bytes;
    assert(size <= BUCKETSIZE);
    return size;
  }
};

template<u32 BUCKETSIZE>
using yzbucket = zbucket<BUCKETSIZE>[NY];
template <u32 BUCKETSIZE>
using matrix = yzbucket<BUCKETSIZE>[NX];

template<u32 BUCKETSIZE>
struct indexer {
  offset_t index[NX];

  void matrixv(const u32 y) {
    const yzbucket<BUCKETSIZE> *foo = 0;
    for (u32 x = 0; x < NX; x++)
      index[x] = foo[x][y].bytes - (u8 *)foo;
  }
  offset_t storev(yzbucket<BUCKETSIZE> *buckets, const u32 y) {
    u8 const *base = (u8 *)buckets;
    offset_t sumsize = 0;
    for (u32 x = 0; x < NX; x++)
      sumsize += buckets[x][y].setsize(base+index[x]);
    return sumsize;
  }
  void matrixu(const u32 x) {
    const yzbucket<BUCKETSIZE> *foo = 0;
    for (u32 y = 0; y < NY; y++)
      index[y] = foo[x][y].bytes - (u8 *)foo;
  }
  offset_t storeu(yzbucket<BUCKETSIZE> *buckets, const u32 x) {
    u8 const *base = (u8 *)buckets;
    offset_t sumsize = 0;
    for (u32 y = 0; y < NY; y++)
      sumsize += buckets[x][y].setsize(base+index[y]);
    return sumsize;
  }
};

#define likely(x)   __builtin_expect((x)!=0, 1)
#define unlikely(x) __builtin_expect((x), 0)

class edgetrimmer; // avoid circular references

typedef struct {
  u32 id;
  pthread_t thread;
  edgetrimmer *et;
} thread_ctx;

typedef u8 zbucket8[NYZ1];
typedef u16 zbucket16[NTRIMMEDZ];
typedef u32 zbucket32[NTRIMMEDZ];

// maintains set of trimmable edges
class edgetrimmer {
public:
  siphash_keys sip_keys;
  yzbucket<ZBUCKETSIZE> *buckets;
  yzbucket<TBUCKETSIZE> *tbuckets;
  zbucket32 *tedges;
  zbucket16 *tzs;
  zbucket8 *tdegs;
  offset_t *tcounts;
  u32 ntrims;
  u32 nthreads;
  bool showall;
  thread_ctx *threads;
  trim_barrier barry;

  void touch(u8 *p, const offset_t n) {
    for (offset_t i=0; i<n; i+=4096)
      *(u32 *)(p+i) = 0;
  }
  edgetrimmer(const u32 n_threads, const u32 n_trims, const bool show_all) : barry(n_threads) {
    assert(sizeof(matrix<ZBUCKETSIZE>) == NX * sizeof(yzbucket<ZBUCKETSIZE>));
    assert(sizeof(matrix<TBUCKETSIZE>) == NX * sizeof(yzbucket<TBUCKETSIZE>));
    nthreads = n_threads;
    ntrims   = n_trims;
    showall = show_all;
    buckets  = new yzbucket<ZBUCKETSIZE>[NX];
    touch((u8 *)buckets, sizeof(matrix<ZBUCKETSIZE>));
    tbuckets = new yzbucket<TBUCKETSIZE>[nthreads];
    touch((u8 *)tbuckets, sizeof(yzbucket<TBUCKETSIZE>[nthreads]));
#ifdef SAVEEDGES
    tedges  = 0;
#else
    tedges  = new zbucket32[nthreads];
#endif
    tdegs   = new zbucket8[nthreads];
    tzs     = new zbucket16[nthreads];
    tcounts = new offset_t[nthreads];
    threads = new thread_ctx[nthreads];
  }
  ~edgetrimmer() {
    delete[] threads;
    delete[] buckets;
    delete[] tbuckets;
    delete[] tedges;
    delete[] tdegs;
    delete[] tzs;
    delete[] tcounts;
  }
  offset_t count() const {
    offset_t cnt = 0;
    for (u32 t = 0; t < nthreads; t++)
      cnt += tcounts[t];
    return cnt;
  }

  void genUnodes(const u32 id, const u32 uorv) {
    u64 rdtsc0, rdtsc1;
#ifdef NEEDSYNC
    u32 last[NX];;
#endif
  
    rdtsc0 = __rdtsc();
    u8 const *base = (u8 *)buckets;
    indexer<ZBUCKETSIZE> dst;
    const u32 starty = NY *  id    / nthreads;
    const u32   endy = NY * (id+1) / nthreads;
    u32 edge = starty << YZBITS, endedge = edge + NYZ;
#if NSIPHASH == 4
    const __m128i vxmask = _mm_set1_epi64x(XMASK);
    const __m128i vyzmask = _mm_set1_epi64x(YZMASK);
    __m128i v0, v1, v2, v3, v4, v5, v6, v7;
    const u32 e2 = 2 * edge + uorv;
    __m128i vpacket0 = _mm_set_epi64x(e2+2, e2+0);
    __m128i vpacket1 = _mm_set_epi64x(e2+6, e2+4);
    const __m128i vpacketinc = _mm_set1_epi64x(8);
    u64 e1 = edge;
    __m128i vhi0 = _mm_set_epi64x((e1+1)<<YZBITS, (e1+0)<<YZBITS);
    __m128i vhi1 = _mm_set_epi64x((e1+3)<<YZBITS, (e1+2)<<YZBITS);
    const __m128i vhiinc = _mm_set1_epi64x(4<<YZBITS);
#elif NSIPHASH == 8
    const __m256i vxmask = _mm256_set1_epi64x(XMASK);
    const __m256i vyzmask = _mm256_set1_epi64x(YZMASK);
    const __m256i vinit = _mm256_load_si256((__m256i *)&sip_keys);
    __m256i v0, v1, v2, v3, v4, v5, v6, v7;
    const u32 e2 = 2 * edge + uorv;
    __m256i vpacket0 = _mm256_set_epi64x(e2+6, e2+4, e2+2, e2+0);
    __m256i vpacket1 = _mm256_set_epi64x(e2+14, e2+12, e2+10, e2+8);
    const __m256i vpacketinc = _mm256_set1_epi64x(16);
    u64 e1 = edge;
    __m256i vhi0 = _mm256_set_epi64x((e1+3)<<YZBITS, (e1+2)<<YZBITS, (e1+1)<<YZBITS, (e1+0)<<YZBITS);
    __m256i vhi1 = _mm256_set_epi64x((e1+7)<<YZBITS, (e1+6)<<YZBITS, (e1+5)<<YZBITS, (e1+4)<<YZBITS);
    const __m256i vhiinc = _mm256_set1_epi64x(8<<YZBITS);
#endif
    offset_t sumsize = 0;
    for (u32 my = starty; my < endy; my++, endedge += NYZ) {
      barrier();
      dst.matrixv(my);
#ifdef NEEDSYNC
      for (u32 x=0; x < NX; x++)
        last[x] = edge;
#endif
      for (; edge < endedge; edge += NSIPHASH) {
// bit        28..21     20..13    12..0
// node       XXXXXX     YYYYYY    ZZZZZ
#if NSIPHASH == 1
        const u32 node = sipnode(&sip_keys, edge, uorv);
        const u32 ux = node >> YZBITS;
        const BIGTYPE0 zz = (BIGTYPE0)edge << YZBITS | (node & YZMASK);
#ifndef NEEDSYNC
// bit        39..21     20..13    12..0
// write        edge     YYYYYY    ZZZZZ
        *(BIGTYPE0 *)(base+dst.index[ux]) = zz;
        dst.index[ux] += BIGSIZE0;
#else
        if (zz) {
          for (; unlikely(last[ux] + NNONYZ <= edge); last[ux] += NNONYZ, dst.index[ux] += BIGSIZE0)
            *(u32 *)(base+dst.index[ux]) = 0;
          *(u32 *)(base+dst.index[ux]) = zz;
          dst.index[ux] += BIGSIZE0;
          last[ux] = edge;
        }
#endif
#elif NSIPHASH == 4
        v7 = v3 = _mm_set1_epi64x(sip_keys.k3);
        v4 = v0 = _mm_set1_epi64x(sip_keys.k0);
        v5 = v1 = _mm_set1_epi64x(sip_keys.k1);
        v6 = v2 = _mm_set1_epi64x(sip_keys.k2);

        v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
        SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
        v2 = XOR(v2, _mm_set1_epi64x(0xffLL));
        v6 = XOR(v6, _mm_set1_epi64x(0xffLL));
        SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(XOR(v0,v1),XOR(v2,v3));
        v4 = XOR(XOR(v4,v5),XOR(v6,v7));

        vpacket0 = _mm_add_epi64(vpacket0, vpacketinc);
        vpacket1 = _mm_add_epi64(vpacket1, vpacketinc);
        v1 = _mm_srli_epi64(v0, YZBITS) & vxmask;
        v5 = _mm_srli_epi64(v4, YZBITS) & vxmask;
        v0 = (v0 & vyzmask) | vhi0;
        v4 = (v4 & vyzmask) | vhi1;
        vhi0 = _mm_add_epi64(vhi0, vhiinc);
        vhi1 = _mm_add_epi64(vhi1, vhiinc);

        u32 ux;
#ifndef __SSE41__
#define extract32(x, imm) _mm_cvtsi128_si32(_mm_srli_si128((x), 4 * (imm)))
#else
#define extract32(x, imm) _mm_extract_epi32(x, imm)
#endif
#ifndef NEEDSYNC
#define STORE0(i,v,x,w) \
  ux = extract32(v,x);\
  *(u64 *)(base+dst.index[ux]) = _mm_extract_epi64(w,i%2);\
  dst.index[ux] += BIGSIZE0;
#else
  u32 zz;
#define STORE0(i,v,x,w) \
  zz = extract32(w,x);\
  if (i || likely(zz)) {\
    ux = extract32(v,x);\
    for (; unlikely(last[ux] + NNONYZ <= edge+i); last[ux] += NNONYZ, dst.index[ux] += BIGSIZE0)\
      *(u32 *)(base+dst.index[ux]) = 0;\
    *(u32 *)(base+dst.index[ux]) = zz;\
    dst.index[ux] += BIGSIZE0;\
    last[ux] = edge+i;\
  }
#endif
        STORE0(0,v1,0,v0); STORE0(1,v1,2,v0);
        STORE0(2,v5,0,v4); STORE0(3,v5,2,v4);
#elif NSIPHASH == 8
        v7 = v3 = _mm256_permute4x64_epi64(vinit, 0xFF);
        v4 = v0 = _mm256_permute4x64_epi64(vinit, 0x00);
        v5 = v1 = _mm256_permute4x64_epi64(vinit, 0x55);
        v6 = v2 = _mm256_permute4x64_epi64(vinit, 0xAA);

        v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
        SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
        v2 = XOR(v2,_mm256_set1_epi64x(0xffLL));
        v6 = XOR(v6,_mm256_set1_epi64x(0xffLL));
        SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(XOR(v0,v1),XOR(v2,v3));
        v4 = XOR(XOR(v4,v5),XOR(v6,v7));

        vpacket0 = _mm256_add_epi64(vpacket0, vpacketinc);
        vpacket1 = _mm256_add_epi64(vpacket1, vpacketinc);
        v1 = _mm256_srli_epi64(v0, YZBITS) & vxmask;
        v5 = _mm256_srli_epi64(v4, YZBITS) & vxmask;
        v0 = (v0 & vyzmask) | vhi0;
        v4 = (v4 & vyzmask) | vhi1;
        vhi0 = _mm256_add_epi64(vhi0, vhiinc);
        vhi1 = _mm256_add_epi64(vhi1, vhiinc);

        u32 ux;
#ifndef NEEDSYNC
#define STORE0(i,v,x,w) \
  ux = _mm256_extract_epi32(v,x);\
  *(u64 *)(base+dst.index[ux]) = _mm256_extract_epi64(w,i%4);\
  dst.index[ux] += BIGSIZE0;
#else
  u32 zz;
#define STORE0(i,v,x,w) \
  zz = _mm256_extract_epi32(w,x);\
  if (i || likely(zz)) {\
    ux = _mm256_extract_epi32(v,x);\
    for (; unlikely(last[ux] + NNONYZ <= edge+i); last[ux] += NNONYZ, dst.index[ux] += BIGSIZE0)\
      *(u32 *)(base+dst.index[ux]) = 0;\
    *(u32 *)(base+dst.index[ux]) = zz;\
    dst.index[ux] += BIGSIZE0;\
    last[ux] = edge+i;\
  }
#endif
        STORE0(0,v1,0,v0); STORE0(1,v1,2,v0); STORE0(2,v1,4,v0); STORE0(3,v1,6,v0);
        STORE0(4,v5,0,v4); STORE0(5,v5,2,v4); STORE0(6,v5,4,v4); STORE0(7,v5,6,v4);
#else
#error not implemented
#endif
      }
#ifdef NEEDSYNC
      for (u32 ux=0; ux < NX; ux++) {
        for (; last[ux]<endedge-NNONYZ; last[ux]+=NNONYZ) {
          *(u32 *)(base+dst.index[ux]) = 0;
          dst.index[ux] += BIGSIZE0;
        }
      }
#endif
      sumsize += dst.storev(buckets, my);
    }
    rdtsc1 = __rdtsc();
    if (!id) print_log("genUnodes round %2d size %u rdtsc: %lu\n", uorv, sumsize/BIGSIZE0, rdtsc1-rdtsc0);
    tcounts[id] = sumsize/BIGSIZE0;
  }

  void genVnodes(const u32 id, const u32 uorv) {
    u64 rdtsc0, rdtsc1;
#if NSIPHASH == 4
    const __m128i vxmask = _mm_set1_epi64x(XMASK);
    const __m128i vyzmask = _mm_set1_epi64x(YZMASK);
    const __m128i ff = _mm_set1_epi64x(0xffLL);
    __m128i v0, v1, v2, v3, v4, v5, v6, v7;
    __m128i vpacket0, vpacket1, vhi0, vhi1;
#elif NSIPHASH == 8
    const __m256i vxmask = _mm256_set1_epi64x(XMASK);
    const __m256i vyzmask = _mm256_set1_epi64x(YZMASK);
    const __m256i vinit = _mm256_load_si256((__m256i *)&sip_keys);
    __m256i vpacket0, vpacket1, vhi0, vhi1;
    __m256i v0, v1, v2, v3, v4, v5, v6, v7;
#endif
    const u32 NONDEGBITS = std::min(BIGSLOTBITS, 2 * YZBITS) - ZBITS;
    const u32 NONDEGMASK = (1 << NONDEGBITS) - 1;
    indexer<ZBUCKETSIZE> dst;
    indexer<TBUCKETSIZE> small;
  
    rdtsc0 = __rdtsc();
    offset_t sumsize = 0;
    u8 const *base = (u8 *)buckets;
    u8 const *small0 = (u8 *)tbuckets[id];
    const u32 startux = NX *  id    / nthreads;
    const u32   endux = NX * (id+1) / nthreads;
    for (u32 ux = startux; ux < endux; ux++) { // matrix x == ux
      barrier();
      small.matrixu(0);
      for (u32 my = 0 ; my < NY; my++) {
        u32 edge = my << YZBITS;
        u8    *readbig = buckets[ux][my].bytes;
        u8 const *endreadbig = readbig + buckets[ux][my].size;
// print_log("id %d x %d y %d size %u read %d\n", id, ux, my, buckets[ux][my].size, readbig-base);
        for (; readbig < endreadbig; readbig += BIGSIZE0) {
// bit     39/31..21     20..13    12..0
// read         edge     UYYYYY    UZZZZ   within UX partition
          BIGTYPE0 e = *(BIGTYPE0 *)readbig;
#if BIGSIZE0 > 4
          e &= BIGSLOTMASK0;
#elif defined NEEDSYNC
          if (unlikely(!e)) { edge += NNONYZ; continue; }
#endif
          edge += ((u32)(e >> YZBITS) - edge) & (NNONYZ-1);
// if (ux==78 && my==243) print_log("id %d ux %d my %d e %08x prefedge %x edge %x\n", id, ux, my, e, e >> YZBITS, edge);
          const u32 uy = (e >> ZBITS) & YMASK;
// bit         39..13     12..0
// write         edge     UZZZZ   within UX UY partition
          *(u64 *)(small0+small.index[uy]) = ((u64)edge << ZBITS) | (e & ZMASK);
// print_log("id %d ux %d y %d e %010lx e' %010x\n", id, ux, my, e, ((u64)edge << ZBITS) | (e >> YBITS));
          small.index[uy] += SMALLSIZE;
        }
        if (unlikely(edge >> NONYZBITS != (((my+1) << YZBITS) - 1) >> NONYZBITS))
        { print_log("OOPS1: id %d ux %d y %d edge %x vs %x\n", id, ux, my, edge, ((my+1)<<YZBITS)-1); exit(0); }
      }
      u8 *degs = tdegs[id];
      small.storeu(tbuckets+id, 0);
      dst.matrixu(ux);
      for (u32 uy = 0 ; uy < NY; uy++) {
        memset(degs, 0, NZ);
        u8 *readsmall = tbuckets[id][uy].bytes, *endreadsmall = readsmall + tbuckets[id][uy].size;
// if (id==1) print_log("id %d ux %d y %d size %u sumsize %u\n", id, ux, uy, tbuckets[id][uy].size/BIGSIZE, sumsize);
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall+=SMALLSIZE)
          degs[*(u32 *)rdsmall & ZMASK] = 1;
        u16 *zs = tzs[id];
#ifdef SAVEEDGES
        u32 *edges0 = buckets[ux][uy].edges;
#else
        u32 *edges0 = tedges[id];
#endif
        u32 *edges = edges0, edge = 0;
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall+=SMALLSIZE) {
// bit         39..13     12..0
// read          edge     UZZZZ    sorted by UY within UX partition
          const u64 e = *(u64 *)rdsmall;
          edge += ((e >> ZBITS) - edge) & NONDEGMASK;
// if (id==0) print_log("id %d ux %d uy %d e %010lx pref %4x edge %x mask %x\n", id, ux, uy, e, e>>ZBITS, edge, NONDEGMASK);
          *edges = edge;
          const u32 z = e & ZMASK;
          *zs = z;
          const u32 delta = degs[z ^ 1];
          edges += delta;
          zs    += delta;
        }
        if (unlikely(edge >> NONDEGBITS != EDGEMASK >> NONDEGBITS))
        { print_log("OOPS2: id %d ux %d uy %d edge %x vs %x\n", id, ux, uy, edge, EDGEMASK); exit(0); }
        assert(edges - edges0 < NTRIMMEDZ);
        const u16 *readz = tzs[id];
        const u32 *readedge = edges0;
        int64_t uy34 = (int64_t)uy << YZZBITS;
#if NSIPHASH == 4
        const __m128i vuy34 = _mm_set1_epi64x(uy34);
        const __m128i vuorv = _mm_set1_epi64x(uorv);
        for (; readedge <= edges-NSIPHASH; readedge += NSIPHASH, readz += NSIPHASH) {
          v4 = v0 = _mm_set1_epi64x(sip_keys.k0);
          v5 = v1 = _mm_set1_epi64x(sip_keys.k1);
          v6 = v2 = _mm_set1_epi64x(sip_keys.k2);
          v7 = v3 = _mm_set1_epi64x(sip_keys.k3);

          vpacket0 = _mm_slli_epi64(_mm_cvtepu32_epi64(*(__m128i*) readedge     ), 1) | vuorv;
          vhi0     = vuy34 | _mm_slli_epi64(_mm_cvtepu16_epi64(_mm_set_epi64x(0,*(u64*)readz)), YZBITS);
          vpacket1 = _mm_slli_epi64(_mm_cvtepu32_epi64(*(__m128i*)(readedge + 2)), 1) | vuorv;
          vhi1     = vuy34 | _mm_slli_epi64(_mm_cvtepu16_epi64(_mm_set_epi64x(0,*(u64*)(readz + 2))), YZBITS);

          v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
          SIPROUNDX2N; SIPROUNDX2N;
          v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
          v2 = XOR(v2,ff);
          v6 = XOR(v6,ff);
          SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
          v0 = XOR(XOR(v0,v1),XOR(v2,v3));
          v4 = XOR(XOR(v4,v5),XOR(v6,v7));

          v1 = _mm_srli_epi64(v0, YZBITS) & vxmask;
          v5 = _mm_srli_epi64(v4, YZBITS) & vxmask;
          v0 = vhi0 | (v0 & vyzmask);
          v4 = vhi1 | (v4 & vyzmask);

          u32 vx;
#define STORE(i,v,x,w) \
  vx = extract32(v,x);\
  *(u64 *)(base+dst.index[vx]) = _mm_extract_epi64(w,i%2);\
  dst.index[vx] += BIGSIZE;
          STORE(0,v1,0,v0); STORE(1,v1,2,v0);
          STORE(2,v5,0,v4); STORE(3,v5,2,v4);
        }
#elif NSIPHASH == 8
        const __m256i vuy34  = _mm256_set1_epi64x(uy34);
        const __m256i vuorv  = _mm256_set1_epi64x(uorv);
        for (; readedge <= edges-NSIPHASH; readedge += NSIPHASH, readz += NSIPHASH) {
          v7 = v3 = _mm256_permute4x64_epi64(vinit, 0xFF);
          v4 = v0 = _mm256_permute4x64_epi64(vinit, 0x00);
          v5 = v1 = _mm256_permute4x64_epi64(vinit, 0x55);
          v6 = v2 = _mm256_permute4x64_epi64(vinit, 0xAA);

          vpacket0 = _mm256_slli_epi64(_mm256_cvtepu32_epi64(*(__m128i*) readedge     ), 1) | vuorv;
          vhi0     = vuy34 | _mm256_slli_epi64(_mm256_cvtepu16_epi64(_mm_set_epi64x(0,*(u64*)readz)), YZBITS);
          vpacket1 = _mm256_slli_epi64(_mm256_cvtepu32_epi64(*(__m128i*)(readedge + 4)), 1) | vuorv;
          vhi1     = vuy34 | _mm256_slli_epi64(_mm256_cvtepu16_epi64(_mm_set_epi64x(0,*(u64*)(readz + 4))), YZBITS);

          v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
          SIPROUNDX2N; SIPROUNDX2N;
          v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
          v2 = XOR(v2,_mm256_set1_epi64x(0xffLL));
          v6 = XOR(v6,_mm256_set1_epi64x(0xffLL));
          SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
          v0 = XOR(XOR(v0,v1),XOR(v2,v3));
          v4 = XOR(XOR(v4,v5),XOR(v6,v7));
    
          v1 = _mm256_srli_epi64(v0, YZBITS) & vxmask;
          v5 = _mm256_srli_epi64(v4, YZBITS) & vxmask;
          v0 = vhi0 | (v0 & vyzmask);
          v4 = vhi1 | (v4 & vyzmask);

          u32 vx;
#define STORE(i,v,x,w) \
  vx = _mm256_extract_epi32(v,x);\
  *(u64 *)(base+dst.index[vx]) = _mm256_extract_epi64(w,i%4);\
  dst.index[vx] += BIGSIZE;
// print_log("Id %d ux %d y %d edge %08x e' %010lx vx %d\n", id, ux, uy, readedge[i], _mm256_extract_epi64(w,i%4), vx);

          STORE(0,v1,0,v0); STORE(1,v1,2,v0); STORE(2,v1,4,v0); STORE(3,v1,6,v0);
          STORE(4,v5,0,v4); STORE(5,v5,2,v4); STORE(6,v5,4,v4); STORE(7,v5,6,v4);
        }
#endif
        for (; readedge < edges; readedge++, readz++) { // process up to 7 leftover edges if NSIPHASH==8
          const u32 node = sipnode(&sip_keys, *readedge, uorv);
          const u32 vx = node >> YZBITS; // & XMASK;
// bit        39..34    33..21     20..13     12..0
// write      UYYYYY    UZZZZZ     VYYYYY     VZZZZ   within VX partition
          *(u64 *)(base+dst.index[vx]) = uy34 | ((u64)*readz << YZBITS) | (node & YZMASK);
// print_log("id %d ux %d y %d edge %08x e' %010lx vx %d\n", id, ux, uy, *readedge, uy34 | ((u64)(node & YZMASK) << ZBITS) | *readz, vx);
          dst.index[vx] += BIGSIZE;
        }
      }
      sumsize += dst.storeu(buckets, ux);
    }
    rdtsc1 = __rdtsc();
    if (!id) print_log("genVnodes round %2d size %u rdtsc: %lu\n", uorv, sumsize/BIGSIZE, rdtsc1-rdtsc0);
    tcounts[id] = sumsize/BIGSIZE;
  }

  template <u32 SRCSIZE, u32 DSTSIZE, bool TRIMONV>
  void trimedges(const u32 id, const u32 round) {
    const u32 SRCSLOTBITS = std::min(SRCSIZE * 8, 2 * YZBITS);
    const u64 SRCSLOTMASK = (1ULL << SRCSLOTBITS) - 1ULL;
    const u32 SRCPREFBITS = SRCSLOTBITS - YZBITS;
    const u32 SRCPREFMASK = (1 << SRCPREFBITS) - 1;
    const u32 DSTSLOTBITS = std::min(DSTSIZE * 8, 2 * YZBITS);
    const u64 DSTSLOTMASK = (1ULL << DSTSLOTBITS) - 1ULL;
    const u32 DSTPREFBITS = DSTSLOTBITS - YZZBITS;
    const u32 DSTPREFMASK = (1 << DSTPREFBITS) - 1;
    u64 rdtsc0, rdtsc1;
    indexer<ZBUCKETSIZE> dst;
    indexer<TBUCKETSIZE> small;
  
    rdtsc0 = __rdtsc();
    offset_t sumsize = 0;
    u8 const *base = (u8 *)buckets;
    u8 const *small0 = (u8 *)tbuckets[id];
    const u32 startvx = NY *  id    / nthreads;
    const u32   endvx = NY * (id+1) / nthreads;
    for (u32 vx = startvx; vx < endvx; vx++) {
      barrier();
      small.matrixu(0);
      for (u32 ux = 0 ; ux < NX; ux++) {
        u32 uxyz = ux << YZBITS;
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        const u8 *readbig = zb.bytes, *endreadbig = readbig + zb.size;
// print_log("id %d vx %d ux %d size %u\n", id, vx, ux, zb.size/SRCSIZE);
        for (; readbig < endreadbig; readbig += SRCSIZE) {
// bit        43..37    36..22     21..15     14..0
// write      UYYYYY    UZZZZZ     VYYYYY     VZZZZ   within VX partition
          const u64 e = *(u64 *)readbig & SRCSLOTMASK;
          uxyz += ((u32)(e >> YZBITS) - uxyz) & SRCPREFMASK;
// if (round==6) print_log("id %d vx %d ux %d e %010lx suffUXYZ %05x suffUXY %03x UXYZ %08x UXY %04x mask %x\n", id, vx, ux, e, (u32)(e >> YZBITS), (u32)(e >> YZZBITS), uxyz, uxyz>>ZBITS, SRCPREFMASK);
          const u32 vy = (e >> ZBITS) & YMASK;
// bit     43/39..37    36..30     29..15     14..0
// write      UXXXXX    UYYYYY     UZZZZZ     VZZZZ   within VX VY partition
          *(u64 *)(small0+small.index[vy]) = ((u64)uxyz << ZBITS) | (e & ZMASK);
          uxyz &= ~ZMASK;
          small.index[vy] += DSTSIZE;
        }
        if (unlikely(uxyz >> YZBITS != ux))
        { print_log("OOPS3: id %d vx %d ux %d UXY %x\n", id, vx, ux, uxyz); exit(0); }
      }
      u8 *degs = tdegs[id];
      small.storeu(tbuckets+id, 0);
      TRIMONV ? dst.matrixv(vx) : dst.matrixu(vx);
      for (u32 vy = 0 ; vy < NY; vy++) {
        const u64 vy34 = (u64)vy << YZZBITS;
        memset(degs, 0, NZ);
        u8    *readsmall = tbuckets[id][vy].bytes, *endreadsmall = readsmall + tbuckets[id][vy].size;
// print_log("id %d vx %d vy %d size %u sumsize %u\n", id, vx, vy, tbuckets[id][vx].size/BIGSIZE, sumsize);
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall += DSTSIZE)
          degs[*(u32 *)rdsmall & ZMASK] = DSTSIZE;
        u32 ux = 0;
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall += DSTSIZE) {
// bit        39..37    36..30     29..15     14..0      with XBITS==YBITS==7
// read       UXXXXX    UYYYYY     UZZZZZ     VZZZZ   within VX VY partition
          const u64 e = *(u64 *)rdsmall & DSTSLOTMASK;
          ux += ((u32)(e >> YZZBITS) - ux) & DSTPREFMASK;
// print_log("id %d vx %d vy %d e %010lx suffUX %02x UX %x mask %x\n", id, vx, vy, e, (u32)(e >> YZZBITS), ux, SRCPREFMASK);
// bit    41/39..34    33..21     20..13     12..0
// write     VYYYYY    VZZZZZ     UYYYYY     UZZZZ   within UX partition
          *(u64 *)(base+dst.index[ux]) = vy34 | ((e & ZMASK) << YZBITS) | ((e >> ZBITS) & YZMASK);
          dst.index[ux] += degs[(e & ZMASK) ^ 1];
        }
        if (unlikely(ux >> DSTPREFBITS != XMASK >> DSTPREFBITS))
        { print_log("OOPS4: id %d vx %x ux %x vs %x\n", id, vx, ux, XMASK); }
      }
      sumsize += TRIMONV ? dst.storev(buckets, vx) : dst.storeu(buckets, vx);
    }
    rdtsc1 = __rdtsc();
    if (showall || (!id && !(round & (round+1))))
      print_log("trimedges id %d round %2d size %u rdtsc: %lu\n", id, round, sumsize/DSTSIZE, rdtsc1-rdtsc0);
    tcounts[id] = sumsize/DSTSIZE;
  }

  template <u32 SRCSIZE, u32 DSTSIZE, bool TRIMONV>
  void trimrename(const u32 id, const u32 round) {
    const u32 SRCSLOTBITS = std::min(SRCSIZE * 8, (TRIMONV ? YZBITS : YZ1BITS) + YZBITS);
    const u64 SRCSLOTMASK = (1ULL << SRCSLOTBITS) - 1ULL;
    const u32 SRCPREFBITS = SRCSLOTBITS - YZBITS;
    const u32 SRCPREFMASK = (1 << SRCPREFBITS) - 1;
    const u32 SRCPREFBITS2 = SRCSLOTBITS - YZZBITS;
    const u32 SRCPREFMASK2 = (1 << SRCPREFBITS2) - 1;
    u64 rdtsc0, rdtsc1;
    indexer<ZBUCKETSIZE> dst;
    indexer<TBUCKETSIZE> small;
    u32 maxnnid = 0;
  
    rdtsc0 = __rdtsc();
    offset_t sumsize = 0;
    u8 const *base = (u8 *)buckets;
    u8 const *small0 = (u8 *)tbuckets[id];
    const u32 startvx = NY *  id    / nthreads;
    const u32   endvx = NY * (id+1) / nthreads;
    for (u32 vx = startvx; vx < endvx; vx++) {
      small.matrixu(0);
      for (u32 ux = 0 ; ux < NX; ux++) {
        u32 uyz = 0;
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        const u8 *readbig = zb.bytes, *endreadbig = readbig + zb.size;
// print_log("id %d vx %d ux %d size %u\n", id, vx, ux, zb.size/SRCSIZE);
        for (; readbig < endreadbig; readbig += SRCSIZE) {
// bit        39..37    36..22     21..15     14..0
// write      UYYYYY    UZZZZZ     VYYYYY     VZZZZ   within VX partition  if TRIMONV
// bit            37...22     21..15     14..0
// write          VYYYZZ'     UYYYYY     UZZZZ   within UX partition  if !TRIMONV
          const u64 e = *(u64 *)readbig & SRCSLOTMASK;
          if (TRIMONV)
            uyz += ((u32)(e >> YZBITS) - uyz) & SRCPREFMASK;
          else uyz = e >> YZBITS;
// if (round==32 && ux==25) print_log("id %d vx %d ux %d e %010lx suffUXYZ %05x suffUXY %03x UXYZ %08x UXY %04x mask %x\n", id, vx, ux, e, (u32)(e >> YZBITS), (u32)(e >> YZZBITS), uxyz, uxyz>>ZBITS, SRCPREFMASK);
          const u32 vy = (e >> ZBITS) & YMASK;
// bit        39..37    36..30     29..15     14..0
// write      UXXXXX    UYYYYY     UZZZZZ     VZZZZ   within VX VY partition  if TRIMONV
// bit            37...31     30...15     14..0
// write          VXXXXXX     VYYYZZ'     UZZZZ   within UX UY partition  if !TRIMONV
          *(u64 *)(small0+small.index[vy]) = ((u64)(ux << (TRIMONV ? YZBITS : YZ1BITS) | uyz) << ZBITS) | (e & ZMASK);
// if (TRIMONV&&vx==75&&vy==83) print_log("id %d vx %d vy %d e %010lx e15 %x ux %x\n", id, vx, vy, ((u64)uxyz << ZBITS) | (e & ZMASK), uxyz, uxyz>>YZBITS);
          if (TRIMONV)
            uyz &= ~ZMASK;
          small.index[vy] += SRCSIZE;
        }
      }
      u8 *degs = tdegs[id];
      small.storeu(tbuckets+id, 0);
      TRIMONV ? dst.matrixv(vx) : dst.matrixu(vx);
      u32 newnodeid = 0;
      u32 *renames = TRIMONV ? buckets[0][vx].renamev : buckets[vx][0].renameu;
      u32 *endrenames = renames + NZ1/2;
      for (u32 vy = 0 ; vy < NY; vy++) {
        memset(degs, 0, NZ);
        u8    *readsmall = tbuckets[id][vy].bytes, *endreadsmall = readsmall + tbuckets[id][vy].size;
// print_log("id %d vx %d vy %d size %u sumsize %u\n", id, vx, vy, tbuckets[id][vx].size/BIGSIZE, sumsize);
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall += SRCSIZE)
          degs[*(u32 *)rdsmall & ZMASK] = 1;
        u32 ux = 0;
        u32 nrenames = 0;
        for (u8 *rdsmall = readsmall; rdsmall < endreadsmall; rdsmall += SRCSIZE) {
// bit        39..37    36..30     29..15     14..0
// read       UXXXXX    UYYYYY     UZZZZZ     VZZZZ   within VX VY partition  if TRIMONV
// bit            37...31     30...15     14..0
// read           VXXXXXX     VYYYZZ'     UZZZZ   within UX UY partition  if !TRIMONV
          const u64 e = *(u64 *)rdsmall & SRCSLOTMASK;
          if (TRIMONV)
            ux += ((u32)(e >> YZZBITS) - ux) & SRCPREFMASK2;
          else ux = e >> YZZ1BITS;
          const u32 vz = e & ZMASK;
          u16 *pdeg = (u16 *)degs + (vz >> 1);
          u16 vdeg = *pdeg;
          if (vdeg >= 0x0101) {
            if (vdeg == 0x0101) {
              *pdeg = vdeg = 0x0102 + nrenames++;
              *renames++ = vy << ZBITS | (vz & ~1);  // neutralize parity, to be restored upon use
              if (renames == endrenames) {
                endrenames += (TRIMONV ? sizeof(yzbucket<ZBUCKETSIZE>) : sizeof(zbucket<ZBUCKETSIZE>)) / sizeof(u32);
                renames = endrenames - NZ1/2;
              }
            }
            vdeg = ((vdeg-0x0102) << 1) | (vz & 1); // preserve parity
// bit       37..22     21..15     14..0
// write     VYYZZ'     UYYYYY     UZZZZ   within UX partition  if TRIMONV
            if (TRIMONV)
                 *(u64 *)(base+dst.index[ux]) = ((u64)(newnodeid + vdeg) << YZBITS ) | ((e >> ZBITS) & YZMASK);
            else *(u32 *)(base+dst.index[ux]) = (     (newnodeid + vdeg) << YZ1BITS) | ((e >> ZBITS) & YZ1MASK);
// if (vx==44&&vy==58) print_log("  id %d vx %d vy %d newe %010lx\n", id, vx, vy, vy28 | ((vdeg) << YZBITS) | ((e >> ZBITS) & YZMASK));
            dst.index[ux] += DSTSIZE;
          }
        }
        newnodeid += 2 * nrenames;
        if (TRIMONV && unlikely(ux >> SRCPREFBITS2 != XMASK >> SRCPREFBITS2))
        { print_log("OOPS6: id %d vx %d vy %d ux %x vs %x\n", id, vx, vy, ux, XMASK); exit(0); }
      }
      if (newnodeid > maxnnid)
        maxnnid = newnodeid;
      sumsize += TRIMONV ? dst.storev(buckets, vx) : dst.storeu(buckets, vx);
    }
    rdtsc1 = __rdtsc();
    if (showall || !id) print_log("trimrename id %d round %2d size %u rdtsc: %lu maxnnid %d\n", id, round, sumsize/DSTSIZE, rdtsc1-rdtsc0, maxnnid);
    if (maxnnid >= NYZ1) print_log("maxnnid %d >= NYZ1 %d\n", maxnnid, NYZ1);
    assert(maxnnid < NYZ1);
    tcounts[id] = sumsize/DSTSIZE;
  }

  template <bool TRIMONV>
  void trimedges1(const u32 id, const u32 round) {
    u64 rdtsc0, rdtsc1;
    indexer<ZBUCKETSIZE> dst;
  
    rdtsc0 = __rdtsc();
    offset_t sumsize = 0;
    u8 *degs = tdegs[id];
    u8 const *base = (u8 *)buckets;
    const u32 startvx = NY *  id    / nthreads;
    const u32   endvx = NY * (id+1) / nthreads;
    for (u32 vx = startvx; vx < endvx; vx++) {
      TRIMONV ? dst.matrixv(vx) : dst.matrixu(vx);
      memset(degs, 0, NYZ1);
      for (u32 ux = 0 ; ux < NX; ux++) {
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        u32 *readbig = zb.words, *endreadbig = readbig + zb.size/sizeof(u32);
        // print_log("id %d vx %d ux %d size %d\n", id, vx, ux, zb.size/SRCSIZE);
        for (; readbig < endreadbig; readbig++)
          degs[*readbig & YZ1MASK] = sizeof(u32);
      }
      for (u32 ux = 0 ; ux < NX; ux++) {
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        u32 *readbig = zb.words, *endreadbig = readbig + zb.size/sizeof(u32);
        for (; readbig < endreadbig; readbig++) {
// bit       31...16     15...0
// read      UYYZZZ'     VYYZZ'   within VX partition
          const u32 e = *readbig;
          const u32 vyz = e & YZ1MASK;
          // print_log("id %d vx %d ux %d e %08lx vyz %04x uyz %04x\n", id, vx, ux, e, vyz, e >> YZ1BITS);
// bit       31...16     15...0
// write     VYYZZZ'     UYYZZ'   within UX partition
          *(u32 *)(base+dst.index[ux]) = (vyz << YZ1BITS) | (e >> YZ1BITS);
          dst.index[ux] += degs[vyz ^ 1];
        }
      }
      sumsize += TRIMONV ? dst.storev(buckets, vx) : dst.storeu(buckets, vx);
    }
    rdtsc1 = __rdtsc();
    if (showall || (!id && !(round & (round+1))))
      print_log("trimedges1 id %d round %2d size %u rdtsc: %lu\n", id, round, sumsize/sizeof(u32), rdtsc1-rdtsc0);
    tcounts[id] = sumsize/sizeof(u32);
  }

  template <bool TRIMONV>
  void trimrename1(const u32 id, const u32 round) {
    u64 rdtsc0, rdtsc1;
    indexer<ZBUCKETSIZE> dst;
    u32 maxnnid = 0;
  
    rdtsc0 = __rdtsc();
    offset_t sumsize = 0;
    u8 *degs = tdegs[id];
    u8 const *base = (u8 *)buckets;
    const u32 startvx = NY *  id    / nthreads;
    const u32   endvx = NY * (id+1) / nthreads;
    for (u32 vx = startvx; vx < endvx; vx++) {
      TRIMONV ? dst.matrixv(vx) : dst.matrixu(vx);
      memset(degs, 0, NYZ1);
      for (u32 ux = 0 ; ux < NX; ux++) {
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        u32 *readbig = zb.words, *endreadbig = readbig + zb.size/sizeof(u32);
        // print_log("id %d vx %d ux %d size %d\n", id, vx, ux, zb.size/SRCSIZE);
        for (; readbig < endreadbig; readbig++)
          degs[*readbig & YZ1MASK] = 1;
      }
      u32 newnodepairid = 0;
      u32 *renames = TRIMONV ? buckets[0][vx].renamev1 : buckets[vx][0].renameu1;
      u32 *endrenames = renames + NZ2/2;
      for (u32 ux = 0 ; ux < NX; ux++) {
        zbucket<ZBUCKETSIZE> &zb = TRIMONV ? buckets[ux][vx] : buckets[vx][ux];
        u32 *readbig = zb.words, *endreadbig = readbig + zb.size/sizeof(u32);
        for (; readbig < endreadbig; readbig++) {
// bit       31...16     15...0
// read      UYYYZZ'     VYYZZ'   within VX partition
          const u32 e = *readbig;
          const u32 vyz = e & YZ1MASK;
          u16 *pdeg = (u16 *)degs + (vyz >> 1);
          u16 vdeg = *pdeg;
          if (vdeg >= 0x0101) {
            if (vdeg == 0x0101) {
              *pdeg = vdeg = 0x0102 + newnodepairid++;
              *renames++ = vyz & ~1;  // neutralize parity, to be restored upon use
              if (renames == endrenames) {
                endrenames += (TRIMONV ? sizeof(yzbucket<ZBUCKETSIZE>) : sizeof(zbucket<ZBUCKETSIZE>)) / sizeof(u32);
                renames = endrenames - NZ2/2;
                assert(renames < buckets[NX][0].renameu1);
              }
            }
            vdeg = ((vdeg-0x0102) << 1) | (vyz & 1); // preserve parity
// bit       26...16     15...0
// write     VYYZZZ"     UYYZZ'   within UX partition
            *(u32 *)(base+dst.index[ux]) = (vdeg << (TRIMONV ? YZ1BITS : YZ2BITS)) | (e >> YZ1BITS);
            dst.index[ux] += sizeof(u32);
          }
        }
      }
      if (2*newnodepairid > maxnnid)
        maxnnid = 2*newnodepairid;
      sumsize += TRIMONV ? dst.storev(buckets, vx) : dst.storeu(buckets, vx);
    }
    rdtsc1 = __rdtsc();
    if (showall || !id) print_log("trimrename1 id %d round %2d size %u rdtsc: %lu maxnnid %d\n", id, round, sumsize/sizeof(u32), rdtsc1-rdtsc0, maxnnid);
    if (maxnnid >= NYZ2) print_log("maxnnid %d >= NYZ2 %d\n", maxnnid, NYZ2);
    assert(maxnnid < NYZ2);
    tcounts[id] = sumsize/sizeof(u32);
  }

  void trim() {
    void *etworker(void *vp);
    barry.clear();
    for (u32 t = 0; t < nthreads; t++) {
      threads[t].id = t;
      threads[t].et = this;
      int err = pthread_create(&threads[t].thread, NULL, etworker, (void *)&threads[t]);
      assert(err == 0);
    }
    // sleep(7); abort();
    for (u32 t = 0; t < nthreads; t++) {
      int err = pthread_join(threads[t].thread, NULL);
      assert(err == 0);
    }
  }
  void abort() {
    barry.abort();
  }
  bool aborted() {
    return barry.aborted();
  }
  void barrier() {
    barry.wait();
  }
#ifdef EXPANDROUND
#define BIGGERSIZE BIGSIZE+1
#else
#define BIGGERSIZE BIGSIZE
#define EXPANDROUND COMPRESSROUND
#endif
  void trimmer(u32 id) {
    genUnodes(id, 0);
    barrier();
    genVnodes(id, 1);
    for (u32 round = 2; round < ntrims-2; round += 2) {
      barrier();
      if (round < COMPRESSROUND) {
        if (round < EXPANDROUND)
          trimedges<BIGSIZE, BIGSIZE, true>(id, round);
        else if (round == EXPANDROUND)
          trimedges<BIGSIZE, BIGGERSIZE, true>(id, round);
        else trimedges<BIGGERSIZE, BIGGERSIZE, true>(id, round);
      } else if (round==COMPRESSROUND) {
        trimrename<BIGGERSIZE, BIGGERSIZE, true>(id, round);
      } else trimedges1<true>(id, round);
      barrier();
      if (round < COMPRESSROUND) {
        if (round+1 < EXPANDROUND)
          trimedges<BIGSIZE, BIGSIZE, false>(id, round+1);
        else if (round+1 == EXPANDROUND)
          trimedges<BIGSIZE, BIGGERSIZE, false>(id, round+1);
        else trimedges<BIGGERSIZE, BIGGERSIZE, false>(id, round+1);
      } else if (round==COMPRESSROUND) {
        trimrename<BIGGERSIZE, sizeof(u32), false>(id, round+1);
      } else trimedges1<false>(id, round+1);
    }
    barrier();
    trimrename1<true >(id, ntrims-2);
    barrier();
    trimrename1<false>(id, ntrims-1);
  }
};

void *etworker(void *vp) {
  thread_ctx *tp = (thread_ctx *)vp;
  tp->et->trimmer(tp->id);
  pthread_exit(NULL);
  return 0;
}

#define NODEBITS (EDGEBITS + 1)

// grow with cube root of size, hardly affected by trimming
const u32 MAXPATHLEN = 16 << (EDGEBITS/3);

int nonce_cmp(const void *a, const void *b) {
  return *(u32 *)a - *(u32 *)b;
}

typedef word_t proof[PROOFSIZE];

// break circular reference with forward declaration
class solver_ctx;

typedef struct {
  u32 id;
  pthread_t thread;
  solver_ctx *solver;
} match_ctx;

class solver_ctx {
public:
  edgetrimmer trimmer;
  graph<word_t> cg;
  bool showcycle;
  bool mutatenonce;
  proof cycleus;
  proof cyclevs;
  std::bitset<NXY> uxymap;
  std::vector<word_t> sols; // concatenation of all proof's indices

#if NSIPHASH > 4  // ensure correct alignment for _mm256_load_si256 of sip_keys at start of trimmer struct
  void* operator new(size_t size) noexcept {
    void* newobj;
    int tmp = posix_memalign(&newobj, NSIPHASH * sizeof(u32), sizeof(solver_ctx));
    if (tmp != 0) {
      return nullptr;
    }
    return newobj;
  }
#endif

  solver_ctx(const u32 nthreads, const u32 n_trims, bool allrounds, bool show_cycle, bool mutate_nonce)
    : trimmer(nthreads, n_trims, allrounds), 
      cg(MAXEDGES, MAXEDGES, MAX_SOLS, 0, (char *)trimmer.tbuckets) {
    assert(cg.bytes() <= sizeof(yzbucket<TBUCKETSIZE>[nthreads])); // check that graph cg can fit in tbucket's memory
    showcycle = show_cycle;
    mutatenonce = mutate_nonce;
  }
  void setheadernonce(char* const headernonce, const u32 len, const u32 nonce) {
    if (mutatenonce) {
      ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    }
    setheader(headernonce, len, &trimmer.sip_keys);
    sols.clear();
  }
  ~solver_ctx() {
  }
  u64 sharedbytes() const {
    return sizeof(matrix<ZBUCKETSIZE>);
  }
  u32 threadbytes() const {
    return sizeof(thread_ctx) + sizeof(yzbucket<TBUCKETSIZE>) + sizeof(zbucket8) + sizeof(zbucket16) + sizeof(zbucket32);
  }
  void recordedge(const u32 i, const u32 u1, const u32 v2) {
    const u32 ux = u1 >> YZ2BITS;
    u32 uyz = trimmer.buckets[ux][(u1 >> Z2BITS) & YMASK].renameu1[(u1 & Z2MASK) >> 1] | (u1 & 1);
    const u32 v1 = v2 - MAXEDGES;
    const u32 vx = v1 >> YZ2BITS;
    u32 vyz = trimmer.buckets[(v1 >> Z2BITS) & YMASK][vx].renamev1[(v1 & Z2MASK) >> 1] | (v1 & 1);
#if COMPRESSROUND > 0
    uyz = trimmer.buckets[ux][uyz >> Z1BITS].renameu[(uyz & Z1MASK) >> 1] | (u1 & 1);
    vyz = trimmer.buckets[vyz >> Z1BITS][vx].renamev[(vyz & Z1MASK) >> 1] | (v1 & 1);
#endif
    const u32 u = cycleus[i] = (ux << YZBITS) | uyz;
    cyclevs[i] = (vx << YZBITS) | vyz;
    // print_log(" (%x,%x)", u, cyclevs[i]);
#ifdef SAVEEDGES
    u32 v = cyclevs[i];
    u32 *readedges = trimmer.buckets[ux][uyz >> ZBITS].edges, *endreadedges = readedges + NTRIMMEDZ;
    for (; readedges < endreadedges; readedges++) {
      u32 edge = *readedges;
      if (sipnode(&trimmer.sip_keys, edge, 1) == v && sipnode(&trimmer.sip_keys, edge, 0) == u) {
        sols.push_back(edge);
        return;
      }
    }
    assert(0);
#else
    uxymap[u >> ZBITS] = 1;
#endif
  }

  void solution(const proof sol) {
    // print_log("Nodes");
    for (u32 i = 0; i < PROOFSIZE; i++)
      recordedge(i, cg.links[2*sol[i]].to, cg.links[2*sol[i]+1].to);
    // print_log("\n");
    if (showcycle) {
#ifndef SAVEEDGES
      void *matchworker(void *vp);

      sols.resize(sols.size() + PROOFSIZE);
      match_ctx *threads = new match_ctx[trimmer.nthreads];
      for (u32 t = 0; t < trimmer.nthreads; t++) {
        threads[t].id = t;
        threads[t].solver = this;
        int err = pthread_create(&threads[t].thread, NULL, matchworker, (void *)&threads[t]);
        assert(err == 0);
      }
      for (u32 t = 0; t < trimmer.nthreads; t++) {
        int err = pthread_join(threads[t].thread, NULL);
        assert(err == 0);
      }
#endif
      qsort(&sols[sols.size()-PROOFSIZE], PROOFSIZE, sizeof(u32), nonce_cmp);
    }
  }

  void findcycles() {
    u64 rdtsc0, rdtsc1;
  
    rdtsc0 = __rdtsc();
    cg.reset();
    for (u32 vx = 0; vx < NX; vx++) {
      for (u32 ux = 0 ; ux < NX; ux++) {
        zbucket<ZBUCKETSIZE> &zb = trimmer.buckets[ux][vx];
        u32 *readbig = zb.words, *endreadbig = readbig + zb.size/sizeof(u32);
// print_log("vx %d ux %d size %u\n", vx, ux, zb.size/4);
        for (; readbig < endreadbig; readbig++) {
// bit        21..11     10...0
// write      UYYZZZ'    VYYZZ'   within VX partition
          const u32 e = *readbig;
          const u32 u = (ux << YZ2BITS) | (e >> YZ2BITS);
          const u32 v = (vx << YZ2BITS) | (e & YZ2MASK);
          // print_log("add_edge(%x, %x)\n", u, v);
          cg.add_edge(u, v);
        }
      }
    }
    for (u32 s=0; s < cg.nsols; s++) {
      solution(cg.sols[s]);
    }
    rdtsc1 = __rdtsc();
    print_log("findcycles rdtsc: %lu\n", rdtsc1-rdtsc0);
  }

  void abort() {
    trimmer.abort();
  }

  int solve() {
    trimmer.trim();
    if (!trimmer.aborted())
      findcycles();
    return sols.size() / PROOFSIZE;
  }

  void *matchUnodes(match_ctx *mc) {
    u64 rdtsc0, rdtsc1;
  
    rdtsc0 = __rdtsc();
    const u32 starty = NY *  mc->id    / trimmer.nthreads;
    const u32   endy = NY * (mc->id+1) / trimmer.nthreads;
    u32 edge = starty << YZBITS, endedge = edge + NYZ;
  #if NSIPHASH == 4
    const __m128i vnodemask = _mm_set1_epi64x(NODEMASK);
    siphash_keys &sip_keys = trimmer.sip_keys;
    __m128i v0, v1, v2, v3, v4, v5, v6, v7;
    const u32 e2 = 2 * edge;
    __m128i vpacket0 = _mm_set_epi64x(e2+2, e2+0);
    __m128i vpacket1 = _mm_set_epi64x(e2+6, e2+4);
    const __m128i vpacketinc = _mm_set1_epi64x(8);
  #elif NSIPHASH == 8
    const __m256i vnodemask = _mm256_set1_epi64x(NODEMASK);
    const __m256i vinit = _mm256_load_si256((__m256i *)&trimmer.sip_keys);
    __m256i v0, v1, v2, v3, v4, v5, v6, v7;
    const u32 e2 = 2 * edge;
    __m256i vpacket0 = _mm256_set_epi64x(e2+6, e2+4, e2+2, e2+0);
    __m256i vpacket1 = _mm256_set_epi64x(e2+14, e2+12, e2+10, e2+8);
    const __m256i vpacketinc = _mm256_set1_epi64x(16);
  #endif
    for (u32 my = starty; my < endy; my++, endedge += NYZ) {
      for (; edge < endedge; edge += NSIPHASH) {
  // bit        28..21     20..13    12..0
  // node       XXXXXX     YYYYYY    ZZZZZ
  #if NSIPHASH == 1
        const u32 nodeu = sipnode(&trimmer.sip_keys, edge, 0);
        if (uxymap[nodeu >> ZBITS]) {
          for (u32 j = 0; j < PROOFSIZE; j++) {
            if (cycleus[j] == nodeu && cyclevs[j] == sipnode(&trimmer.sip_keys, edge, 1)) {
              sols[sols.size()-PROOFSIZE + j] = edge;
            }
          }
        }
  // bit        39..21     20..13    12..0
  // write        edge     YYYYYY    ZZZZZ
  #elif NSIPHASH == 4
        v7 = v3 = _mm_set1_epi64x(sip_keys.k3);
        v4 = v0 = _mm_set1_epi64x(sip_keys.k0);
        v5 = v1 = _mm_set1_epi64x(sip_keys.k1);
        v6 = v2 = _mm_set1_epi64x(sip_keys.k2);

        v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
        SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
        v2 = XOR(v2, _mm_set1_epi64x(0xffLL));
        v6 = XOR(v6, _mm_set1_epi64x(0xffLL));
        SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(XOR(v0,v1),XOR(v2,v3));
        v4 = XOR(XOR(v4,v5),XOR(v6,v7));

        vpacket0 = _mm_add_epi64(vpacket0, vpacketinc);
        vpacket1 = _mm_add_epi64(vpacket1, vpacketinc);
        v0 = v0 & vnodemask;
        v4 = v4 & vnodemask;
        v1 = _mm_srli_epi64(v0, ZBITS);
        v5 = _mm_srli_epi64(v4, ZBITS);

        u32 uxy;
  #define MATCH(i,v,x,w) \
  uxy = extract32(v,x);\
  if (uxymap[uxy]) {\
    u32 u = extract32(w,x);\
    for (u32 j = 0; j < PROOFSIZE; j++) {\
      if (cycleus[j] == u && cyclevs[j] == sipnode(&trimmer.sip_keys, edge+i, 1)) {\
        sols[sols.size()-PROOFSIZE + j] = edge + i;\
      }\
    }\
  }
        MATCH(0,v1,0,v0); MATCH(1,v1,2,v0);
        MATCH(2,v5,0,v4); MATCH(3,v5,2,v4);
  #elif NSIPHASH == 8
        v7 = v3 = _mm256_permute4x64_epi64(vinit, 0xFF);
        v4 = v0 = _mm256_permute4x64_epi64(vinit, 0x00);
        v5 = v1 = _mm256_permute4x64_epi64(vinit, 0x55);
        v6 = v2 = _mm256_permute4x64_epi64(vinit, 0xAA);

        v3 = XOR(v3,vpacket0); v7 = XOR(v7,vpacket1);
        SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(v0,vpacket0); v4 = XOR(v4,vpacket1);
        v2 = XOR(v2,_mm256_set1_epi64x(0xffLL));
        v6 = XOR(v6,_mm256_set1_epi64x(0xffLL));
        SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N; SIPROUNDX2N;
        v0 = XOR(XOR(v0,v1),XOR(v2,v3));
        v4 = XOR(XOR(v4,v5),XOR(v6,v7));
  
        vpacket0 = _mm256_add_epi64(vpacket0, vpacketinc);
        vpacket1 = _mm256_add_epi64(vpacket1, vpacketinc);
        v0 = v0 & vnodemask;
        v4 = v4 & vnodemask;
        v1 = _mm256_srli_epi64(v0, ZBITS);
        v5 = _mm256_srli_epi64(v4, ZBITS);
  
        u32 uxy;
  #define MATCH(i,v,x,w) \
  uxy = _mm256_extract_epi32(v,x);\
  if (uxymap[uxy]) {\
    u32 u = _mm256_extract_epi32(w,x);\
    for (u32 j = 0; j < PROOFSIZE; j++) {\
      if (cycleus[j] == u && cyclevs[j] == sipnode(&trimmer.sip_keys, edge+i, 1)) {\
        sols[sols.size()-PROOFSIZE + j] = edge + i;\
      }\
    }\
  }
        MATCH(0,v1,0,v0); MATCH(1,v1,2,v0); MATCH(2,v1,4,v0); MATCH(3,v1,6,v0);
        MATCH(4,v5,0,v4); MATCH(5,v5,2,v4); MATCH(6,v5,4,v4); MATCH(7,v5,6,v4);
  #else
  #error not implemented
  #endif
      }
    }
    rdtsc1 = __rdtsc();
    if (trimmer.showall || !mc->id) print_log("matchUnodes id %d rdtsc: %lu\n", mc->id, rdtsc1-rdtsc0);
    pthread_exit(NULL);
    return 0;
  }
};

void *matchworker(void *vp) {
  match_ctx *tp = (match_ctx *)vp;
  tp->solver->matchUnodes(tp);
  pthread_exit(NULL);
  return 0;
}
