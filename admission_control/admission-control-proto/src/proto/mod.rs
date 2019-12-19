// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::libra_types::proto::*;
use libra_mempool_shared_proto::proto::mempool_status;

pub mod admission_control {
    include!(concat!(env!("OUT_DIR"), "/admission_control.rs"));
}

pub use self::admission_control::{
    AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse,
};
