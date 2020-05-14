// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test file for the procedural macros CryptoHasher and LCSCryptoHash.

use crate as libra_crypto;
use crate::{
    hash::{CryptoHash, CryptoHasher, LIBRA_HASH_SUFFIX},
    HashValue,
};
use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

// The expected use case.
#[derive(Serialize, Deserialize, CryptoHasher, LCSCryptoHash)]
pub struct Foo {
    a: u64,
    b: u32,
}

// Used for testing the seed in FooHasher.
pub struct Bar {}

impl CryptoHash for Bar {
    type Hasher = FooHasher;

    fn hash(&self) -> HashValue {
        let state = Self::Hasher::default();
        state.finish()
    }
}

#[test]
fn test_cryptohasher_name() {
    let mut name = "libra_crypto::unit_tests::cryptohasher::Foo"
        .as_bytes()
        .to_vec();
    name.extend_from_slice(LIBRA_HASH_SUFFIX);

    let value = Bar {};
    let expected = {
        let mut digest = Sha3::v256();
        digest.update(HashValue::from_sha3_256(&name[..]).as_ref());
        let mut hasher_bytes = [0u8; 32];
        digest.finalize(&mut hasher_bytes);
        hasher_bytes
    };
    let actual = CryptoHash::hash(&value);
    assert_eq!(
        &expected,
        actual.as_ref(),
        "\nexpected: {} actual: {}",
        String::from_utf8_lossy(&expected),
        String::from_utf8_lossy(actual.as_ref())
    );
}

#[test]
fn test_lcs_cryptohash() {
    let mut name = "libra_crypto::unit_tests::cryptohasher::Foo"
        .as_bytes()
        .to_vec();
    name.extend_from_slice(LIBRA_HASH_SUFFIX);

    let value = Foo { a: 5, b: 1025 };
    let expected = {
        let mut digest = Sha3::v256();
        digest.update(HashValue::from_sha3_256(&name[..]).as_ref());
        digest.update(&lcs::to_bytes(&value).unwrap());
        let mut hasher_bytes = [0u8; 32];
        digest.finalize(&mut hasher_bytes);
        hasher_bytes
    };
    let actual = CryptoHash::hash(&value);
    assert_eq!(
        &expected,
        actual.as_ref(),
        "\nexpected: {} actual: {}",
        String::from_utf8_lossy(&expected),
        String::from_utf8_lossy(actual.as_ref())
    );
}
