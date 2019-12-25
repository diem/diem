// Time Memory Trade Off (TMTO, or tomato) solver

#include "tomato_miner.h"
#include <unistd.h>

// arbitrary length of header hashed into siphash key
#define HEADERLEN 80

int main(int argc, char **argv) {
  int nthreads = 1;
  bool minimalbfs = false;
  int nparts = NUPARTS;
  int range = 1;
  int nonce = 0;
  int c;
  char header[HEADERLEN];
  unsigned len;

  memset(header, 0, sizeof(header));
  while ((c = getopt (argc, argv, "h:n:p:t:r:m")) != -1) {
    switch (c) {
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'm':
        minimalbfs = true;
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'p':
        nparts = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }
  printf("Looking for %d-cycle on cuckoo%d(\"%s\",%d", PROOFSIZE, NODEBITS, header, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with 50%% edges, 1/%d memory, %d/%d parts, %d threads %d minimalbfs\n",
    1<<SAVEMEM_BITS, nparts, NUPARTS, nthreads, minimalbfs);
  u64 nodeBytes = CUCKOO_SIZE*sizeof(u64);
  int nodeUnit;
  for (nodeUnit=0; nodeBytes >= 1024; nodeBytes>>=10,nodeUnit++) ;
  printf("Using %d%cB node memory.\n", (int)nodeBytes, " KMGT"[nodeUnit]);
  thread_ctx *threads = (thread_ctx *)calloc(nthreads, sizeof(thread_ctx));
  assert(threads);
  cuckoo_ctx ctx(nthreads, nparts, minimalbfs);

  for (int r = 0; r < range; r++) {
    ctx.setheadernonce(header, sizeof(header), nonce + r);

    for (int t = 0; t < nthreads; t++) {
      threads[t].id = t;
      threads[t].ctx = &ctx;
      int err = pthread_create(&threads[t].thread, NULL, worker, (void *)&threads[t]);
      assert(err == 0);
    }
    for (int t = 0; t < nthreads; t++) {
      int err = pthread_join(threads[t].thread, NULL);
      assert(err == 0);
    }
  }
  free(threads);
  return 0;
}
