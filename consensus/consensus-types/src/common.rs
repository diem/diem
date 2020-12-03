// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically. It is used for the protocol safety and liveness (please see the detailed
/// protocol description).
pub type Round = u64;
/// Author refers to the author's account address
pub type Author = AccountAddress;

/// The payload in block.
pub type Payload = Vec<SignedTransaction>;
