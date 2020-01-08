// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Scanner;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

class Edge {
  int u;
  int v;

  public Edge(int x, int y) {
    u = x;
    v = y;
  }

  public int hashCode() {
    return (int)(u^v);
  }

  public boolean equals(Object o) {
    Edge f = (Edge)o;
    return u == f.u && v == f.v;
  }
}

public class Cuckoo {
  public static final int EDGEBITS = 19;
  public static final int NEDGES = 1 << EDGEBITS;
  public static final int NODEBITS = EDGEBITS + 1;
  public static final int NNODES = 1 << NODEBITS;
  public static final int EDGEMASK  = NEDGES - 1;
  public static final int PROOFSIZE = 42;

  long v[] = new long[4];

  public static long u8(byte b) {
    return (long)(b) & 0xff;
  }

  public static long u8to64(byte[] p, int i) {
    return u8(p[i  ])       | u8(p[i+1]) <<  8 |
           u8(p[i+2]) << 16 | u8(p[i+3]) << 24 |
           u8(p[i+4]) << 32 | u8(p[i+5]) << 40 |
           u8(p[i+6]) << 48 | u8(p[i+7]) << 56 ;
  }

  public Cuckoo(byte[] header) {
    byte[] hdrkey;
    try {
      hdrkey = MessageDigest.getInstance("SHA-256").digest(header);
      v[0] = u8to64(hdrkey, 0);
      v[1] = u8to64(hdrkey, 8);
      v[2] = u8to64(hdrkey, 16);
      v[3] = u8to64(hdrkey, 24);
    } catch(NoSuchAlgorithmException e) {
      System.out.println(e);
      System.exit(1);
    }
  }

  public long siphash24(int nonce) {
    long v0 = v[0], v1 = v[1], v2 = v[2], v3 = v[3] ^ nonce;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    v0 ^= nonce; v2 ^= 0xff;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    v0 += v1; v2 += v3; v1 = (v1 << 13) | v1 >>> 51;
                        v3 = (v3 << 16) | v3 >>> 48;
    v1 ^= v0; v3 ^= v2; v0 = (v0 << 32) | v0 >>> 32;
    v2 += v1; v0 += v3; v1 = (v1 << 17) | v1 >>> 47;
                        v3 = (v3 << 21) | v3 >>> 43;
    v1 ^= v2; v3 ^= v0; v2 = (v2 << 32) | v2 >>> 32;

    return v0 ^ v1 ^ v2 ^ v3;
  }

  // generate edge in cuckoo graph
  public int sipnode(int nonce, int uorv) {
    return (int)siphash24(2*nonce + uorv) & EDGEMASK;
  }
  
  // generate edge in cuckoo graph
  public Edge sipedge(int nonce) {
    return new Edge(sipnode(nonce,0), sipnode(nonce,1));
  }
  
  // verify that (ascending) nonces, all less than easiness, form a cycle in graph
  public Boolean verify(int[] nonces, int easiness) {
    int us[] = new int[PROOFSIZE], vs[] = new int[PROOFSIZE];
    int i = 0, n;
    for (n = 0; n < PROOFSIZE; n++) {
      if (nonces[n] >= easiness || (n != 0  && nonces[n] <= nonces[n-1]))
        return false;
      us[n] = sipnode(nonces[n],0);
      vs[n] = sipnode(nonces[n],1);
    }
    do {  // follow cycle until we return to i==0; n edges left to visit
      int j = i;
      for (int k = 0; k < PROOFSIZE; k++) // find unique other j with same vs[j]
        if (k != i && vs[k] == vs[i]) {
          if (j != i)
            return false;
          j = k;
      }
      if (j == i)
        return false;
      i = j;
      for (int k = 0; k < PROOFSIZE; k++) // find unique other i with same us[i]
        if (k != j && us[k] == us[j]) {
          if (i != j)
            return false;
          i = k;
      }
      if (i == j)
        return false;
      n -= 2;
    } while (i != 0);
    return n == 0;
  }

  public static void main(String argv[]) {
    String header = "";
    int i, easipct = 50;
    for (i = 0; i < argv.length; i++) {
      if (argv[i].equals("-e")) {
        easipct = Integer.parseInt(argv[++i]);
      } else if (argv[i].equals("-h")) {
        header = argv[++i];
      }
    }
    System.out.println("Verifying size " + PROOFSIZE + " proof for cuckoo" + NODEBITS + "(\"" + header + "\") with " + easipct + "% edges");
    Scanner sc = new Scanner(System.in);
    sc.next();
    int nonces[] = new int[PROOFSIZE];
    for (int n = 0; n < PROOFSIZE; n++) {
      nonces[n] = Integer.parseInt(sc.next(), 16);
    }
    int easiness = (int)(easipct * NNODES / 100L);
    Cuckoo cuckoo = new Cuckoo(header.getBytes());
    Boolean ok = cuckoo.verify(nonces, easiness);
    if (!ok) {
      System.out.println("FAILED");
      System.exit(1);
    }
    System.out.print("Verified with cyclehash ");
    ByteBuffer buf = ByteBuffer.allocate(PROOFSIZE*8);
    buf.order(ByteOrder.LITTLE_ENDIAN).asIntBuffer().put(nonces);
    byte[] cyclehash;
    try {
      cyclehash = MessageDigest.getInstance("SHA-256").digest(buf.array());
      for (i=0; i<32; i++)
        System.out.print(String.format("%02x",((int)cyclehash[i] & 0xff)));
      System.out.println("");
    } catch(NoSuchAlgorithmException e) {
      System.out.println(e);
      System.exit(1);
    }
  }
}
