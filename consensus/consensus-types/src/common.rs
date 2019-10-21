// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::account_address::AccountAddress;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically. It is used for the protocol safety and liveness (please see the detailed
/// protocol description).
pub type Round = u64;
/// Author refers to the author's account address
pub type Author = AccountAddress;

/// Trait alias for the Block Payload.
pub trait Payload:
    Clone + Send + Sync + DeserializeOwned + Serialize + Default + Debug + PartialEq + Eq + 'static
{
}

impl<T> Payload for T where
    T: Clone
        + Send
        + Sync
        + DeserializeOwned
        + Serialize
        + Default
        + Debug
        + PartialEq
        + Eq
        + 'static
{
}
