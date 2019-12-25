// Cuckaroom Cycle, a memory-hard proof-of-work by John Tromp and team Grin
// Copyright (c) 2019-2020 Jiri Photon Vadura and John Tromp
// This GGM miner file is covered by the FAIR MINING license

//Includes for IntelliSense
#define _SIZE_T_DEFINED
#ifndef __CUDACC__
#define __CUDACC__
#endif
#ifndef __cplusplus
#define __cplusplus
#endif

#include <stdio.h>
#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;

#ifndef XBITS
#define XBITS 6
#endif

const u32 NX = 1 << XBITS;
const u32 XMASK = NX - 1;
const u32 NX2 = NX * NX;
const u32 YBITS = XBITS;
const u32 NY = 1 << YBITS;
const u32 YZBITS = EDGEBITS - XBITS;
const u32 ZBITS = YZBITS - YBITS;
const u32 NZ = 1 << ZBITS;
const u32 ZMASK = NZ - 1;

__device__ __forceinline__ uint2 ld_cs_u32_v2(const uint2 *p_src)
{
  uint2 n_result;
  asm("ld.global.cs.v2.u32 {%0,%1}, [%2];"  : "=r"(n_result.x), "=r"(n_result.y) : "l"(p_src));
  return n_result;
}

__device__ __forceinline__ void st_cg_u32_v2(uint2 *p_dest, const uint2 n_value)
{
  asm("st.global.cg.v2.u32 [%0], {%1, %2};" :: "l"(p_dest), "r"(n_value.x), "r"(n_value.y));
}

__device__ __forceinline__ void st_cg_u32_v4(uint4 *p_dest, const uint4 n_value)
{
  asm("st.global.cg.v4.u32 [%0], {%1, %2, %3, %4};" :: "l"(p_dest), "r"(n_value.x), "r"(n_value.y), "r"(n_value.z), "r"(n_value.w));
}

__device__ __forceinline__  void setbit(u32 *bitmap, const int index)
{
  const u32 word = index / 32;
  const u32  bit = index % 32;
  const u32 mask = 1 << bit;
  atomicOr(&bitmap[word], mask);
}

__device__ __forceinline__  bool testbit(u32 *bitmap, const int index)
{
  const u32 word = index / 32;
  const u32  bit = index % 32;
  return (bitmap[word] >> bit) & 1;
}

__constant__ siphash_keys dipkeys;
__constant__ u64 recovery[42];

#define ROTL(x,b) ( ((x) << (b)) | ( (x) >> (64 - (b))) )
#define SIPROUND {\
  v0 += v1; v2 += v3; v1 = ROTL(v1,13); \
  v3 = ROTL(v3,16); v1 ^= v0; v3 ^= v2; \
  v0 = ROTL(v0,32); v2 += v1; v0 += v3; \
  v1 = ROTL(v1,17);   v3 = ROTL(v3,21); \
  v1 ^= v2; v3 ^= v0; v2 = ROTL(v2,32); \
}
#define SIPBLOCK(b) {\
  v3 ^= (b);\
  for (int r = 0; r < 2; r++)\
  SIPROUND;\
  v0 ^= (b);\
  v2 ^= 0xff;\
  for (int r = 0; r < 4; r++)\
  SIPROUND;\
}

#define NX2NA (NX2 / NA)
#define YDUMP(E) {\
  u32 edgey = ((E) >> 32) & NODEMASK;\
  int bucket = edgey >> ZBITS;\
  for (u32 ret = atomicCAS(&magazine[bucket], 0, edgey); ret; ) {\
    u32 ret2 = atomicCAS(&magazine[bucket], ret, 0);\
    if (ret2 == ret) {\
      int position = (min(bktOutSize - 2, (atomicAdd(indexes + bucket, 2))));\
      int idx = ((offi * NX2 + bucket) * bktOutSize + position) / 2;\
      buffer[idx] = make_uint2(ret2, edgey);\
      break;\
    }\
    ret = ret2 ? ret2 : atomicCAS(&magazine[bucket], 0, edgey);\
  }\
}

