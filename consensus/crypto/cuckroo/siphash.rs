// Copyright 2018 The Grin Developers
//! Simple implementation of the siphash 2-4 hashing function from
//! Jean-Philippe Aumasson and Daniel J. Bernstein.

// Parameters to the siphash block algorithm. Used by Cuckaroo but can be
// seen as a generic way to derive a hash within a block of them.
const SIPHASH_BLOCK_BITS: u64 = 6;
const SIPHASH_BLOCK_SIZE: u64 = 1 << SIPHASH_BLOCK_BITS;
const SIPHASH_BLOCK_MASK: u64 = SIPHASH_BLOCK_SIZE - 1;

// helper macro for left rotation
macro_rules! rotl {
    ($num:expr, $shift:expr) => {
        $num = ($num << $shift) | ($num >> (64 - $shift));
    };
}

/// Utility function to compute a single siphash 2-4 based on a seed and
/// a nonce
pub fn siphash24(v: &[u64; 4], nonce: u64) -> u64 {
    let mut siphash = SipHash24::new(v);
    siphash.hash(nonce);
    siphash.digest()
}

/// Builds a block of siphash values by repeatedly hashing from the nonce
/// truncated to its closest block start, up to the end of the block. Returns
/// the resulting hash at the nonce's position.
pub fn siphash_block(v: &[u64; 4], nonce: u64) -> u64 {
    // beginning of the block of hashes
    let nonce0 = nonce & !SIPHASH_BLOCK_MASK;
    let mut nonce_hash = 0;

    // repeated hashing over the whole block
    let mut siphash = SipHash24::new(v);
    for n in nonce0..(nonce0 + SIPHASH_BLOCK_SIZE) {
        siphash.hash(n);
        if n == nonce {
            nonce_hash = siphash.digest();
        }
    }
    // xor the nonce with the last hash to force hashing the whole block
    // unless the nonce is last in the block
    if nonce == nonce0 + SIPHASH_BLOCK_MASK {
        return siphash.digest();
    } else {
        return nonce_hash ^ siphash.digest();
    }
}

/// Implements siphash 2-4 specialized for a 4 u64 array key and a u64 nonce
/// that can be used for a single or multiple repeated hashing.
///
/// The siphash structure is represented by a vector of four 64-bits words
/// that we simply reference by their position. A hashing round consists of
/// a series of arithmetic operations on those words, while the resulting
/// hash digest is an xor of xor on them.
///
/// Note that this implementation is only secure if it's already fed words
/// output from a previous hash function (in our case blake2).
pub struct SipHash24(u64, u64, u64, u64);

impl SipHash24 {
    /// Create a new siphash context
    pub fn new(v: &[u64; 4]) -> SipHash24 {
        SipHash24(v[0], v[1], v[2], v[3])
    }

    /// One siphash24 hashing, consisting of 2 and then 4 rounds
    pub fn hash(&mut self, nonce: u64) {
        self.3 ^= nonce;
        self.round();
        self.round();

        self.0 ^= nonce;
        self.2 ^= 0xff;

        for _ in 0..4 {
            self.round();
        }
    }

    /// Resulting hash digest
    pub fn digest(&self) -> u64 {
        (self.0 ^ self.1) ^ (self.2 ^ self.3)
    }

    fn round(&mut self) {
        self.0 = self.0.wrapping_add(self.1);
        self.2 = self.2.wrapping_add(self.3);
        rotl!(self.1, 13);
        rotl!(self.3, 16);
        self.1 ^= self.0;
        self.3 ^= self.2;
        rotl!(self.0, 32);
        self.2 = self.2.wrapping_add(self.1);
        self.0 = self.0.wrapping_add(self.3);
        rotl!(self.1, 17);
        rotl!(self.3, 21);
        self.1 ^= self.2;
        self.3 ^= self.0;
        rotl!(self.2, 32);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_some() {
        assert_eq!(siphash24(&[1, 2, 3, 4], 10), 928382149599306901);
        assert_eq!(siphash24(&[1, 2, 3, 4], 111), 10524991083049122233);
        assert_eq!(siphash24(&[9, 7, 6, 7], 12), 1305683875471634734);
        assert_eq!(siphash24(&[9, 7, 6, 7], 10), 11589833042187638814);
    }

    #[test]
    fn hash_block() {
        assert_eq!(siphash_block(&[1, 2, 3, 4], 10), 1182162244994096396);
        assert_eq!(siphash_block(&[1, 2, 3, 4], 123), 11303676240481718781);
        assert_eq!(siphash_block(&[9, 7, 6, 7], 12), 4886136884237259030);
    }
}
