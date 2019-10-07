// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{account_address::AccountAddress, transaction::SignedTransaction};
use std::time::Duration;

#[derive(Clone)]
pub struct MempoolTransaction {
    pub txn: SignedTransaction,
    // system expiration time of transaction. It should be removed from mempool by that time
    pub expiration_time: Duration,
    pub gas_amount: u64,
    pub timeline_state: TimelineState,
}

impl MempoolTransaction {
    pub(crate) fn new(
        txn: SignedTransaction,
        expiration_time: Duration,
        gas_amount: u64,
        timeline_state: TimelineState,
    ) -> Self {
        Self {
            txn,
            gas_amount,
            expiration_time,
            timeline_state,
        }
    }
    pub(crate) fn get_sequence_number(&self) -> u64 {
        self.txn.sequence_number()
    }
    pub(crate) fn get_sender(&self) -> AccountAddress {
        self.txn.sender()
    }
    pub(crate) fn get_gas_price(&self) -> u64 {
        self.txn.gas_unit_price()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TimelineState {
    // transaction is ready for broadcast
    // Associated integer represents it's position in log of such transactions
    Ready(u64),
    // transaction is not yet ready for broadcast
    // but it might change in a future
    NotReady,
    // transaction will never be qualified for broadcasting
    // currently we don't broadcast transactions originated on other peers
    NonQualified,
}
