#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "bitmap.hpp"
#include "compress.hpp"
#include <new>

typedef word_t proof[PROOFSIZE];

// cuck(at)oo graph with given limit on number of edges (and on single partition nodes)
template <typename word_t>
class graph {
public:
  // terminates adjacency lists
  const word_t NIL = ~(word_t)0;	// NOTE: matches last edge when EDGEBITS==32

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

  graph(word_t maxedges, word_t maxnodes, u32 maxsols, u32 compressbits) : visited(maxedges) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new word_t[2*MAXNODES]; // index into links array
    links   = new link[2*MAXEDGES];
    compressu = compressbits ? new compressor<word_t>(EDGEBITS, compressbits) : 0;
    compressv = compressbits ? new compressor<word_t>(EDGEBITS, compressbits) : 0;
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

  graph(word_t maxedges, word_t maxnodes, u32 maxsols, u32 compressbits, char *bytes) : visited(maxedges) {
    MAXEDGES = maxedges;
    MAXNODES = maxnodes;
    MAXSOLS = maxsols;
    adjlist = new (bytes) word_t[2*MAXNODES]; // index into links array
    links   = new (bytes += sizeof(word_t[2*MAXNODES])) link[2*MAXEDGES];
    compressu = compressbits ? new compressor<word_t>(EDGEBITS, compressbits, bytes += sizeof(link[2*MAXEDGES])) : 0;
    compressv = compressbits ? new compressor<word_t>(EDGEBITS, compressbits, bytes + compressu->bytes()) : 0;
    sharedmem = true;
    sols    = new  proof[MAXSOLS+1];
    visited.clear();
  }

  // total size of new-operated data, excludes sols and visited bitmap of MAXEDGES bits
  uint64_t bytes() {
    assert(2*MAXNODES != 0 && 2*MAXEDGES != 0); // allocation fails for uncompressed EDGEBITS=31
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
    // assert((u>>1) < MAXEDGES);
    if (visited.test(u >> 1))
      return;
    if ((u ^ 1) == dest) {
      print_log("  %d-cycle found\n", len);
      if (len == PROOFSIZE && nsols < MAXSOLS) {
        memcpy(sols[nsols+1], sols[nsols], sizeof(sols[0]));
        qsort(sols[nsols++], PROOFSIZE, sizeof(word_t), nonce_cmp);
      }
      return;
    }
    if (len == PROOFSIZE)
      return;
    word_t au1 = adjlist[u ^ 1];
    if (au1 != NIL) {
      visited.set(u >> 1);
      for (; au1 != NIL; au1 = links[au1].next) {
        sols[nsols][len] = au1/2;
        cycles_with_link(len+1, links[au1 ^ 1].to, dest);
      }
      visited.reset(u >> 1);
    }
  }

  bool add_edge(word_t u, word_t v) {
    assert(u < MAXNODES);
    assert(v < MAXNODES);
    v += MAXNODES; // distinguish partitions
    if (adjlist[u ^ 1] != NIL && adjlist[v ^ 1] != NIL) { // possibly part of a cycle
      sols[nsols][0] = nlinks/2;
      assert(!visited.test(u >> 1));
      cycles_with_link(1, u, v);
    }
    word_t ulink = nlinks++;
    word_t vlink = nlinks++; // the two halfedges of an edge differ only in last bit
    assert(vlink != NIL);    // avoid confusing links with NIL (possible if word_t is u32 and EDGEBITS is 31 or 32)
#ifndef ALLOWDUPES
    for (word_t au = adjlist[u]; au != NIL; au = links[au].next)
      if (links[au ^ 1].to == v) return false; // drop duplicate edge
#endif
    links[ulink].next = adjlist[u];
    links[vlink].next = adjlist[v];
    links[adjlist[u] = ulink].to = u;
    links[adjlist[v] = vlink].to = v;
    return true;
  }

  bool add_compress_edge(word_t u, word_t v) {
    return add_edge(compressu->compress(u), compressv->compress(v));
  }
};
