// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{account_address::AccountAddress, transaction::SignedTransaction};

/// Consensus starts at epoch 0 and increments it periodically.
pub type Epoch = u64;

/// LibraBFT works in a series of "rounds". Rounds are reset to 0 when entering a new epoch.
pub type Round = u64;

/// Author refers to a validator's account address
pub type Author = AccountAddress;

/// The payload in a block is a list of signed transactions.
pub type Payload = Vec<SignedTransaction>;
