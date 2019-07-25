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
//! use crypto::hash::{CryptoHasher, TestOnlyHasher};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hash_value = hasher.finish();
//! ```
//! The output is of type [`HashValue`], which can be used as an input for signing.
//!
//! # Implementing new hashers
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
//! Then, the `CryptoHash` trait should be implemented:
//! ```
//! # use crypto::hash::*;
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

#![allow(clippy::unit_arg)]

use bytes::Bytes;
use failure::prelude::*;
use lazy_static::lazy_static;
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use rand::{rngs::EntropyRng, Rng};
use serde::{Deserialize, Serialize};
use std::{self, convert::AsRef, fmt};
use tiny_keccak::Keccak;

const LIBRA_HASH_SUFFIX: &[u8] = b"@@$$LIBRA$$@@";

#[cfg(test)]
#[path = "unit_tests/hash_test.rs"]
mod hash_test;

/// Output value of our hash function. Intentionally opaque for safety and modularity.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Arbitrary)]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    /// The length of the hash in bytes.
    pub const LENGTH: usize = 32;
    /// The length of the hash in bits.
    pub const LENGTH_IN_BITS: usize = HashValue::LENGTH * 8;

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

    /// Get the size of the hash.
    pub fn len() -> usize {
        HashValue::LENGTH
    }

    /// Get the last n bytes as a String.
    pub fn last_n_bytes(&self, bytes: usize) -> String {
        let mut string = String::from("HashValue(..");
        for byte in &self.hash[(HashValue::LENGTH - bytes)..] {
            string.push_str(&format!("{:02x}", byte));
        }
        string.push_str(")");
        string
    }

    // Intentionally not public.
    fn from_sha3(buffer: &[u8]) -> Self {
        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(buffer);
        HashValue::from_keccak(sha3)
    }

    #[cfg(test)]
    pub fn from_iter_sha3<'a, I>(buffers: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let mut sha3 = Keccak::new_sha3_256();
        for buffer in buffers {
            sha3.update(buffer);
        }
        HashValue::from_keccak(sha3)
    }

    fn as_ref_mut(&mut self) -> &mut [u8] {
        &mut self.hash[..]
    }

    fn from_keccak(state: Keccak) -> Self {
        let mut hash = Self::zero();
        state.finalize(hash.as_ref_mut());
        hash
    }

    /// Returns a `HashValueBitIterator` over all the bits that represent this `HashValue`.
    pub fn iter_bits(&self) -> HashValueBitIterator<'_> {
        HashValueBitIterator::new(self)
    }

    /// Returns the length of common prefix of `self` and `other` in bits.
    pub fn common_prefix_bits_len(&self, other: HashValue) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
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

impl FromProto for HashValue {
    type ProtoType = Vec<u8>;

    fn from_proto(bytes: Self::ProtoType) -> Result<Self> {
        HashValue::from_slice(&bytes)
    }
}

impl IntoProto for HashValue {
    type ProtoType = Vec<u8>;

    fn into_proto(self) -> Self::ProtoType {
        self.to_vec()
    }
}

impl From<HashValue> for Bytes {
    fn from(value: HashValue) -> Bytes {
        value.hash.as_ref().into()
    }
}

/// An iterator over `HashValue` that generates one bit for each iteration.
pub struct HashValueBitIterator<'a> {
    /// The reference to the bytes that represent the `HashValue`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
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
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for HashValueBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().and_then(|x| Some(self.get_bit(x)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> std::iter::DoubleEndedIterator for HashValueBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().and_then(|x| Some(self.get_bit(x)))
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
struct DefaultHasher {
    state: Keccak,
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
            state: Keccak::new_sha3_256(),
        }
    }
}

