// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(clippy::large_enum_variant)]

//! Protobuf definitions for data structures sent over the network
mod consensus {
    include!(concat!(env!("OUT_DIR"), "/consensus.rs"));
}

pub use self::consensus::ConsensusMsg;
