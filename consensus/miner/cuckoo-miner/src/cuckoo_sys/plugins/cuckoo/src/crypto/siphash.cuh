#pragma once

#if (__CUDA_ARCH__  >= 320) // make rotate-left use funnel shifter, 3% speed gain
typedef uint2 sip64;

static __device__ __forceinline__ sip64 operator^ (uint2 a, uint2 b) {
  return make_uint2(a.x ^ b.x, a.y ^ b.y);
}
static __device__ __forceinline__ void operator^= (uint2 &a, uint2 b) {
  a.x ^= b.x, a.y ^= b.y;
}
static __device__ __forceinline__ void operator+= (uint2 &a, uint2 b) {
  asm("{\n\tadd.cc.u32 %0,%2,%4;\n\taddc.u32 %1,%3,%5;\n\t}\n\t"
    : "=r"(a.x), "=r"(a.y) : "r"(a.x), "r"(a.y), "r"(b.x), "r"(b.y));
}

__inline__ __device__ sip64 rotl(const sip64 a, const int offset) {
  sip64 result;
  if (offset >= 32) {
    asm("shf.l.wrap.b32 %0, %1, %2, %3;" : "=r"(result.x) : "r"(a.x), "r"(a.y), "r"(offset));
    asm("shf.l.wrap.b32 %0, %1, %2, %3;" : "=r"(result.y) : "r"(a.y), "r"(a.x), "r"(offset));
  } else {
    asm("shf.l.wrap.b32 %0, %1, %2, %3;" : "=r"(result.x) : "r"(a.y), "r"(a.x), "r"(offset));
    asm("shf.l.wrap.b32 %0, %1, %2, %3;" : "=r"(result.y) : "r"(a.x), "r"(a.y), "r"(offset));
  }
  return result;
}
__device__ __forceinline__ sip64 vectorize(const uint64_t x) {
  uint2 result;
  asm("mov.b64 {%0,%1},%2; \n\t" : "=r"(result.x), "=r"(result.y) : "l"(x));
  return result;
}
__device__ __forceinline__ uint64_t devectorize(sip64 x) {
  uint64_t result;
  asm("mov.b64 %0,{%1,%2}; \n\t" : "=l"(result) : "r"(x.x), "r"(x.y));
  return result;
}

#else

typedef uint64_t sip64;

__inline__ __device__ sip64 rotl(const sip64 a, const int offset) {
  return (a << offset) | (a >> (64 - offset));
}
__device__ __forceinline__ sip64 vectorize(const uint64_t x) {
  return x;
}
__device__ __forceinline__ uint64_t devectorize(sip64 x) {
  return x;
}

#endif

template <int rotE = 21>
class diphash_state {
public:
  sip64 v0;
  sip64 v1;
  sip64 v2;
  sip64 v3;

  __device__ diphash_state(const siphash_keys &sk) {
    v0 = vectorize(sk.k0); v1 = vectorize(sk.k1); v2 = vectorize(sk.k2); v3 = vectorize(sk.k3);
  }
  __device__ uint64_t xor_lanes() {
    return devectorize((v0 ^ v1) ^ (v2  ^ v3));
  }
  __device__ void xor_with(const diphash_state &x) {
    v0 ^= x.v0;
    v1 ^= x.v1;
    v2 ^= x.v2;
    v3 ^= x.v3;
  }
  __device__ void dip_round() {
    v0 += v1; v2 += v3; v1 = rotl(v1,13);
    v3 = rotl(v3,16); v1 ^= v0; v3 ^= v2;
    v0 = rotl(v0,32); v2 += v1; v0 += v3;
    v1 = rotl(v1,17);   v3 = rotl(v3,rotE);
    v1 ^= v2; v3 ^= v0; v2 = rotl(v2,32);
  }
  __device__ void hash24(const uint64_t nonce) {
    v3 ^= vectorize(nonce);
    dip_round(); dip_round();
    v0 ^= vectorize(nonce);
    v2 ^= vectorize(0xff);
    dip_round(); dip_round(); dip_round(); dip_round();
  }
};
 
__device__ uint64_t dipnode(const siphash_keys &sip_keys, const uint64_t nonce, const int uorv) {
  diphash_state<> v(sip_keys);
  v.hash24((nonce << 1) | uorv);
  return v.xor_lanes() & NODE1MASK;
}
