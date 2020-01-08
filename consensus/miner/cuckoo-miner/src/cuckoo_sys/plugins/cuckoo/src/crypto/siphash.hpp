#pragma once

#include <stdint.h>    // for types uint32_t,uint64_t
#include "portable_endian.h"    // for htole32/64

// generalize siphash by using a quadruple of 64-bit keys,
class siphash_keys {
public:
  uint64_t k0;
  uint64_t k1;
  uint64_t k2;
  uint64_t k3;

  void setkeys(const char *keybuf);

  uint64_t siphash24(const uint64_t nonce) const;
};

template <int rotE = 21>
class siphash_state {
public:
  uint64_t v0;
  uint64_t v1;
  uint64_t v2;
  uint64_t v3;

  siphash_state(const siphash_keys &sk) {
    v0 = sk.k0; v1 = sk.k1; v2 = sk.k2; v3 = sk.k3;
  }
  uint64_t xor_lanes() {
    return (v0 ^ v1) ^ (v2  ^ v3);
  }
  void xor_with(const siphash_state &x) {
    v0 ^= x.v0;
    v1 ^= x.v1;
    v2 ^= x.v2;
    v3 ^= x.v3;
  }
  static uint64_t rotl(uint64_t x, uint64_t b) {
    return (x << b) | (x >> (64 - b));
  }
  void sip_round() {
    v0 += v1; v2 += v3; v1 = rotl(v1,13);
    v3 = rotl(v3,16); v1 ^= v0; v3 ^= v2;
    v0 = rotl(v0,32); v2 += v1; v0 += v3;
    v1 = rotl(v1,17);   v3 = rotl(v3,rotE);
    v1 ^= v2; v3 ^= v0; v2 = rotl(v2,32);
  }
  void hash24(const uint64_t nonce) {
    v3 ^= nonce;
    sip_round(); sip_round();
    v0 ^= nonce;
    v2 ^= 0xff;
    sip_round(); sip_round(); sip_round(); sip_round();
  }
};
 
// set siphash keys from 32 byte char array
void siphash_keys::setkeys(const char *keybuf) {
  k0 = htole64(((uint64_t *)keybuf)[0]);
  k1 = htole64(((uint64_t *)keybuf)[1]);
  k2 = htole64(((uint64_t *)keybuf)[2]);
  k3 = htole64(((uint64_t *)keybuf)[3]);
}

uint64_t siphash_keys::siphash24(const uint64_t nonce) const {
  siphash_state<> v(*this);
  v.hash24(nonce);
  return v.xor_lanes();
}
