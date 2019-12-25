#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "bitmap.hpp"
#include "compress.hpp"
#include <new>

typedef word_t proof[PROOFSIZE];

// cuck(ar)oo graph with given limit on number of edges (and on single partition nodes)
template <typename word_t>
class graph {
public:
  // terminates adjacency lists
  static const word_t NIL = ~(word_t)0;

  struct link { // element of adjacency list
    word_t next;
    word_t to;
  };

  word_t MAXEDGES;
  word_t MAXNODES;
  word_t nlinks; // aka halfedges, twice number of edges
  word_t *adjlist; // index into links array
  link *links;
  bool sharedmem;
  compressor<word_t> *compressu;
  compressor<word_t> *compressv;
  bitmap<u32> visited;
  u32 MAXSOLS;
  proof *sols;
  u32 nsols;

  graph(word_t maxedges, word_t maxnodes, u32 maxsols) : visited(2*maxnodes) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new word_t[2*MAXNODES]; // index into links array
    links   = new link[2*MAXEDGES];
    compressu = compressv = 0;
    sharedmem = false;
    sols    = new proof[MAXSOLS+1]; // extra one for current path
    visited.clear();
  }

  ~graph() {
    if (!sharedmem) {
      delete[] adjlist;
      delete[] links;
    }
    delete[] sols;
  }

  graph(word_t maxedges, word_t maxnodes, u32 maxsols, u32 compressbits) : visited(2*maxnodes) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new word_t[2*MAXNODES]; // index into links array
    links   = new link[2*MAXEDGES];
    compressu = new compressor<word_t>(EDGEBITS, compressbits);
    compressv = new compressor<word_t>(EDGEBITS, compressbits);
    sharedmem = false;
    sols    = new  proof[MAXSOLS];
    visited.clear();
  }

  graph(word_t maxedges, word_t maxnodes, u32 maxsols, char *bytes) : visited(2*maxnodes) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new (bytes) word_t[2*MAXNODES]; // index into links array
    links   = new (bytes += sizeof(word_t[2*MAXNODES])) link[2*MAXEDGES];
    compressu = compressv = 0;
    sharedmem = true;
    sols    = new  proof[MAXSOLS];
    visited.clear();
  }

  graph(word_t maxedges, word_t maxnodes, u32 maxsols, u32 compressbits, char *bytes) : visited(2*maxnodes) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new (bytes) word_t[2*MAXNODES]; // index into links array
    links   = new (bytes += sizeof(word_t[2*MAXNODES])) link[2*MAXEDGES];
    compressu = new compressor<word_t>(EDGEBITS, compressbits, bytes += sizeof(link[2*MAXEDGES]));
    compressv = new compressor<word_t>(EDGEBITS, compressbits, bytes + compressu->bytes());
    sharedmem = true;
    sols    = new  proof[MAXSOLS];
    visited.clear();
  }

  // total size of new-operated data, excludes sols and visited bitmap of MAXEDGES bits
  uint64_t bytes() {
    return sizeof(word_t[2*MAXNODES]) + sizeof(link[2*MAXEDGES]) + (compressu ? 2 * compressu->bytes() : 0);
  }

  void reset() {
    memset(adjlist, (char)NIL, sizeof(word_t[2*MAXNODES]));
    if (compressu) {
      compressu->reset();
      compressv->reset();
    }
    resetcounts();
  }

  void resetcounts() {
    nlinks = nsols = 0;
    // visited has entries set only during cycles() call
  }

  static int nonce_cmp(const void *a, const void *b) {
    return *(word_t *)a - *(word_t *)b;
  }

  void cycles_with_link(u32 len, word_t u, word_t dest) {
    // printf("cycles_with_link(%d, %x, %x)\n", len, u, dest);
    if (visited.test(u))
      return;
    if (u == dest) {
      print_log("  %d-cycle found\n", len);
      if (len == PROOFSIZE && nsols < MAXSOLS) {
        qsort(sols[nsols++], PROOFSIZE, sizeof(word_t), nonce_cmp);
        memcpy(sols[nsols], sols[nsols-1], sizeof(sols[0]));
      }
      return;
    }
    if (len == PROOFSIZE)
      return;
    word_t au1 = adjlist[u];
    if (au1 != NIL) {
      visited.set(u);
      for (; au1 != NIL; au1 = links[au1].next) {
        sols[nsols][len] = au1/2;
        cycles_with_link(len+1, links[au1 ^ 1].to, dest);
      }
      visited.reset(u);
    }
  }

  void add_edge(word_t u, word_t v) {
    assert(u < MAXNODES);
    assert(v < MAXNODES);
    v += MAXNODES; // distinguish partitions
    if (adjlist[u] != NIL && adjlist[v] != NIL) { // possibly part of a cycle
      sols[nsols][0] = nlinks/2;
      assert(!visited.test(u));
      cycles_with_link(1, u, v);
    }
    word_t ulink = nlinks++;
    word_t vlink = nlinks++; // the two halfedges of an edge differ only in last bit
    assert(vlink != NIL);    // avoid confusing links with NIL; guaranteed if bits in word_t > EDGEBITS + 1
    links[ulink].next = adjlist[u];
    links[vlink].next = adjlist[v];
    links[adjlist[u] = ulink].to = u;
    links[adjlist[v] = vlink].to = v;
  }

  void add_compress_edge(word_t u, word_t v) {
    add_edge(compressu->compress(u), compressv->compress(v));
  }
};
