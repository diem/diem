// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines traits and implementations of
//! [cryptographic hash functions](https://en.wikipedia.org/wiki/Cryptographic_hash_function)
//! for the Libra project.
//!
//! It is designed to help authors protect against two types of real world attacks:
//!
//! 1. **Domain Ambiguity**: imagine that Alice has a private key and is using
//!    two different applications, X and Y. X asks Alice to sign a message saying
//!    "I am Alice". naturally, Alice is willing to sign this message, since she
//!    is in fact Alice. However, unbeknownst to Alice, in application Y,
//!    messages beginning with the letter "I" represent transfers. " am "
//!    represents a transfer of 500 coins and "Alice" can be interpreted as a
//!    destination address. When Alice signed the message she needed to be
//!    aware of how other applications might interpret that message.
//!
//! 2. **Format Ambiguity**: imagine a program that hashes a pair of strings.
//!    To hash the strings `a` and `b` it hashes `a + "||" + b`. The pair of
//!    strings `a="foo||", b = "bar"` and `a="foo", b = "||bar"` result in the
//!    same input to the hash function and therefore the same hash. This
//!    creates a collision.
//!
//! # Examples
//!
//! ```
//! use libra_crypto::hash::{CryptoHasher, TestOnlyHasher};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hash_value = hasher.finish();
//! ```
//! The output is of type [`HashValue`], which can be used as an input for signing.
//!
//! # Implementing new hashers
//!
//! ## The automatic way
//!
//! For any new structure `MyNewStruct` that needs to be hashed, the developer
//! should use the [`CryptoHasher` derive macro](https://doc.rust-lang.org/reference/procedural-macros.html).
//!
//! ```ignore
//! #[derive(CryptoHasher)]
//! struct MyNewStruct {
//!   ...
//! }
//! ```
//!
//! The macro will define a hasher automatically called `MyNewStructHasher`, and pick a salt
//! equal to the full module path + "::"  + structure name, i.e. if
//! `MyNewStruct` is defined in `bar::baz::quux`, the salt will be `b"bar::baz::quux::MyNewStruct"`.
//! You can then use it in your implementation of `CryptoHash` (see below).
//!
//! ## The semi-automatic way
//!
//! For any new structure `MyNewStruct` that needs to be hashed, the developer should define a
//! new hasher with:
//!
//! ```
//! # // To get around that there's no way to doc-test a non-exported macro:
//! # macro_rules! define_hasher { ($e:expr) => () }
//! define_hasher! { (MyNewStructHasher, MY_NEW_STRUCT_HASHER, b"MyNewStruct") }
//! ```
//!
//! **Note**: The last argument for the `define_hasher` macro must be a unique string.
//!
//! ## The `CryptoHash` implementation (for both automatic and semi-automatic way)
//! Then, the `CryptoHash` trait should be implemented:
//! ```
//! # use libra_crypto::hash::*;
//! # #[derive(Default)]
//! # struct MyNewStructHasher;
//! # impl CryptoHasher for MyNewStructHasher {
//! #   fn finish(self) -> HashValue { unimplemented!() }
//! #   fn write(&mut self, bytes: &[u8]) -> &mut Self { unimplemented!() }
//! # }
//! struct MyNewStruct;
//!
//! impl CryptoHash for MyNewStruct {
//!     type Hasher = MyNewStructHasher; // use the above defined hasher here
//!
//!     fn hash(&self) -> HashValue {
//!         let mut state = Self::Hasher::default();
//!         state.write(b"Struct serialized into bytes here");
//!         state.finish()
//!     }
//! }
//! ```

use anyhow::{ensure, Result};
use bytes::Bytes;
use libra_nibble::Nibble;
use mirai_annotations::*;
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::EntropyRng, Rng};
use serde::{de, ser};
use std::{self, convert::AsRef, fmt};
use tiny_keccak::{Hasher, Sha3};

const LIBRA_HASH_SUFFIX: &[u8] = b"@@$$LIBRA$$@@";

#[cfg(test)]
#[path = "unit_tests/hash_test.rs"]
mod hash_test;

const SHORT_STRING_LENGTH: usize = 4;