impl DefaultHasher {
    fn new_with_salt(typename: &[u8]) -> Self {
        let mut state = Keccak::new_sha3_256();
        if !typename.is_empty() {
            let mut salt = typename.to_vec();
            salt.extend_from_slice(LIBRA_HASH_SUFFIX);
            state.update(HashValue::from_sha3(&salt[..]).as_ref());
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

        lazy_static! {
            static ref $hasher_name: $hasher_type = { $hasher_type::new() };
        }
    };
}

define_hasher! {
    /// The hasher used to compute the hash of an AccessPath object.
    (AccessPathHasher, ACCESS_PATH_HASHER, b"VM_ACCESS_PATH")
}

define_hasher! {
    /// The hasher used to compute the hash of an AccountAddress object.
    (
        AccountAddressHasher,
        ACCOUNT_ADDRESS_HASHER,
        b"AccountAddress"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of a LedgerInfo object.
    (LedgerInfoHasher, LEDGER_INFO_HASHER, b"LedgerInfo")
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
    /// The hasher used to compute the hash of a leaf node in the Sparse Merkle Tree.
    (
        SparseMerkleLeafHasher,
        SPARSE_MERKLE_LEAF_HASHER,
        b"SparseMerkleLeaf"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of the blob content of an account.
    (
        AccountStateBlobHasher,
        ACCOUNT_STATE_BLOB_HASHER,
        b"AccountStateBlob"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of a TransactionInfo object.
    (
        TransactionInfoHasher,
        TRANSACTION_INFO_HASHER,
        b"TransactionInfo"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of a RawTransaction object.
    (
        RawTransactionHasher,
        RAW_TRANSACTION_HASHER,
        b"RawTransaction"
    )
}

define_hasher! {
    /// The hasher used to compute the hash of a SignedTransaction object.
    (
        SignedTransactionHasher,
        SIGNED_TRANSACTION_HASHER,
        b"SignedTransaction"
    )
}

define_hasher! {
    /// The hasher used to compute the hash (block_id) of a Block object.
    (BlockHasher, BLOCK_HASHER, b"BlockId")
}

define_hasher! {
    /// The hasher used to compute the hash of a PacemakerTimeout object.
    (PacemakerTimeoutHasher, PACEMAKER_TIMEOUT_HASHER, b"PacemakerTimeout")
}

define_hasher! {
    /// The hasher used to compute the hash of a TimeoutMsgHasher object.
    (TimeoutMsgHasher, TIMEOUT_MSG_HASHER, b"TimeoutMsg")
}

define_hasher! {
    /// The hasher used to compute the hash of a VoteMsg object.
    (VoteMsgHasher, VOTE_MSG_HASHER, b"VoteMsg")
}

define_hasher! {
    /// The hasher used to compute the hash of a ContractEvent object.
    (ContractEventHasher, CONTRACT_EVENT_HASHER, b"ContractEvent")
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

lazy_static! {
    /// Placeholder hash of `Accumulator`.
    pub static ref ACCUMULATOR_PLACEHOLDER_HASH: HashValue =
        create_literal_hash("ACCUMULATOR_PLACEHOLDER_HASH");

    /// Placeholder hash of `SparseMerkleTree`.
    pub static ref SPARSE_MERKLE_PLACEHOLDER_HASH: HashValue =
        create_literal_hash("SPARSE_MERKLE_PLACEHOLDER_HASH");

    /// Block id reserved as the id of parent block of the genesis block.
    pub static ref PRE_GENESIS_BLOCK_ID: HashValue =
        create_literal_hash("PRE_GENESIS_BLOCK_ID");

    /// Genesis block id is used as a parent of the very first block executed by the executor.
    pub static ref GENESIS_BLOCK_ID: HashValue =
        create_literal_hash("GENESIS_BLOCK_ID");
}

/// Provides a test_only_hash() method that can be used in tests on types that implement
/// `serde::Serialize`.
///
/// # Example
/// ```
/// use crypto::hash::TestOnlyHash;
///
/// b"hello world".test_only_hash();
/// ```
pub trait TestOnlyHash {
    /// Generates a hash used only for tests.
    fn test_only_hash(&self) -> HashValue;
}

impl<T: Serialize + ?Sized> TestOnlyHash for T {
    fn test_only_hash(&self) -> HashValue {
        let bytes = ::bincode::serialize(self).expect("serialize failed during hash.");
        let mut hasher = TestOnlyHasher::default();
        hasher.write(&bytes);
        hasher.finish()
    }
}
