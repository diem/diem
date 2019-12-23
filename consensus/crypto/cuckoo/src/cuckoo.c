#include "portable_endian.h" // for htole32/64
#include <stdint.h>          // for types uint32_t,uint64_t
#include <stdlib.h>
#include <stdio.h>
#define rotl(x, b) ((x) << (b)) | ((x) >> (64 - (b)))

#define MAX_CN 12
#define MAX_EN 1 << 15
#define MAX_GN MAX_EN << 1

// set siphash keys from 32 byte char array
#define setkeys()                                                              \
  k0 = le64toh(((uint64_t *)mesg)[0]);                                         \
  k1 = le64toh(((uint64_t *)mesg)[1]);                                         \
  k2 = le64toh(((uint64_t *)mesg)[2]);                                         \
  k3 = le64toh(((uint64_t *)mesg)[3]);

#define sip_round()                                                            \
  v0 += v1;                                                                    \
  v2 += v3;                                                                    \
  v1 = rotl(v1, 13);                                                           \
  v3 = rotl(v3, 16);                                                           \
  v1 ^= v0;                                                                    \
  v3 ^= v2;                                                                    \
  v0 = rotl(v0, 32);                                                           \
  v2 += v1;                                                                    \
  v0 += v3;                                                                    \
  v1 = rotl(v1, 17);                                                           \
  v3 = rotl(v3, 21);                                                           \
  v1 ^= v2;                                                                    \
  v3 ^= v0;                                                                    \
  v2 = rotl(v2, 32);

#define siphash24(nonce)                                                       \
  v0 = k0;                                                                     \
  v1 = k1;                                                                     \
  v2 = k2;                                                                     \
  v3 = k3;                                                                     \
  v3 ^= (nonce);                                                               \
  sip_round();                                                                 \
  sip_round();                                                                 \
  v0 ^= (nonce);                                                               \
  v2 ^= 0xff;                                                                  \
  sip_round();                                                                 \
  sip_round();                                                                 \
  sip_round();                                                                 \
  sip_round();                                                                 \
  h = ((v0 ^ v1 ^ v2 ^ v3) & mask) << 1;

uint32_t c_solve(uint32_t *prof, const uint8_t *mesg, const uint64_t max_edge,
                 const uint32_t cycle_len) {
  uint64_t graph_size = max_edge << 1;
  uint64_t mask = max_edge - 1;
  int graph[MAX_GN];
  int V[MAX_EN];
  int U[MAX_EN];
  int path[MAX_CN];

  uint64_t k0, k1, k2, k3;
  uint64_t v0, v1, v2, v3;
  uint64_t h;
  uint64_t i;

  setkeys();

  for (i = 0; i < graph_size; ++i) {
    graph[i] = -1;
  }

  for (i = 0; i < max_edge; ++i) {
    siphash24((i << 1));
    U[i] = h;
    siphash24((i << 1) | 1);
    V[i] = h | 1;
  }

  for (i = 0; i < max_edge; ++i) {
    int u = U[i];
    int v = V[i];

    int pre = -1;
    int cur = u;
    int next;
    while (cur != -1) {
      next = graph[cur];
      graph[cur] = pre;
      pre = cur;
      cur = next;
    }

    uint32_t m = 0;
    cur = v;
    while (graph[cur] != -1 && m < cycle_len) {
      cur = graph[cur];
      ++m;
    }

    if (cur != u) {
      graph[u] = v;
    } else if (m == cycle_len - 1) {
      uint32_t j;

      cur = v;
      for (j = 0; j <= m; ++j) {
        path[j] = cur;
        cur = graph[cur];
      }

      for (j = 0; j < graph_size; ++j) {
        graph[j] = -1;
      }

      for (j = 1; j <= m; ++j) {
        graph[path[j]] = path[j - 1];
      }

      uint32_t k = 0;
      uint32_t b = cycle_len - 1;
      for (j = 0; k < b; ++j) {
        int u = U[j];
        int v = V[j];

        if (graph[u] == v) {
          prof[k] = j;
          graph[u] = -1;
          ++k;
        } else if (graph[v] == u) {
          prof[k] = j;
          graph[v] = -1;
          ++k;
        }
      }
      prof[k] = i;
      return 1;
    }
  }

  return 0;
}
