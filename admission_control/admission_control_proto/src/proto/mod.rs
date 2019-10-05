// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::types::proto::*;
use mempool::proto::shared::mempool_status;

pub mod admission_control {
    include!(concat!(env!("OUT_DIR"), "/admission_control.rs"));
}
