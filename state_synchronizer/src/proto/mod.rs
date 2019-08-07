// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(missing_docs)]
use types::proto::{ledger_info, transaction};

pub mod state_synchronizer;
pub mod state_synchronizer_client;
pub mod state_synchronizer_grpc;