/// Output value of our hash function. Intentionally opaque for safety and modularity.
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    /// The length of the hash in bytes.
    pub const LENGTH: usize = 32;
    /// The length of the hash in bits.
    pub const LENGTH_IN_BITS: usize = Self::LENGTH * 8;
    /// The length of the hash in nibbles.
    pub const LENGTH_IN_NIBBLES: usize = Self::LENGTH * 2;

    /// Create a new [`HashValue`] from a byte array.
    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue { hash }
    }

    /// Create from a slice (e.g. retrieved from storage).
    pub fn from_slice(src: &[u8]) -> Result<Self> {
        ensure!(
            src.len() == HashValue::LENGTH,
            "HashValue decoding failed due to length mismatch. HashValue \
             length: {}, src length: {}",
            HashValue::LENGTH,
            src.len()
        );
        let mut value = Self::zero();
        value.hash.copy_from_slice(src);
        Ok(value)
    }

    /// Dumps into a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    /// Creates a zero-initialized instance.
    pub fn zero() -> Self {
        HashValue {
            hash: [0; HashValue::LENGTH],
        }
    }

    /// Check if the hash value is zero.
    pub fn is_zero(&self) -> bool {
        *self == HashValue::zero()
    }

    /// Create a cryptographically random instance.
    pub fn random() -> Self {
        let mut rng = EntropyRng::new();
        let hash: [u8; HashValue::LENGTH] = rng.gen();
        HashValue { hash }
    }

    /// Creates a random instance with given rng. Useful in unit tests.
    pub fn random_with_rng<R: Rng>(rng: &mut R) -> Self {
        let hash: [u8; HashValue::LENGTH] = rng.gen();
        HashValue { hash }
    }

    /// Convenience function to compute a sha3-256 HashValue of the buffer. It will handle hasher
    /// creation, data feeding and finalization.
    pub fn from_sha3_256(buffer: &[u8]) -> Self {
        let mut sha3 = Sha3::v256();
        sha3.update(buffer);
        HashValue::from_keccak(sha3)
    }

    #[cfg(test)]
    pub fn from_iter_sha3<'a, I>(buffers: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let mut sha3 = Sha3::v256();
        for buffer in buffers {
            sha3.update(buffer);
        }
        HashValue::from_keccak(sha3)
    }

    fn as_ref_mut(&mut self) -> &mut [u8] {
        &mut self.hash[..]
    }

    fn from_keccak(state: Sha3) -> Self {
        let mut hash = Self::zero();
        state.finalize(hash.as_ref_mut());
        hash
    }

    /// Returns a `HashValueBitIterator` over all the bits that represent this `HashValue`.
    pub fn iter_bits(&self) -> HashValueBitIterator<'_> {
        HashValueBitIterator::new(self)
    }

    /// Constructs a `HashValue` from an iterator of bits.
    pub fn from_bit_iter(iter: impl ExactSizeIterator<Item = bool>) -> Result<Self> {
        ensure!(
            iter.len() == Self::LENGTH_IN_BITS,
            "The iterator should yield exactly {} bits. Actual number of bits: {}.",
            Self::LENGTH_IN_BITS,
            iter.len(),
        );

        let mut buf = [0; Self::LENGTH];
        for (i, bit) in iter.enumerate() {
            if bit {
                buf[i / 8] |= 1 << (7 - i % 8);
            }
        }
        Ok(Self::new(buf))
    }

    /// Returns the length of common prefix of `self` and `other` in bits.
    pub fn common_prefix_bits_len(&self, other: HashValue) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
    }

    /// Returns the length of common prefix of `self` and `other` in nibbles.
    pub fn common_prefix_nibbles_len(&self, other: HashValue) -> usize {
        self.common_prefix_bits_len(other) / 4
    }

    /// Returns the `index`-th nibble.
    pub fn get_nibble(&self, index: usize) -> Nibble {
        Nibble::from(if index % 2 == 0 {
            self[index / 2] >> 4
        } else {
            self[index / 2] & 0x0F
        })
    }

    /// Returns first SHORT_STRING_LENGTH bytes as String in hex
    pub fn short_str(&self) -> String {
        hex::encode(&self.hash[0..SHORT_STRING_LENGTH])
    }

    /// Full hex representation of a given hash value.
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }

    /// Parse a given hex string to a hash value.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        Self::from_slice(hex::decode(hex_str)?.as_slice())
    }
}

// TODO(#1307)
impl ser::Serialize for HashValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            serializer.serialize_newtype_struct("HashValue", &self.hash[..])
        }
    }
}

