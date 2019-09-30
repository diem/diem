// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::shared::mempool_status::MempoolAddTransactionStatusCode;
use failure::prelude::*;
use libra_proto_conv::{FromProto, IntoProto};
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

/// Status of transaction insertion operation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MempoolAddTransactionStatus {
    /// Status code of the transaction insertion operation
    pub code: MempoolAddTransactionStatusCode,
    /// Message to give more details about the transaction insertion operation
    pub message: String,
}

impl MempoolAddTransactionStatus {
    /// Create a new MempoolAddTransactionStatus
    pub fn new(code: MempoolAddTransactionStatusCode, message: String) -> Self {
        Self { code, message }
    }
}

//***********************************
// Decoding/Encoding to Protobuffers
//***********************************
impl IntoProto for MempoolAddTransactionStatus {
    type ProtoType = crate::proto::shared::mempool_status::MempoolAddTransactionStatus;

    fn into_proto(self) -> Self::ProtoType {
        let mut mempool_add_transaction_status = Self::ProtoType::new();
        mempool_add_transaction_status.set_message(self.message);
        mempool_add_transaction_status.set_code(self.code);
        mempool_add_transaction_status
    }
}

impl FromProto for MempoolAddTransactionStatus {
    type ProtoType = crate::proto::shared::mempool_status::MempoolAddTransactionStatus;

    fn from_proto(proto: Self::ProtoType) -> Result<Self> {
        Ok(MempoolAddTransactionStatus::new(
            proto.get_code(),
            proto.get_message().to_string(),
        ))
    }
}
