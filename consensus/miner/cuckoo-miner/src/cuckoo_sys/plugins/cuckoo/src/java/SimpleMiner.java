// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

import java.util.Set;
import java.util.HashSet;

class CuckooSolve {
  static final int MAXPATHLEN = 4096;
  Cuckoo graph;
  int easiness;
  int[] cuckoo;
  int[][] sols;
  int nsols;
  int nthreads;

  public CuckooSolve(byte[] hdr, int en, int ms, int nt) {
    graph = new Cuckoo(hdr);
    easiness = en;
    sols = new int[ms][Cuckoo.PROOFSIZE];
    cuckoo = new int[1+(int)Cuckoo.NNODES];
    assert cuckoo != null;
    nsols = 0;
    nthreads = nt;
  }

  public int path(int u, int[] us) {
    int nu;
    for (nu = 0; u != 0; u = cuckoo[u]) {
      if (++nu >= MAXPATHLEN) {
        while (nu-- != 0 && us[nu] != u) ;
        if (nu < 0)
          System.out.println("maximum path length exceeded");
        else System.out.println("illegal " + (MAXPATHLEN-nu) + "-cycle");
        Thread.currentThread().interrupt();
      }
      us[nu] = u;
    }
    return nu;
  }
  
  public synchronized void solution(int[] us, int nu, int[] vs, int nv) {
    Set<Edge> cycle = new HashSet<Edge>();
    int n;
    cycle.add(new Edge(us[0],vs[0]-Cuckoo.NEDGES));
    while (nu-- != 0) // u's in even position; v's in odd
      cycle.add(new Edge(us[(nu+1)&~1],us[nu|1]-Cuckoo.NEDGES));
    while (nv-- != 0) // u's in odd position; v's in even
      cycle.add(new Edge(vs[nv|1],vs[(nv+1)&~1]-Cuckoo.NEDGES));
    for (int nonce = n = 0; nonce < easiness; nonce++) {
      Edge e = graph.sipedge(nonce);
      if (cycle.contains(e)) {
        sols[nsols][n++] = nonce;
        cycle.remove(e);
      }
    }
    if (n == Cuckoo.PROOFSIZE)
      nsols++;
    else System.out.println("Only recovered " + n + " nonces");
  }
}

public class SimpleMiner implements Runnable {
  int id;
  CuckooSolve solve;

  public SimpleMiner(int i, CuckooSolve cs) {
    id = i;
    solve = cs;
  }

  public void run() {
    int[] cuckoo = solve.cuckoo;
    int[] us = new int[CuckooSolve.MAXPATHLEN], vs = new int[CuckooSolve.MAXPATHLEN];
    for (int nonce = id; nonce < solve.easiness; nonce += solve.nthreads) {
      int u = cuckoo[us[0] = (int)solve.graph.sipnode(nonce,0)];
      int v = cuckoo[vs[0] = (int)(Cuckoo.NEDGES + solve.graph.sipnode(nonce,1))];
      if (u == vs[0] || v == us[0])
        continue; // ignore duplicate edges
      int nu = solve.path(u, us), nv = solve.path(v, vs);
      if (us[nu] == vs[nv]) {
        int min = nu < nv ? nu : nv;
        for (nu -= min, nv -= min; us[nu] != vs[nv]; nu++, nv++) ;
        int len = nu + nv + 1;
        System.out.println(" " + len + "-cycle found at " + id + ":" + (int)(nonce*100L/solve.easiness) + "%");
        if (len == Cuckoo.PROOFSIZE && solve.nsols < solve.sols.length)
          solve.solution(us, nu, vs, nv);
        continue;
      }
      if (nu < nv) {
        while (nu-- != 0)
          cuckoo[us[nu+1]] = us[nu];
        cuckoo[us[0]] = vs[0];
      } else {
        while (nv-- != 0)
          cuckoo[vs[nv+1]] = vs[nv];
        cuckoo[vs[0]] = us[0];
      }
    }
    Thread.currentThread().interrupt();
  }

  public static void main(String argv[]) {
    assert Cuckoo.NNODES > 0;
    int nthreads = 1;
    int maxsols = 8;
    String header = "";
    int easipct = 50;
    for (int i = 0; i < argv.length; i++) {
      if (argv[i].equals("-e")) {
        easipct = Integer.parseInt(argv[++i]);
      } else if (argv[i].equals("-h")) {
        header = argv[++i];
      } else if (argv[i].equals("-m")) {
        maxsols = Integer.parseInt(argv[++i]);
      } else if (argv[i].equals("-t")) {
        nthreads = Integer.parseInt(argv[++i]);
      }
    }
    assert easipct >= 0 && easipct <= 100;
    System.out.println("Looking for " + Cuckoo.PROOFSIZE + "-cycle on cuckoo" + Cuckoo.NODEBITS + "(\"" + header + "\") with " + easipct + "% edges and " + nthreads + " threads");
    CuckooSolve solve = new CuckooSolve(header.getBytes(), (int)(easipct * (long)Cuckoo.NNODES / 100), maxsols, nthreads);
  
    Thread[] threads = new Thread[nthreads];
    for (int t = 0; t < nthreads; t++) {
      threads[t] = new Thread(new SimpleMiner(t, solve));
      threads[t].start();
    }
    for (int t = 0; t < nthreads; t++) {
      try {
        threads[t].join();
      } catch (InterruptedException e) {
        System.out.println(e);
        System.exit(1);
      }
    }
    for (int s = 0; s < solve.nsols; s++) {
      System.out.print("Solution");
      for (int i = 0; i < Cuckoo.PROOFSIZE; i++)
        System.out.print(String.format(" %x", solve.sols[s][i]));
      System.out.println("");
    }
  }
}
