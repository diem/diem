// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

#[allow(clippy::large_enum_variant)]
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}
