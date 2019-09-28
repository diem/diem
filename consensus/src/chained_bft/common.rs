// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use canonical_serialization::{
    CanonicalDeserialize, CanonicalSerialize, CanonicalSerializer, SimpleSerializer,
};
use crypto::{
    hash::{CryptoHasher, RoundHasher},
    HashValue,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use types::account_address::AccountAddress;

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically. It is used for the protocol safety and liveness (please see the detailed
/// protocol description).
pub type Round = u64;
/// Height refers to the chain depth of a consensus block in a tree with respect to parent links.
/// The genesis block starts at height 0.  The round of a block is always >= height.  Height is
/// only used for debugging and testing as it is not required for implementing LibraBFT.
pub type Height = u64;
/// Author refers to the author's account address
pub type Author = AccountAddress;

/// Trait alias for the Block Payload.
pub trait Payload:
    Clone
    + Send
    + Sync
    + CanonicalSerialize
    + CanonicalDeserialize
    + DeserializeOwned
    + Serialize
    + Default
    + Debug
    + PartialEq
    + Eq
    + 'static
{
}

impl<T> Payload for T where
    T: Clone
        + Send
        + Sync
        + CanonicalSerialize
        + CanonicalDeserialize
        + DeserializeOwned
        + Serialize
        + Default
        + Debug
        + PartialEq
        + Eq
        + 'static
{
}

pub fn round_hash(round: Round) -> HashValue {
    let mut state = RoundHasher::default();
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_u64(round).expect("Should serialize.");
    let digest = serializer.get_output();
    state.write(digest.as_ref());
    state.finish()
}
