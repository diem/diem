// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

// The edge-trimming memory optimization is due to Dave Andersen
// http://da-data.blogspot.com/2014/03/a-public-review-of-cuckoo-cycle.html

#include <stdint.h>
#include <string.h>
#include "cuckatoo.h"
#include "../crypto/siphash.cuh"
#include "graph.hpp"

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <set>

#ifndef MAXSOLS
#define MAXSOLS 4
#endif

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint64_t u64; // save some typing

// algorithm parameters
#ifndef PART_BITS
// #bits used to partition edge set processing to save memory
// a value of 0 does no partitioning and is fastest
// a value of 1 partitions in two at about 33% slowdown
// higher values are not that interesting
#define PART_BITS 0
#endif

#ifndef IDXSHIFT
#define IDXSHIFT (PART_BITS + 8)
#endif
#define MAXEDGES (NEDGES >> IDXSHIFT)

const u64 edgeBytes = NEDGES/8;
const u64 nodeBytes = (NEDGES>>PART_BITS)/8;
const u32 PART_MASK = (1 << PART_BITS) - 1;
const u32 NONPART_BITS = EDGEBITS - PART_BITS;
const word_t NONPART_MASK = ((word_t)1 << NONPART_BITS) - 1;

#define NODEBITS (EDGEBITS + 1)
#define NNODES2 (2 * NEDGES)
#define NODE2MASK (NNODES2-1)

#define checkCudaErrors_V(ans) ({if (gpuAssert((ans), __FILE__, __LINE__) != cudaSuccess) return;})
#define checkCudaErrors_N(ans) ({if (gpuAssert((ans), __FILE__, __LINE__) != cudaSuccess) return NULL;})
#define checkCudaErrors(ans) ({int retval = gpuAssert((ans), __FILE__, __LINE__); if (retval != cudaSuccess) return retval;})

inline int gpuAssert(cudaError_t code, const char *file, int line, bool abort=true) {
  int device_id;
  cudaGetDevice(&device_id);
  if (code != cudaSuccess) {
    snprintf(LAST_ERROR_REASON, MAX_NAME_LEN, "Device %d GPUassert: %s %s %d", device_id, cudaGetErrorString(code), file, line);
    cudaDeviceReset();
    if (abort) return code;
  }
  return code;
}

// set that starts out full and gets reset by threads on disjoint words
class shrinkingset {
public:
  u32 *bits;
  __device__ void reset(word_t n) {
    bits[n/32] |= 1 << (n%32);
  }
  __device__ bool test(word_t n) const {
    return !((bits[n/32] >> (n%32)) & 1);
  }
  __device__ u32 block(word_t n) const {
    return ~bits[n/32];
  }
};

class biitmap {
public:
  u32 *bits;
  __device__ void set(word_t n) {
    atomicOr(&bits[n/32], 1 << (n%32));
  }
  __device__ bool test(word_t n) const {
    return (bits[n/32] >> (n%32)) & 1;
  }
};

struct trimparams {
  u16 ntrims;
  u16 blocks;
  u16 tpb;

  trimparams() {
    ntrims      = 128;
    blocks      = 128;
    tpb         = 128;
  }
};

__global__ void count_node_deg(siphash_keys &sipkeys, shrinkingset &alive, biitmap &nonleaf, u32 uorv, u32 part) {
  const int nthreads = blockDim.x * gridDim.x;
  const int id = blockIdx.x * blockDim.x + threadIdx.x;
  for (word_t block = id*32; block < NEDGES; block += nthreads*32) {
    u32 alive32 = alive.block(block);
    for (word_t nonce = block-1; alive32; ) { // -1 compensates for 1-based ffs
      u32 ffs = __ffs(alive32);
      nonce += ffs; alive32 >>= ffs;
      word_t u = dipnode(sipkeys, nonce, uorv);
      if ((u >> NONPART_BITS) == part) {
        nonleaf.set(u & NONPART_MASK);
      }
    }
  }
}

