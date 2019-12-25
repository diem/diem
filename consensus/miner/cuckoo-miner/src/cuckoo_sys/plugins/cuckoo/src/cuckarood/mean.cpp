// Cuckarood Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2019 John Tromp

#include "mean.hpp"
#include <unistd.h>
#include <chrono>

#ifndef HEADERLEN
// arbitrary length of header hashed into siphash key
#define HEADERLEN 80
#endif

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

  for (u32 r = 0; r < range; r++) {
    time0 = timestamp();
    ctx->setheadernonce(header, header_length, nonce + r);
    print_log("nonce %d k0 k1 k2 k3 %llx %llx %llx %llx\n", nonce+r, ctx->trimmer.sip_keys.k0, ctx->trimmer.sip_keys.k1, ctx->trimmer.sip_keys.k2, ctx->trimmer.sip_keys.k3);
    u32 nsols = ctx->solve();
    time1 = timestamp();
    timems = (time1 - time0) / 1000000;
    print_log("Time: %d ms\n", timems);

    for (unsigned s = 0; s < nsols; s++) {
      print_log("Solution");
      word_t *prf = &ctx->sols[s * PROOFSIZE];
      for (u32 i = 0; i < PROOFSIZE; i++)
        print_log(" %jx", (uintmax_t)prf[i]);
      print_log("\n");
      if (solutions != NULL){
        solutions->edge_bits = EDGEBITS;
        solutions->num_sols++;
        solutions->sols[sumnsols+s].nonce = nonce + r;
        for (u32 i = 0; i < PROOFSIZE; i++) 
          solutions->sols[sumnsols+s].proof[i] = (u64) prf[i];
      }
      int pow_rc = verify(prf, ctx->trimmer.sip_keys);
      if (pow_rc == POW_OK) {
        print_log("Verified with cyclehash ");
        unsigned char cyclehash[32];
        blake2b((void *)cyclehash, sizeof(cyclehash), (const void *)prf, sizeof(proof), 0, 0);
        for (int i=0; i<32; i++)
          print_log("%02x", cyclehash[i]);
        print_log("\n");
      } else {
        print_log("FAILED due to %s\n", errstr[pow_rc]);
      }
    }
    sumnsols += nsols;
    if (stats != NULL) {
        stats->device_id = 0;
        stats->edge_bits = EDGEBITS;
        strncpy(stats->device_name, "CPU\0", MAX_NAME_LEN);
        stats->last_start_time = time0;
        stats->last_end_time = time1;
        stats->last_solution_time = time1 - time0;
    }
  }
  print_log("%d total solutions\n", sumnsols);
  return sumnsols > 0;
}

CALL_CONVENTION SolverCtx* create_solver_ctx(SolverParams* params) {
  if (params->nthreads == 0) params->nthreads = 1;
  if (params->ntrims == 0) params->ntrims = EDGEBITS >= 30 ? 96 : 68;

  SolverCtx* ctx = new SolverCtx(params->nthreads,
                                 params->ntrims,
                                 params->allrounds,
                                 params->showcycle,
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
	// not required in this solver
}

int main(int argc, char **argv) {
  u32 nthreads = 0;
  u32 ntrims = 0;
  u32 nonce = 0;
  u32 range = 1;
#ifdef SAVEEDGES
  bool showcycle = 1;
#else
  bool showcycle = 0;
#endif
  char header[HEADERLEN];
  u32 len;
  bool allrounds = false;
  int c;

  memset(header, 0, sizeof(header));
  while ((c = getopt (argc, argv, "ah:m:n:r:st:x:")) != -1) {
    switch (c) {
      case 'a':
        allrounds = true;
        break;
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'x':
        len = strlen(optarg)/2;
        assert(len == sizeof(header));
        for (u32 i=0; i<len; i++)
          sscanf(optarg+2*i, "%2hhx", header+i);
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 'm':
        ntrims = atoi(optarg) & -2; // make even as required by solve()
        break;
      case 's':
        showcycle = true;
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }

  SolverParams params;
  params.nthreads = nthreads;
  params.ntrims = ntrims;
  params.showcycle = showcycle;
  params.allrounds = allrounds;

  SolverCtx* ctx = create_solver_ctx(&params);

  print_log("Looking for %d-cycle on cuckarood%d(\"%s\",%d", PROOFSIZE, EDGEBITS, header, nonce);
  if (range > 1)
    print_log("-%d", nonce+range-1);
  print_log(") with 50%% edges\n");

  u64 sbytes = ctx->sharedbytes();
  u32 tbytes = ctx->threadbytes();
  int sunit,tunit;
  for (sunit=0; sbytes >= 10240; sbytes>>=10,sunit++) ;
  for (tunit=0; tbytes >= 10240; tbytes>>=10,tunit++) ;
  print_log("Using %d%cB bucket memory at %lx,\n", sbytes, " KMGT"[sunit], (u64)ctx->trimmer.buckets);
  print_log("%dx%d%cB thread memory at %lx,\n", params.nthreads, tbytes, " KMGT"[tunit], (u64)ctx->trimmer.tbuckets);
  print_log("%d-way siphash, and %d buckets.\n", NSIPHASH, NX);

	run_solver(ctx, header, sizeof(header), nonce, range, NULL, NULL);

	destroy_solver_ctx(ctx);
}