template<int tpb, int bktOutSize>
__global__  void YSeed(uint2 * __restrict__ buffer, u32 * __restrict__ indexes, const u32 offi)
{
  const int gid = blockDim.x * blockIdx.x + threadIdx.x;
  const int lid = threadIdx.x;
  const int nthreads = gridDim.x * tpb;

  const int nloops = (NEDGES / NA / EDGE_BLOCK_SIZE - gid + nthreads-1) / nthreads;
  ulonglong4 sipblockL[EDGE_BLOCK_SIZE/4];
  __shared__ u32 magazine[NX2];

  uint64_t v0, v1, v2, v3;

#if tpb && NX2 % tpb == 0
  for (int i = 0; i < NX2/tpb; i++)
#else
  for (int i = 0; i < (NX2 - lid + tpb-1) / tpb; i++)
#endif
    magazine[lid + tpb * i] = 0;

  __syncthreads();

  for (int i = 0; i < nloops; i++) {
    u64 blockNonce = offi * (NEDGES / NA) + (gid * nloops * EDGE_BLOCK_SIZE + i * EDGE_BLOCK_SIZE);

    v0 = dipkeys.k0;
    v1 = dipkeys.k1;
    v2 = dipkeys.k2;
    v3 = dipkeys.k3;

    // do one block of 64 edges
    for (int b = 0; b < EDGE_BLOCK_SIZE; b += 4) {
      SIPBLOCK(blockNonce + b);
      u64 e0 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 1);
      u64 e1 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 2);
      u64 e2 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 3);
      u64 e3 = (v0 ^ v1) ^ (v2  ^ v3);
      sipblockL[b / 4] = make_ulonglong4(e0, e1, e2, e3);
    }

    u64 last = 0;

    for (int s = EDGE_BLOCK_SIZE/4; s--; ) {
      ulonglong4 edges = sipblockL[s];
      YDUMP(last ^= edges.w);
      YDUMP(last ^= edges.z);
      YDUMP(last ^= edges.y);
      YDUMP(last ^= edges.x);
    }
  }

  __syncthreads();

  for (int i = 0; i < NX2/tpb; i++) {
    int bucket = lid + (tpb * i);
    u32 edgey = magazine[bucket];
    if (edgey != 0) {
      int position = (min(bktOutSize - 2, (atomicAdd(indexes + bucket, 2))));
      int idx = ((offi * NX2 + bucket) * bktOutSize + position) / 2;
      buffer[idx] = make_uint2(edgey, 0);
    }
  }
}

template<int tpb, int bktInSize>
__global__  void NodemapRound(const u32 *source, const u32 *sourceIndexes, u32 *nodemap)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;

  __shared__ u32 ecounters[NZ/32];

  for (int i = 0; i < NZ/32/tpb; i++)
    ecounters[lid + i * tpb] = 0;

  int nloops[NA];
  for (int a = 0; a < NA; a++)
    nloops[a] = (min(sourceIndexes[a * NX2 + group], bktInSize) - lid + tpb-1) / tpb;

  source += lid;

  __syncthreads();

  for (int a = 0; a < NA; a++) {
    const int delta = (a * NX2 + group) * bktInSize;
    for (int i = 0; i < nloops[a]; i++) {
      u32 edgey = source[delta + i * tpb];
      setbit(ecounters, edgey & ZMASK);
    }
  }

  __syncthreads();

  nodemap += group * (NZ / 32);
  for (int i = 0; i < NZ/32/tpb; i++) {
    const int idx = lid + i * tpb;
    nodemap[idx] = ecounters[idx];
  }
}

#define DUMP(E) {\
  u64 lookup = (E);\
  const uint2 edge1 = make_uint2(lookup & NODEMASK, (lookup >> 32) & NODEMASK);\
  int bucket = edge1.x >> ZBITS;\
  u64 edge64 = (((u64)edge1.y) << 32) | edge1.x;\
  for (u64 ret = atomicCAS(&magazine[bucket], 0, edge64); ret; ) {\
    u64 ret2 = atomicCAS(&magazine[bucket], ret, 0);\
    if (ret2 == ret) {\
      int block = bucket / NX2NA;\
      int shift = (bktOutSize * NX2) * block;\
      int position = (min(bktOutSize - 2, (atomicAdd(indexes + bucket, 2))));\
      int idx = (shift+((bucket%NX2NA) * (bktOutSize) + position)) / 2;\
      buffer[idx] = make_uint4(ret, ret >> 32, edge1.x, edge1.y);\
      break;\
    }\
    ret = ret2 ? ret2 : atomicCAS(&magazine[bucket], 0, edge64);\
  }\
}