__global__ void kill_leaf_edges(siphash_keys &sipkeys, shrinkingset &alive, biitmap &nonleaf, u32 uorv, u32 part) {
  const int nthreads = blockDim.x * gridDim.x;
  const int id = blockIdx.x * blockDim.x + threadIdx.x;
  for (word_t block = id*32; block < NEDGES; block += nthreads*32) {
    u32 alive32 = alive.block(block);
    for (word_t nonce = block-1; alive32; ) { // -1 compensates for 1-based ffs
      u32 ffs = __ffs(alive32);
      nonce += ffs; alive32 >>= ffs;
      word_t u = dipnode(sipkeys, nonce, uorv);
      if ((u >> NONPART_BITS) == part && !nonleaf.test((u & NONPART_MASK) ^ 1)) {
        alive.reset(nonce);
      }
    }
  }
}

// maintains set of trimmable edges
struct edgetrimmer {
  trimparams tp;
  edgetrimmer *dt;
  shrinkingset alive;
  biitmap nonleaf;
  siphash_keys sipkeys, *dipkeys;
  bool abort;
  bool initsuccess = false;

  edgetrimmer(const trimparams _tp) : tp(_tp) {
    checkCudaErrors_V(cudaMalloc((void**)&dt, sizeof(edgetrimmer)));
    checkCudaErrors_V(cudaMalloc((void**)&dipkeys, sizeof(siphash_keys)));
    checkCudaErrors_V(cudaMalloc((void**)&alive.bits, edgeBytes));
    checkCudaErrors_V(cudaMalloc((void**)&nonleaf.bits, nodeBytes));
    cudaMemcpy(dt, this, sizeof(edgetrimmer), cudaMemcpyHostToDevice);
    initsuccess = true;
  }
  ~edgetrimmer() {
    checkCudaErrors_V(cudaFree(nonleaf.bits));
    checkCudaErrors_V(cudaFree(alive.bits));
    checkCudaErrors_V(cudaFree(dipkeys));
    checkCudaErrors_V(cudaFree(dt));
    cudaDeviceReset();
  }
  bool trim() {
    cudaMemcpy(dipkeys, &sipkeys, sizeof(sipkeys), cudaMemcpyHostToDevice);
    cudaDeviceSynchronize();
    checkCudaErrors(cudaMemset(alive.bits, 0, edgeBytes));
    for (u32 round=0; round < tp.ntrims; round++) {
      for (u32 part = 0; part <= PART_MASK; part++) {
        checkCudaErrors(cudaMemset(nonleaf.bits, 0, nodeBytes));
        if (abort) return false;
        count_node_deg<<<tp.blocks,tp.tpb>>>(*dipkeys, dt->alive, dt->nonleaf, round&1, part);
        if (abort) return false;
        kill_leaf_edges<<<tp.blocks,tp.tpb>>>(*dipkeys, dt->alive, dt->nonleaf, round&1, part);
        if (abort) return false;
      }
    }
    return true;
  }
};

struct solver_ctx {
public:
  edgetrimmer trimmer;
  bool mutatenonce;
  graph<word_t> cg;
  u64 *bits;
  proof sols[MAXSOLS];

  solver_ctx(const trimparams tp, bool mutate_nonce) : trimmer(tp), cg(MAXEDGES, MAXEDGES, MAXSOLS, IDXSHIFT) {
    bits = new u64[NEDGES/64];
    mutatenonce = mutate_nonce;
  }

  void setheadernonce(char * const headernonce, const u32 len, const u32 nonce) {
    if (mutatenonce) {
      ((u32 *)headernonce)[len/sizeof(u32)-1] = htole32(nonce); // place nonce at end
    }
    setheader(headernonce, len, &trimmer.sipkeys);
  }
  ~solver_ctx() {
    delete[] bits;
  }