impl<'de> de::Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(::serde::Deserialize)]
        #[serde(rename = "HashValue")]
        struct _Value<'a>(&'a [u8]);

        if deserializer.is_human_readable() {
            let encoded_hash: &str = ::serde::Deserialize::deserialize(deserializer)?;
            HashValue::from_hex(encoded_hash).map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            let value = <_Value>::deserialize(deserializer)?;
            Self::from_slice(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
        }
    }
}

impl Default for HashValue {
    fn default() -> Self {
        HashValue::zero()
    }
}

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.hash
    }
}

impl std::ops::Index<usize> for HashValue {
    type Output = u8;

    fn index(&self, s: usize) -> &u8 {
        self.hash.index(s)
    }
}

impl fmt::Binary for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.hash {
            write!(f, "{:08b}", byte)?;
        }
        Ok(())
    }
}

impl fmt::LowerHex for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashValue(")?;
        <Self as fmt::LowerHex>::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

/// Will print shortened (4 bytes) hash
impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.hash.iter().take(4) {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<HashValue> for Bytes {
    fn from(value: HashValue) -> Bytes {
        Bytes::copy_from_slice(value.hash.as_ref())
    }
}

/// An iterator over `HashValue` that generates one bit for each iteration.
pub struct HashValueBitIterator<'a> {
    /// The reference to the bytes that represent the `HashValue`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
    // invariant hash_bytes.len() == HashValue::LENGTH;
    // invariant pos.end == hash_bytes.len() * 8;
}

impl<'a> HashValueBitIterator<'a> {
    /// Constructs a new `HashValueBitIterator` using given `HashValue`.
    fn new(hash_value: &'a HashValue) -> Self {
        HashValueBitIterator {
            hash_bytes: hash_value.as_ref(),
            pos: (0..HashValue::LENGTH_IN_BITS),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        assume!(index < self.pos.end); // assumed precondition
        assume!(self.hash_bytes.len() == HashValue::LENGTH); // invariant
        assume!(self.pos.end == self.hash_bytes.len() * 8); // invariant
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for HashValueBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> std::iter::DoubleEndedIterator for HashValueBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

impl<'a> std::iter::ExactSizeIterator for HashValueBitIterator<'a> {}

/// A type that implements `CryptoHash` can be hashed by a cryptographic hash function and produce
/// a `HashValue`. Each type needs to have its own `Hasher` type.
pub trait CryptoHash {
    /// The associated `Hasher` type which comes with a unique salt for this type.
    type Hasher: CryptoHasher;

    /// Hashes the object and produces a `HashValue`.
    fn hash(&self) -> HashValue;
}

/// A trait for generating hash from arbitrary stream of bytes.
///
/// Instances of `CryptoHasher` usually represent state that is changed while hashing data.
/// Similar to `std::hash::Hasher` but not same. CryptoHasher cannot be reused after finish() has
/// been called.
pub trait CryptoHasher: Default {
    /// Finish constructing the [`HashValue`].
    fn finish(self) -> HashValue;
    /// Write bytes into the hasher.
    fn write(&mut self, bytes: &[u8]) -> &mut Self;
    /// Write a single byte into the hasher.
    fn write_u8(&mut self, byte: u8) {
        self.write(&[byte]);
    }
}

/// Our preferred hashing schema, outputting [`HashValue`]s.
/// * Hashing is parameterized by a `domain` to prevent domain
/// ambiguity attacks.
/// * The existence of serialization/deserialization function rules
/// out any formatting ambiguity.
/// * Assuming that the `domain` seed is used only once per Rust type,
/// or that the serialization carries enough type information to avoid
/// ambiguities within a same domain.
/// * Only used internally within this crate
#[derive(Clone)]
pub struct DefaultHasher {
    state: Sha3,
}

impl CryptoHasher for DefaultHasher {
    fn finish(self) -> HashValue {
        let mut hasher = HashValue::default();
        self.state.finalize(hasher.as_ref_mut());
        hasher
    }

    fn write(&mut self, bytes: &[u8]) -> &mut Self {
        self.state.update(bytes);
        self
    }
}

impl Default for DefaultHasher {
    fn default() -> Self {
        DefaultHasher {
            state: Sha3::v256(),
        }
    }
}

impl DefaultHasher {
    /// initialize a new hasher with a specific salt
    pub fn new_with_salt(typename: &[u8]) -> Self {
        let mut state = Sha3::v256();
        if !typename.is_empty() {
            let mut salt = typename.to_vec();
            salt.extend_from_slice(LIBRA_HASH_SUFFIX);
            state.update(HashValue::from_sha3_256(&salt[..]).as_ref());
        }
        DefaultHasher { state }
    }
}

macro_rules! define_hasher {
    (
        $(#[$attr:meta])*
        ($hasher_type: ident, $hasher_name: ident, $salt: expr)
    ) => {

        #[derive(Clone)]
        $(#[$attr])*
        pub struct $hasher_type(DefaultHasher);

        impl $hasher_type {
            fn new() -> Self {
                $hasher_type(DefaultHasher::new_with_salt($salt))
            }
        }

        impl Default for $hasher_type {
            fn default() -> Self {
                $hasher_name.clone()
            }
        }

        impl CryptoHasher for $hasher_type {
            fn finish(self) -> HashValue {
                self.0.finish()
            }

            fn write(&mut self, bytes: &[u8]) -> &mut Self {
                self.0.write(bytes);
                self
            }
        }

        static $hasher_name: Lazy<$hasher_type> = Lazy::new(|| { $hasher_type::new() });
    };
}

define_hasher! {
    /// The hasher used to compute the hash of an internal node in the transaction accumulator.
    (
        TransactionAccumulatorHasher,
        TRANSACTION_ACCUMULATOR_HASHER,
        b"TransactionAccumulator"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of an internal node in the event accumulator.
    (
        EventAccumulatorHasher,
        EVENT_ACCUMULATOR_HASHER,
        b"EventAccumulator"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of an internal node in the Sparse Merkle Tree.
    (
        SparseMerkleInternalHasher,
        SPARSE_MERKLE_INTERNAL_HASHER,
        b"SparseMerkleInternal"
    )
}

define_hasher! {
    /// The hasher used only for testing. It doesn't have a salt.
    (TestOnlyHasher, TEST_ONLY_HASHER, b"")
}

define_hasher! {
    /// The hasher used to compute the hash of a DiscoveryMsg object.
    (DiscoveryMsgHasher, DISCOVERY_MSG_HASHER, b"DiscoveryMsg")
}

fn create_literal_hash(word: &str) -> HashValue {
    let mut s = word.as_bytes().to_vec();
    assert!(s.len() <= HashValue::LENGTH);
    s.resize(HashValue::LENGTH, 0);
    HashValue::from_slice(&s).expect("Cannot fail")
}

/// Placeholder hash of `Accumulator`.
pub static ACCUMULATOR_PLACEHOLDER_HASH: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("ACCUMULATOR_PLACEHOLDER_HASH"));

/// Placeholder hash of `SparseMerkleTree`.
pub static SPARSE_MERKLE_PLACEHOLDER_HASH: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("SPARSE_MERKLE_PLACEHOLDER_HASH"));

/// Block id reserved as the id of parent block of the genesis block.
pub static PRE_GENESIS_BLOCK_ID: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("PRE_GENESIS_BLOCK_ID"));

/// Genesis block id is used as a parent of the very first block executed by the executor.
pub static GENESIS_BLOCK_ID: Lazy<HashValue> = Lazy::new(|| {
    // This maintains the invariant that block.id() == block.hash(), for
    // the genesis block and allows us to (de/)serialize it consistently
    HashValue::new([
        0x5e, 0x10, 0xba, 0xd4, 0x5b, 0x35, 0xed, 0x92, 0x9c, 0xd6, 0xd2, 0xc7, 0x09, 0x8b, 0x13,
        0x5d, 0x02, 0xdd, 0x25, 0x9a, 0xe8, 0x8a, 0x8d, 0x09, 0xf4, 0xeb, 0x5f, 0xba, 0xe9, 0xa6,
        0xf6, 0xe4,
    ])
});

/// Provides a test_only_hash() method that can be used in tests on types that implement
/// `serde::Serialize`.
///
/// # Example
/// ```
/// use libra_crypto::hash::TestOnlyHash;
///
/// b"hello world".test_only_hash();
/// ```
pub trait TestOnlyHash {
    /// Generates a hash used only for tests.
    fn test_only_hash(&self) -> HashValue;
}

impl<T: ser::Serialize + ?Sized> TestOnlyHash for T {
    fn test_only_hash(&self) -> HashValue {
        let bytes = lcs::to_bytes(self).expect("serialize failed during hash.");
        let mut hasher = TestOnlyHasher::default();
        hasher.write(&bytes);
        hasher.finish()
    }
}