template<int tpb, int bktOutSize>
__global__  void FluffySeed(uint4 * __restrict__ buffer, u32 * __restrict__ indexes, const u32 offset)
{
  const int gid = blockDim.x * blockIdx.x + threadIdx.x;
  const int lid = threadIdx.x;
  const int nthreads = gridDim.x * tpb;

  const int nloops = (NEDGES / NA / EDGE_BLOCK_SIZE - gid + nthreads-1) / nthreads;
  ulonglong4 sipblockL[EDGE_BLOCK_SIZE/4];
  __shared__ unsigned long long magazine[NX2];

  uint64_t v0, v1, v2, v3;

#if tpb && NX2 % tpb == 0
  for (int i = 0; i < NX2/tpb; i++)
#else
  for (int i = 0; i < (NX2 - lid + tpb-1) / tpb; i++)
#endif
    magazine[lid + tpb * i] = 0;

  __syncthreads();

  for (int i = 0; i < nloops; i++) {
    u64 blockNonce = offset + (gid * nloops * EDGE_BLOCK_SIZE + i * EDGE_BLOCK_SIZE);

    v0 = dipkeys.k0;
    v1 = dipkeys.k1;
    v2 = dipkeys.k2;
    v3 = dipkeys.k3;

    // do one block of 64 edges
    for (int b = 0; b < EDGE_BLOCK_SIZE; b += 4) {
      SIPBLOCK(blockNonce + b);
      u64 e0 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 1);
      u64 e1 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 2);
      u64 e2 = (v0 ^ v1) ^ (v2  ^ v3);
      SIPBLOCK(blockNonce + b + 3);
      u64 e3 = (v0 ^ v1) ^ (v2  ^ v3);
      sipblockL[b / 4] = make_ulonglong4(e0, e1, e2, e3);
    }

    u64 last = 0;

    for (int s = EDGE_BLOCK_SIZE/4; s--; ) {
      ulonglong4 edges = sipblockL[s];
      DUMP(last ^= edges.w);
      DUMP(last ^= edges.z);
      DUMP(last ^= edges.y);
      DUMP(last ^= edges.x);
    }
  }

  __syncthreads();

  for (int i = 0; i < NX2/tpb; i++) {
    int bucket = lid + (tpb * i);
    u64 edge = magazine[bucket];
    if (edge != 0) {
      int block = bucket / NX2NA;
      int shift = (bktOutSize * NX2) * block;
      int position = (min(bktOutSize - 2, (atomicAdd(indexes + bucket, 2))));
      int idx = (shift + ((bucket % NX2NA) * bktOutSize + position)) / 2;
      buffer[idx] = make_uint4(edge, edge >> 32, 0, 0);
    }
  }
}

template<int tpb, int bktInSize, int bktOutSize>
__global__  void FluffyRound_A1(const uint2 *source, uint4 *destination, const u32 *sourceIndexes, u32 *destinationIndexes, u32 *nodemap, const int offset)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;

  __shared__ u32 ecounters[NZ/32];
  __shared__ unsigned long long magazine[NX2];

  int nloops[NA];

  for (int i = 0; i < NZ/32/tpb; i++)
    ecounters[lid + i * tpb] = 0;

#if tpb && NX2 % tpb == 0
  for (int i = 0; i < NX2/tpb; i++)
#else
  for (int i = 0; i < (NX2 - lid + tpb-1) / tpb; i++)
#endif
    magazine[lid + tpb * i] = 0;

  for (int a = 0; a < NA; a++)
    nloops[a] = (min(sourceIndexes[a * NX2 + offset + group], bktInSize) - lid + tpb-1) / tpb;

  const int rowOffset = offset * NA;
  source += bktInSize * (rowOffset + group) + lid;

  __syncthreads();

  for (int a = 0; a < NA; a++) {
    const int delta = a * NX2NA * bktInSize;
    for (int i = 0; i < nloops[a]; i++) {
      uint2 edge = source[delta + i * tpb];
      if (edge.x == 0 && edge.y == 0) continue;
      if (testbit(nodemap, edge.x)) {
        setbit(ecounters, edge.x & ZMASK);
        int bucket = edge.y >> ZBITS;
        u64 edge64 = (((u64)edge.y) << 32) | edge.x;
        for (u64 ret = atomicCAS(&magazine[bucket], 0, edge64); ret; ) {
          u64 ret2 = atomicCAS(&magazine[bucket], ret, 0);
          if (ret2 == ret) {
            int bktIdx = min(atomicAdd(destinationIndexes + bucket + rowOffset, 2), bktOutSize - 2);
            destination[ ((bucket * bktOutSize) + bktIdx) / 2] = make_uint4(ret >> 32, ret, edge.y, edge.x);
            break;
          }
          ret = ret2 ? ret2 : atomicCAS(&magazine[bucket], 0, edge64);
        }
      }
    }
  }

  __syncthreads();

  nodemap += (offset + group) * (NZ / 32);
  for (int i = 0; i < NZ/32/tpb; i++) {
    const int idx = lid + i * tpb;
    nodemap[idx] = ecounters[idx];
  }

  for (int i = 0; i < NX2/tpb; i++) {
    int bucket = lid + tpb * i;
    u64 edge = magazine[bucket];
    if (edge != 0) {
      int bktIdx = min(atomicAdd(destinationIndexes + bucket + rowOffset, 2), bktOutSize - 2);
      destination[((bucket * bktOutSize) + bktIdx) / 2] = make_uint4(edge >> 32, edge, 0, 0);
    }
  }
}