  void findcycles() {
    cg.reset();
    for (word_t block = 0; block < NEDGES; block += 64) {
      u64 alive64 = ~bits[block/64];
      for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
        u32 ffs = __builtin_ffsll(alive64);
        nonce += ffs; alive64 >>= ffs;
        word_t u=sipnode(&trimmer.sipkeys, nonce, 0), v=sipnode(&trimmer.sipkeys, nonce, 1);
	cg.add_compress_edge(u, v);
        if (ffs & 64) break; // can't shift by 64
      }
    }
  }

  int solve() {
    u64 time0, time1;
    u32 timems,timems2;
    time0 = timestamp();

    trimmer.abort = false;
    if (!trimmer.trim()) // trimmer aborted
      return 0;

    cudaMemcpy(bits, trimmer.alive.bits, edgeBytes, cudaMemcpyDeviceToHost);
    u32 nedges = 0;
    for (int i = 0; i < NEDGES/64; i++)
      nedges += __builtin_popcountll(~bits[i]);
    if (nedges >= MAXEDGES) {
      print_log("overloaded! exiting...");
      exit(0);
    }
    time1 = timestamp(); timems  = (time1 - time0) / 1000000;
    time0 = timestamp();
    findcycles();
    time1 = timestamp(); timems2 = (time1 - time0) / 1000000;

    print_log("%d trims %d ms %d edges %d ms total %d ms\n", trimmer.tp.ntrims, timems, nedges, timems2, timems+timems2);

    for (u32 s=0; s < cg.nsols; s++) {
      u32 j = 0, nalive = 0;
      for (word_t block = 0; block < NEDGES; block += 64) {
        u64 alive64 = ~bits[block/64];
        for (word_t nonce = block-1; alive64; ) { // -1 compensates for 1-based ffs
          u32 ffs = __builtin_ffsll(alive64);
          nonce += ffs; alive64 >>= ffs;
          if (nalive++ == cg.sols[s][j]) {
            sols[s][j] = nonce;
            if (++j == PROOFSIZE)
              goto uncompressed;
          }
          if (ffs & 64) break; // can't shift by 64
        }
      }
      uncompressed: ;
    }
    return cg.nsols;
  }

  void abort() {
    trimmer.abort = true;
  }
};

// arbitrary length of header hashed into siphash key
#define HEADERLEN 80

typedef std::pair<word_t,word_t> edge;

#include <unistd.h>

typedef solver_ctx SolverCtx;

CALL_CONVENTION int run_solver(SolverCtx* ctx,
                               char* header,
                               int header_length,
                               u32 nonce,
                               u32 range,
                               SolverSolutions *solutions,
                               SolverStats *stats
                               )
{
  u64 time0, time1;
  u32 timems;
  u32 sumnsols = 0;
  int device_id;
  if (stats != NULL) {
    cudaGetDevice(&device_id);
    cudaDeviceProp props;
    cudaGetDeviceProperties(&props, stats->device_id);
    stats->device_id = device_id;
    stats->edge_bits = EDGEBITS;
    strncpy(stats->device_name, props.name, MAX_NAME_LEN);
  }

  if (ctx == NULL || !ctx->trimmer.initsuccess){
    print_log("Error initialising trimmer. Aborting.\n");
    print_log("Reason: %s\n", LAST_ERROR_REASON);
    if (stats != NULL) {
       stats->has_errored = true;
       strncpy(stats->error_reason, LAST_ERROR_REASON, MAX_NAME_LEN);
    }
    return 0;
  }

  for (u32 r = 0; r < range; r++) {
    time0 = timestamp();
    ctx->setheadernonce(header, header_length, nonce + r);
    print_log("nonce %d k0 k1 k2 k3 %llx %llx %llx %llx\n", nonce+r, ctx->trimmer.sipkeys.k0, ctx->trimmer.sipkeys.k1, ctx->trimmer.sipkeys.k2, ctx->trimmer.sipkeys.k3);
    u32 nsols = ctx->solve();
    time1 = timestamp(); timems = (time1 - time0) / 1000000;
    print_log("Time: %d ms\n", timems);
    for (u32 s = 0; s < nsols; s++) {
      print_log("Solution");
      for (u32 j = 0; j < PROOFSIZE; j++)
        print_log(" %x", ctx->sols[s][j]);
      print_log("\n");
      if (solutions != NULL){
        solutions->edge_bits = EDGEBITS;
        solutions->num_sols++;
        solutions->sols[sumnsols+s].nonce = nonce + r;
        for (u32 i = 0; i < PROOFSIZE; i++)
          solutions->sols[sumnsols+s].proof[i] = (u64) ctx->sols[s][i];
      }
      int pow_rc = verify(ctx->sols[s], &ctx->trimmer.sipkeys);
      if (pow_rc == POW_OK) {
        print_log("Verified with cyclehash ");
        unsigned char cyclehash[32];
        blake2b((void *)cyclehash, sizeof(cyclehash), (const void *)ctx->cg.sols[s], sizeof(ctx->cg.sols[0]), 0, 0);
        for (int i=0; i<32; i++)
          print_log("%02x", cyclehash[i]);
        print_log("\n");
      } else {
        print_log("FAILED due to %s\n", errstr[pow_rc]);
      }
    }
    sumnsols += nsols;
    if (stats != NULL) {
        stats->last_start_time = time0;
        stats->last_end_time = time1;
        stats->last_solution_time = time1 - time0;
    }
  }
  print_log("%d total solutions\n", sumnsols);
	return 0;
}

