// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

//pub mod mempool_status;

//pub use self::super::MempoolAddTransactionStatus;

pub mod mempool_status {
    include!(concat!(env!("OUT_DIR"), "/mempool_status.rs"));
}
