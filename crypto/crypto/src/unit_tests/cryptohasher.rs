// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate as libra_crypto;
use crate::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
use tiny_keccak::{Hasher, Sha3};

/// This file tests the CryptoHasher derive macro defined in the crypto-derive
/// crate. This macro, as the comments there indicate, is meant to derive an
/// instance of CryptoHasher (see `crate::hash`) for any type, which
/// domain-separation tag is precisely the fully-qualified name of the
/// struct. Here, we create a struct, manually "compute" its tag and compare it
/// to the generated one.

#[derive(CryptoHasher)]
pub struct Foo {}

impl CryptoHash for Foo {
    type Hasher = FooHasher;

    fn hash(&self) -> HashValue {
        // we voluntarily keep the contents empty, since we just want to test
        // the hasher prefix
        let state = Self::Hasher::default();
        state.finish()
    }
}

// workaround of crate::hash::LIBRA_HASH_SUFFIX being private
// the const below should always be a copy of it
const LIBRA_HASH_SUFFIX_FOR_TESTING: &[u8] = b"@@$$LIBRA$$@@";

#[test]
fn test_cryptohasher_name() {
    let mut name = "libra_crypto::unit_tests::cryptohasher::Foo"
        .as_bytes()
        .to_vec();
    name.extend_from_slice(LIBRA_HASH_SUFFIX_FOR_TESTING);

    let mut digest = Sha3::v256();
    digest.update(HashValue::from_sha3_256(&name[..]).as_ref());

    let expected = {
        let mut hasher_bytes = [0u8; 32];
        digest.finalize(&mut hasher_bytes);
        hasher_bytes
    };
    let foo_instance = Foo {};
    let foo_hash = CryptoHash::hash(&foo_instance);
    let actual = foo_hash.as_ref();
    assert_eq!(
        &expected,
        actual,
        "\nexpected: {} actual: {}",
        String::from_utf8_lossy(&expected),
        String::from_utf8_lossy(actual)
    );
}