CALL_CONVENTION SolverCtx* create_solver_ctx(SolverParams* params) {
  trimparams tp;
  tp.ntrims = params->ntrims;
	tp.blocks = params->blocks;
	tp.tpb = params->tpb;

  cudaDeviceProp prop;
  checkCudaErrors_N(cudaGetDeviceProperties(&prop, params->device));

  assert(tp.tpb <= prop.maxThreadsPerBlock);

  cudaSetDevice(params->device);

  SolverCtx* ctx = new SolverCtx(tp, params->mutate_nonce);

  return ctx;
}

CALL_CONVENTION void destroy_solver_ctx(SolverCtx* ctx) {
  delete ctx;
}

CALL_CONVENTION void stop_solver(SolverCtx* ctx) {
  ctx->abort();
}

CALL_CONVENTION void fill_default_params(SolverParams* params) {
  trimparams tp;
  params->device = 0;
  params->ntrims = tp.ntrims;
  params->blocks = tp.blocks;
  params->tpb = tp.tpb;
}

int main(int argc, char **argv) {
  trimparams tp;
  u32 nonce = 0;
  u32 range = 1;
  u32 device = 0;
  char header[HEADERLEN];
  u32 len;
  int c;

  // set defaults
  SolverParams params;
  fill_default_params(&params);

  memset(header, 0, sizeof(header));
  while ((c = getopt (argc, argv, "sb:d:h:n:m:r:t:")) != -1) {
    switch (c) {
      case 's':
        print_log("SYNOPSIS\n  lcuda%d [-d device] [-h hexheader] [-m trims] [-n nonce] [-r range] [-b blocks] [-t threads]\n", NODEBITS);
        print_log("DEFAULTS\n  cuda%d -d %d -h \"\" -m %d -n %d -r %d -b %d -t %d\n", NODEBITS, device, tp.ntrims, nonce, range, tp.blocks, tp.tpb);
        exit(0);
      case 'd':
        device = params.device = atoi(optarg);
        break;
      case 'h':
        len = strlen(optarg)/2;
        assert(len <= sizeof(header));
        for (u32 i=0; i<len; i++)
          sscanf(optarg+2*i, "%2hhx", header+i); // hh specifies storage of a single byte
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'm':
        params.ntrims = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 'b':
        params.blocks = atoi(optarg);
        break;
      case 't':
        params.tpb = atoi(optarg);
        break;
    }
  }
  int nDevices;
  checkCudaErrors(cudaGetDeviceCount(&nDevices));
  assert(device < nDevices);
  cudaDeviceProp prop;
  checkCudaErrors(cudaGetDeviceProperties(&prop, device));
  u64 dbytes = prop.totalGlobalMem;
  int dunit;
  for (dunit=0; dbytes >= 10240; dbytes>>=10,dunit++) ;
  print_log("%s with %d%cB @ %d bits x %dMHz\n", prop.name, (u32)dbytes, " KMGT"[dunit], prop.memoryBusWidth, prop.memoryClockRate/1000);

  print_log("Looking for %d-cycle on cuckatoo%d(\"%s\",%d", PROOFSIZE, EDGEBITS, header, nonce);
  if (range > 1)
    print_log("-%d", nonce+range-1);
  print_log(") with 50%% edges, %d trims, %d threads %d per block\n", tp.ntrims, tp.blocks*tp.tpb, tp.tpb);

  int edgeUnit=0, nodeUnit=0;
  u64 eb = edgeBytes, nb = nodeBytes;
  for (; eb >= 1024; eb>>=10) edgeUnit++;
  for (; nb >= 1024; nb>>=10) nodeUnit++;
  print_log("Using %d%cB edge and %d%cB node memory.\n", (int)eb, " KMGT"[edgeUnit], (int)nb, " KMGT"[nodeUnit]);

  SolverCtx* ctx = create_solver_ctx(&params);

  run_solver(ctx, header, sizeof(header), nonce, range, NULL, NULL);

  return 0;
}