template<int tpb, int bktInSize, int bktOutSize>
__global__  void FluffyRound_A2(const uint2 *source, uint2 *destination, const u32 *sourceIndexes, u32 *destinationIndexes, u32 *nodemap)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;

  __shared__ u32 ecounters[NZ/32];

  const int nloops = (min(sourceIndexes[group], bktInSize) - lid + tpb-1) / tpb;

  source += bktInSize * group + lid;

  for (int i = 0; i < NZ/32/tpb; i++)
    ecounters[lid + (tpb * i)] = 0;

  __syncthreads();

  for (int i = nloops; --i >= 0;) {
    uint2 edge = source[i * tpb];
    if (testbit(nodemap, edge.x)) {
      setbit(ecounters, edge.x & ZMASK);
      const int bucket = edge.y >> ZBITS;
      const int bktIdx = min(atomicAdd(destinationIndexes + bucket, 1), bktOutSize - 1);
      st_cg_u32_v2(&destination[(bucket * bktOutSize) + bktIdx], make_uint2(edge.y, edge.x));
    }
  }

  __syncthreads();

  nodemap += group * (NZ / 32);
  for (int i = 0; i < NZ/32/tpb; i++) {
    const int idx = lid + i * tpb;
    nodemap[idx] = ecounters[idx];
  }
}

template<int tpb, int bktInSize, int bktOutSize>
__global__  void FluffyRound_A3(uint2 *source, uint2 *destination, const u32 *sourceIndexes, u32 *destinationIndexes, u32 *nodemap)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;

  __shared__ u32 ecounters[NZ/32];
  int nloops[NA];

  for (int i = 0; i < NZ/32/tpb; i++)
    ecounters[lid + (i * tpb)] = 0;

  for (int a = 0; a < NA; a++)
    nloops[a] = (min(sourceIndexes[group + a*NX2], bktInSize) - lid + tpb-1) / tpb;

  source += bktInSize * group + lid;

  __syncthreads();

  for (int a = 0; a < NA; a++) {
    const int delta = a * bktInSize * NX2;
    for (int i = 0; i < nloops[a]; i++) {
      uint2 edge = ld_cs_u32_v2(&source[delta + i * tpb]);
      if (edge.x == 0 && edge.y == 0) continue;
      if (testbit(nodemap, edge.x)) {
        setbit(ecounters, edge.x & ZMASK);
        const int bucket = edge.y >> ZBITS;
        const int bktIdx = min(atomicAdd(destinationIndexes + bucket, 1), bktOutSize - 1);
        st_cg_u32_v2(&destination[(bucket * bktOutSize) + bktIdx], make_uint2(edge.y, edge.x));
      }
    }
  }

  __syncthreads();

  nodemap += group * (NZ / 32);
  for (int i = 0; i < NZ/32/tpb; i++) {
    const int idx = lid + i * tpb;
    nodemap[idx] = ecounters[idx];
  }
}

#ifndef LISTBITS
#define LISTBITS 12
#endif

const u32 NLISTS  = 1 << LISTBITS;
const u32 LISTMASK = NLISTS - 1;

#ifndef NNEXTS
#define NNEXTS NLISTS
#endif

