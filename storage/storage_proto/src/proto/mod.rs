// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::types::proto::{account_state_blob, get_with_proof, ledger_info, proof, transaction, types};

pub mod storage_prost {
    include!(concat!(env!("OUT_DIR"), "/storage.rs"));
}

pub mod storage;
pub mod storage_grpc;
