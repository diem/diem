// Bounty Cycle, an attempt to disprove John's claims

#include "momentomatum.h"
#include <unistd.h>

int main(int argc, char **argv) {
  int nthreads = 1;
  bool minimalbfs = true;
  int nparts = NUPARTS;
  const char *header = "";
  int c;
  while ((c = getopt (argc, argv, "h:mn:t:")) != -1) {
    switch (c) {
      case 'h':
        header = optarg;
        break;
      case 'm':
        minimalbfs = true;
        break;
      case 'n':
        nparts = atoi(optarg);
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }
  printf("Looking for %d-cycle on cuckoo%d(\"%s\") with 50%% edges, 1/%d memory, %d/%d parts, %d threads %d minimalbfs\n", PROOFSIZE, NODEBITS, header, 1<<SAVEMEM_BITS, nparts, NUPARTS, nthreads, minimalbfs);
  u64 nodeBytes = CUCKOO_SIZE*sizeof(u64);
  int nodeUnit;
  for (nodeUnit=0; nodeBytes >= 1024; nodeBytes>>=10,nodeUnit++) ;
  printf("Using %d%cB node memory.\n", (int)nodeBytes, " KMGT"[nodeUnit]);
  cuckoo_ctx ctx(header, nthreads, nparts, minimalbfs);
  thread_ctx *threads = (thread_ctx *)calloc(nthreads, sizeof(thread_ctx));
  assert(threads);
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
  free(threads);
  return 0;
}