template<int tpb, int bktInSize, int bktOutSize>
__global__  void Tag_Relay(const uint2 *source, uint2 *destination, const u32 *sourceIndexes, u32 *destinationIndexes, bool TAGGED)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;

  __shared__ u32 lists[NLISTS];
  __shared__ u32 nexts[NNEXTS];

  const int nloops = (min(sourceIndexes[group], NNEXTS) - lid + tpb-1) / tpb;

  source += bktInSize * group;

  for (int i = 0; i < NLISTS/tpb; i++)
    lists[i * tpb + lid] = ~0;

  __syncthreads();

  for (int i = 0; i < nloops; i++) {
    const u32 index = i * tpb + lid;
    const u32 list = source[index].x & LISTMASK;
    nexts[index] = atomicExch(&lists[list], index);
  }

  __syncthreads();

  for (int i = nloops; --i >= 0;) {
    const u32 index = i * tpb + lid;
    const uint2 edge = source[index];
    if (edge.y & NEDGES) continue; // copies don't relay
    u32 bucket = edge.y >> ZBITS;
    u32 copybit = 0;
    const u32 list = (edge.x & LISTMASK) ^ 1;
    for (u32 idx = lists[list]; idx != ~0; idx = nexts[idx]) {
      uint2 tagged = source[idx];
      if ((tagged.x ^ edge.x ^ 1) & ZMASK) continue;
      u32 bktIdx = min(atomicAdd(destinationIndexes + bucket, 1), bktOutSize - 1);
      u32 tag = TAGGED ? tagged.x >> ZBITS : tagged.y >> 1;
      destination[(bucket * bktOutSize) + bktIdx] = make_uint2((tag << ZBITS) | (edge.y & ZMASK), copybit | (group << ZBITS) | (edge.x & ZMASK));
      copybit = NEDGES;
    }
  }
}

template<int tpb, int bktInSize>
__global__  void FluffyTail(const uint2 *source, uint2 *destination, const u32 *sourceIndexes, u32 *destinationIndexes)
{
  const int lid = threadIdx.x;
  const int group = blockIdx.x;
  __shared__ u32 destIdx;

  const u32 myEdges = sourceIndexes[group];
  const int nloops = (myEdges - lid + tpb-1) / tpb;

  if (lid == 0)
    destIdx = atomicAdd(destinationIndexes, myEdges);

  __syncthreads();

  source += bktInSize * group;
  destination += destIdx;
  for (int i = 0; i < nloops; i++) {
    const u32 index = i * tpb + lid;
    destination[index] = source[index];
  }
}

__global__  void FluffyRecovery(u32 * indexes)
{
  const int gid = blockDim.x * blockIdx.x + threadIdx.x;
  const int lid = threadIdx.x;
  const int nthreads = gridDim.x * blockDim.x;

  __shared__ u32 nonces[PROOFSIZE];
  u64 sipblock[EDGE_BLOCK_SIZE];

  uint64_t v0;
  uint64_t v1;
  uint64_t v2;
  uint64_t v3;

  const int nloops = (NEDGES / EDGE_BLOCK_SIZE - gid + nthreads-1) / nthreads;
  if (lid < PROOFSIZE) nonces[lid] = 0;

  __syncthreads();

  for (int i = 0; i < nloops; i++) {
    u64 blockNonce = gid * nloops * EDGE_BLOCK_SIZE + i * EDGE_BLOCK_SIZE;

    v0 = dipkeys.k0;
    v1 = dipkeys.k1;
    v2 = dipkeys.k2;
    v3 = dipkeys.k3;

    for (u32 b = 0; b < EDGE_BLOCK_SIZE; b++) {
      v3 ^= blockNonce + b;
      for (int r = 0; r < 2; r++)
        SIPROUND;
      v0 ^= blockNonce + b;
      v2 ^= 0xff;
      for (int r = 0; r < 4; r++)
        SIPROUND;

      sipblock[b] = (v0 ^ v1) ^ (v2  ^ v3);
    }

    u64 last = 0;
    const u64 NODEMASK2 = NODEMASK | ((u64)NODEMASK << 32);
    u64 lookup;
    for (int s = EDGE_BLOCK_SIZE; --s >= 0; last = lookup) {
      lookup = sipblock[s] ^ last;
      for (int i = 0; i < PROOFSIZE; i++) {
        if (recovery[i] == (lookup & NODEMASK2))
          nonces[i] = blockNonce + s;
      }
    }
  }

  __syncthreads();

  if (lid < PROOFSIZE) {
    if (nonces[lid] > 0)
      indexes[lid] = nonces[lid];
  }
}
