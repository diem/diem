// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains various utility functions for testing and debugging purposes

use crate::signing::{generate_keypair_for_testing, PrivateKey, PublicKey};
use bincode::{deserialize, serialize};
use proptest::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Serialize};

/// Used to produce keypairs from a seed for testing purposes
pub fn keypair_strategy() -> impl Strategy<Value = (PrivateKey, PublicKey)> {
    // The no_shrink is because keypairs should be fixed -- shrinking would cause a different
    // keypair to be generated, which appears to not be very useful.
    any::<[u8; 32]>()
        .prop_map(|seed| {
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            let (private_key, public_key) = generate_keypair_for_testing(&mut rng);
            (private_key, public_key)
        })
        .no_shrink()
}

/// Generically deserializes a string into a struct `T`, used for producing test cases
pub fn from_encoded_string<T>(encoded_str: String) -> T
where
    T: DeserializeOwned,
{
    assert!(!encoded_str.is_empty());
    let bytes_out = ::hex::decode(encoded_str).unwrap();
    deserialize::<T>(&bytes_out).unwrap()
}

/// Generically serializes a string from a struct `T`, used for producing test cases
pub fn encode_to_string<T>(to_encode: &T) -> String
where
    T: Serialize,
{
    let bytes = serialize(to_encode).unwrap();
    ::hex::encode(&bytes)
}
