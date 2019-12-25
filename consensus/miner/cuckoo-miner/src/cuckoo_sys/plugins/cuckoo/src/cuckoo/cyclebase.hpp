#include <utility>
#include <stdio.h>
#include <assert.h>
#include <set>

#ifndef MAXCYCLES
#define MAXCYCLES 64 // single byte
#endif

struct edge {
  u32 u;
  u32 v;
  edge() : u(0), v(0) { }
  edge(u32 x, u32 y) : u(x), v(y) { }
};

struct cyclebase {
  // should avoid different values of MAXPATHLEN in different threads of one process
  static const u32 MAXPATHLEN = 16 << (EDGEBITS/3);

  int ncycles;
  word_t *cuckoo;
  edge cycleedges[MAXCYCLES];
  u32 cyclelengths[MAXCYCLES];
  u32 prevcycle[MAXCYCLES];
  u32 us[MAXPATHLEN];
  u32 vs[MAXPATHLEN];

  void alloc() {
    cuckoo = (word_t *)calloc(NCUCKOO, sizeof(word_t));
  }

  void freemem() { // not a destructor, as memory may have been allocated elsewhere, bypassing alloc()
    free(cuckoo);
  }

  void reset() {
    resetcounts();
  }

  void resetcounts() {
    memset(cuckoo, -1, NCUCKOO * sizeof(word_t)); // for prevcycle nil
    ncycles = 0;
  }

  int path(u32 u0, u32 *us) const {
    int nu;
    for (u32 u = us[nu = 0] = u0; cuckoo[u] < 0x80000000; ) {
      u = cuckoo[u];
      if (++nu >= (int)MAXPATHLEN) {
        while (nu-- && us[nu] != u) ;
        if (nu < 0)
          printf("maximum path length exceeded\n");
        else printf("illegal % 4d-cycle from node %d\n", MAXPATHLEN-nu, u0);
        exit(0);
      }
      us[nu] = u;
    }
    return nu;
  }

  int pathjoin(u32 *us, int *pnu, u32 *vs, int *pnv) {
    int nu = *pnu, nv = *pnv;
    int min = nu < nv ? nu : nv;
    for (nu -= min, nv -= min; us[nu] != vs[nv]; nu++, nv++) min--;
    *pnu = nu; *pnv = nv;
    return min;
  }

  void addedge(u32 u0, u32 v0) {
    u32 u = u0 << 1, v = (v0 << 1) | 1;
    int nu = path(u, us), nv = path(v, vs);
    if (us[nu] == vs[nv]) {
     u32 ccsize = -cuckoo[us[nu]];
      pathjoin(us, &nu, vs, &nv);
      int len = nu + nv + 1;
      printf("% 4d-cycle found in ccsize %d\n", len, ccsize);
      cycleedges[ncycles].u = u;
      cycleedges[ncycles].v = v;
      cyclelengths[ncycles++] = len;
      if (len == PROOFSIZE)
        solution(us, nu, vs, nv);
      assert(ncycles < MAXCYCLES);
    } else if (nu < nv) {
      cuckoo[vs[nv]] += cuckoo[us[nu]];
      while (nu--)
        cuckoo[us[nu+1]] = us[nu];
      cuckoo[u] = v;
    } else {
      cuckoo[us[nu]] += cuckoo[vs[nv]];
      while (nv--)
        cuckoo[vs[nv+1]] = vs[nv];
      cuckoo[v] = u;
    }
  }

  void recordedge(const u32 i, const u32 u, const u32 v) {
    printf(" (%x,%x)", u, v);
  }

  void solution(u32 *us, int nu, u32 *vs, int nv) {
    printf("Nodes");
    u32 ni = 0;
    recordedge(ni++, *us, *vs);
    while (nu--)
      recordedge(ni++, us[(nu+1)&~1], us[nu|1]); // u's in even position; v's in odd
    while (nv--)
      recordedge(ni++, vs[nv|1], vs[(nv+1)&~1]); // u's in odd position; v's in even
    printf("\n");
#if 0
    for (u32 nonce = n = 0; nonce < NEDGES; nonce++) {
      edge e(2*sipnode(&sip_keys, nonce, 0), 2*sipnode(&sip_keys, nonce, 1)+1);
      if (cycle.find(e) != cycle.end()) {
        printf(" %x", nonce);
        cycle.erase(e);
      }
    }
    printf("\n");
#endif
  }

  int sharedlen(u32 *us, int nu, u32 *vs, int nv) {
    int len = 0;
    for (; nu-- && nv-- && us[nu] == vs[nv]; len++) ;
    return len;
  }
};
