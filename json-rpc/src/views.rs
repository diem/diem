// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use hex;
use libra_types::account_config::AccountResource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountView {
    balance: u64,
    sequence_number: u64,
    authentication_key: BytesView,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
}

impl AccountView {
    pub fn new(account: &AccountResource) -> Self {
        Self {
            balance: account.balance(),
            sequence_number: account.sequence_number(),
            authentication_key: BytesView::from(account.authentication_key()),
            delegated_key_rotation_capability: account.delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account.delegated_withdrawal_capability(),
        }
    }

    // TODO remove test tag once used by CLI client
    #[cfg(test)]
    pub(crate) fn balance(&self) -> u64 {
        self.balance
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockMetadata {
    pub version: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BytesView(String);

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }
}
