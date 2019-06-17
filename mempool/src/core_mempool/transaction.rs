// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::shared::mempool_status::MempoolAddTransactionStatus as ProtoMempoolAddTransactionStatus;
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::time::Duration;
use types::{account_address::AccountAddress, transaction::SignedTransaction};

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
pub enum MempoolAddTransactionStatus {
    /// Transaction was successfully sent to Mempool
    Valid,
    /// The sender does not have enough balance for the transaction
    InsufficientBalance,
    /// Transaction sequence number is invalid(e.g. too old)
    InvalidSeqNumber,
    /// Mempool is full (reached max global capacity)
    MempoolIsFull,
    /// Account reached max capacity per account
    TooManyTransactions,
    /// Invalid update. Only gas price increase is allowed
    InvalidUpdate,
}

impl IntoProto for MempoolAddTransactionStatus {
    type ProtoType = crate::proto::shared::mempool_status::MempoolAddTransactionStatus;

    fn into_proto(self) -> Self::ProtoType {
        match self {
            MempoolAddTransactionStatus::Valid => ProtoMempoolAddTransactionStatus::Valid,
            MempoolAddTransactionStatus::InsufficientBalance => {
                ProtoMempoolAddTransactionStatus::InsufficientBalance
            }
            MempoolAddTransactionStatus::InvalidSeqNumber => {
                ProtoMempoolAddTransactionStatus::InvalidSeqNumber
            }
            MempoolAddTransactionStatus::InvalidUpdate => {
                ProtoMempoolAddTransactionStatus::InvalidUpdate
            }
            MempoolAddTransactionStatus::MempoolIsFull => {
                ProtoMempoolAddTransactionStatus::MempoolIsFull
            }
            MempoolAddTransactionStatus::TooManyTransactions => {
                ProtoMempoolAddTransactionStatus::TooManyTransactions
            }
        }
    }
}

impl FromProto for MempoolAddTransactionStatus {
    type ProtoType = crate::proto::shared::mempool_status::MempoolAddTransactionStatus;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        let ret = match object {
            ProtoMempoolAddTransactionStatus::Valid => MempoolAddTransactionStatus::Valid,
            ProtoMempoolAddTransactionStatus::InsufficientBalance => {
                MempoolAddTransactionStatus::InsufficientBalance
            }
            ProtoMempoolAddTransactionStatus::InvalidSeqNumber => {
                MempoolAddTransactionStatus::InvalidSeqNumber
            }
            ProtoMempoolAddTransactionStatus::InvalidUpdate => {
                MempoolAddTransactionStatus::InvalidUpdate
            }
            ProtoMempoolAddTransactionStatus::MempoolIsFull => {
                MempoolAddTransactionStatus::MempoolIsFull
            }
            ProtoMempoolAddTransactionStatus::TooManyTransactions => {
                MempoolAddTransactionStatus::TooManyTransactions
            }
        };
        Ok(ret)
    }
}
