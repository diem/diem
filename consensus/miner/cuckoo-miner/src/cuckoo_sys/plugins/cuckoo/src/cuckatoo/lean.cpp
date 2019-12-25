// Cuckatoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp

#include "lean.hpp"
#include <unistd.h>

// arbitrary length of header hashed into siphash key
#define HEADERLEN 80

typedef cuckoo_ctx SolverCtx;

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
  thread_ctx *threads = new thread_ctx[ctx->nthreads];
  assert(threads);
  for (u32 r = 0; r < range; r++) {
    time0 = timestamp();
    ctx->setheadernonce(header, header_length, nonce + r);
    print_log("nonce %d k0 k1 k2 k3 %llx %llx %llx %llx\n", nonce+r, ctx->sip_keys.k0, ctx->sip_keys.k1, ctx->sip_keys.k2, ctx->sip_keys.k3);
    ctx->barry.clear();
    for (u32 t = 0; t < ctx->nthreads; t++) {
      threads[t].id = t;
      threads[t].ctx = ctx;
      int err = pthread_create(&threads[t].thread, NULL, worker, (void *)&threads[t]);
      assert(err == 0);
    }
    // sleep(39); ctx->abort();
    for (u32 t = 0; t < ctx->nthreads; t++) {
      int err = pthread_join(threads[t].thread, NULL);
      assert(err == 0);
    }
    time1 = timestamp();
    timems = (time1 - time0) / 1000000;
    print_log("Time: %d ms\n", timems);
    for (unsigned s = 0; s < ctx->nsols; s++) {
      print_log("Solution");
      for (int j = 0; j < PROOFSIZE; j++)
        print_log(" %jx", (uintmax_t)ctx->sols[s][j]);
      print_log("\n");
      if (solutions != NULL){
        solutions->edge_bits = EDGEBITS;
        solutions->num_sols++;
        solutions->sols[sumnsols+s].nonce = nonce + r;
        for (u32 i = 0; i < PROOFSIZE; i++) 
          solutions->sols[sumnsols+s].proof[i] = (u64) ctx->sols[s][i];
      }
      int pow_rc = verify(ctx->sols[s], &ctx->sip_keys);
      if (pow_rc == POW_OK) {
        print_log("Verified with cyclehash ");
        unsigned char cyclehash[32];
        blake2b((void *)cyclehash, sizeof(cyclehash), (const void *)ctx->sols[s], sizeof(ctx->sols[0]), 0, 0);
        for (int i=0; i<32; i++)
          print_log("%02x", cyclehash[i]);
        print_log("\n");
      } else {
        print_log("FAILED due to %s\n", errstr[pow_rc]);
      }
      sumnsols += ctx->nsols;
    }
    if (stats != NULL) {
      stats->device_id = 0;
      stats->edge_bits = EDGEBITS;
      strncpy(stats->device_name, "CPU\0", MAX_NAME_LEN);
      stats->last_start_time = time0;
      stats->last_end_time = time1;
      stats->last_solution_time = time1 - time0;
    }
  }
  delete[] threads;
  print_log("%d total solutions\n", sumnsols);
  return 0;
}

CALL_CONVENTION SolverCtx* create_solver_ctx(SolverParams* params) {
  if (params->nthreads == 0) params->nthreads = 1;
  if (params->ntrims == 0) params->ntrims = EDGEBITS > 30 ? 96 : 68;

  SolverCtx* ctx = new SolverCtx(params->nthreads,
                                 params->ntrims,
                                 MAXSOLS,
                                 params->mutate_nonce);
  return ctx;
}

CALL_CONVENTION void destroy_solver_ctx(SolverCtx* ctx) {
  delete ctx;
}

CALL_CONVENTION void stop_solver(SolverCtx* ctx) {
  ctx->abort();
}

CALL_CONVENTION void fill_default_params(SolverParams* params) {
  params->nthreads = 1;
  params->ntrims   = 8 * (PART_BITS+3) * (PART_BITS+4);
}

int main(int argc, char **argv) {
  int nthreads = 1;
  int ntrims   = 8 * (PART_BITS+3) * (PART_BITS+4);
  int nonce = 0;
  int range = 1;
  char header[HEADERLEN];
  unsigned len;
  int c;

  memset(header, 0, sizeof(header));
  while ((c = getopt (argc, argv, "h:m:n:r:t:")) != -1) {
    switch (c) {
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 'm':
        ntrims = atoi(optarg);
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }
  SolverParams params;
  fill_default_params(&params);
  params.nthreads = nthreads;
  params.ntrims = ntrims;

  print_log("Looking for %d-cycle on cuckatoo%d(\"%s\",%d", PROOFSIZE, EDGEBITS, header, nonce);
  if (range > 1)
    print_log("-%d", nonce+range-1);
  print_log(") with trimming to %d bits, %d threads\n", EDGEBITS-IDXSHIFT, nthreads);

  u64 EdgeBytes = NEDGES/8;
  int EdgeUnit;
  for (EdgeUnit=0; EdgeBytes >= 1024; EdgeBytes>>=10,EdgeUnit++) ;
  u64 NodeBytes = (NEDGES >> PART_BITS)/8;
  int NodeUnit;
  for (NodeUnit=0; NodeBytes >= 1024; NodeBytes>>=10,NodeUnit++) ;
  print_log("Using %d%cB edge and %d%cB node memory, and %d-way siphash\n",
     (int)EdgeBytes, " KMGT"[EdgeUnit], (int)NodeBytes, " KMGT"[NodeUnit], NSIPHASH);

  SolverCtx* ctx = create_solver_ctx(&params);

  run_solver(ctx, header, sizeof(header), nonce, range, NULL, NULL);

  destroy_solver_ctx(ctx);

  return 0;
}
