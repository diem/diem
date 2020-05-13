// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Error, Result};
use libra_crypto::HashValue;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    transaction::{
        SignatureCheckedTransaction, SignedTransaction, Transaction, TransactionInfo,
        TransactionListWithProof, TransactionToCommit, TransactionWithProof,
    },
    vm_error::StatusCode,
};

impl TryFrom<crate::proto::types::SignedTransaction> for SignedTransaction {
    type Error = Error;

    fn try_from(txn: crate::proto::types::SignedTransaction) -> Result<Self> {
        lcs::from_bytes(&txn.txn_bytes).map_err(Into::into)
    }
}

impl From<SignedTransaction> for crate::proto::types::SignedTransaction {
    fn from(txn: SignedTransaction) -> Self {
        let txn_bytes = lcs::to_bytes(&txn).expect("Unable to serialize SignedTransaction");
        Self { txn_bytes }
    }
}

impl From<SignatureCheckedTransaction> for crate::proto::types::SignedTransaction {
    fn from(txn: SignatureCheckedTransaction) -> Self {
        txn.into_inner().into()
    }
}

impl TryFrom<crate::proto::types::TransactionWithProof> for TransactionWithProof {
    type Error = Error;

    fn try_from(mut proto: crate::proto::types::TransactionWithProof) -> Result<Self> {
        let version = proto.version;
        let transaction = proto
            .transaction
            .ok_or_else(|| format_err!("Missing transaction"))?
            .try_into()?;
        let proof = proto
            .proof
            .ok_or_else(|| format_err!("Missing proof"))?
            .try_into()?;
        let events = proto
            .events
            .take()
            .map(|list| {
                list.events
                    .into_iter()
                    .map(ContractEvent::try_from)
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        Ok(Self {
            version,
            transaction,
            proof,
            events,
        })
    }
}

impl From<TransactionWithProof> for crate::proto::types::TransactionWithProof {
    fn from(mut txn: TransactionWithProof) -> Self {
        Self {
            version: txn.version,
            transaction: Some(txn.transaction.into()),
            proof: Some(txn.proof.into()),
            events: txn
                .events
                .take()
                .map(|list| crate::proto::types::EventsList {
                    events: list.into_iter().map(ContractEvent::into).collect(),
                }),
        }
    }
}

// Should be placed back in transaction as soon as feasible.
impl TryFrom<crate::proto::types::TransactionInfo> for TransactionInfo {
    type Error = Error;

    fn try_from(proto_txn_info: crate::proto::types::TransactionInfo) -> Result<Self> {
        let transaction_hash = HashValue::from_slice(&proto_txn_info.transaction_hash)?;
        let state_root_hash = HashValue::from_slice(&proto_txn_info.state_root_hash)?;
        let event_root_hash = HashValue::from_slice(&proto_txn_info.event_root_hash)?;
        let gas_used = proto_txn_info.gas_used;
        let major_status =
            StatusCode::try_from(proto_txn_info.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS);
        Ok(TransactionInfo::new(
            transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
            major_status,
        ))
    }
}

// Should be placed back in transaction as soon as feasible
impl From<&TransactionInfo> for crate::proto::types::TransactionInfo {
    fn from(txn_info: &TransactionInfo) -> Self {
        Self {
            transaction_hash: txn_info.transaction_hash().to_vec(),
            state_root_hash: txn_info.state_root_hash().to_vec(),
            event_root_hash: txn_info.event_root_hash().to_vec(),
            gas_used: txn_info.gas_used(),
            major_status: txn_info.major_status().into(),
        }
    }
}

// Explicit implementation for &TransactionInfo needed.  ??
impl From<TransactionInfo> for crate::proto::types::TransactionInfo {
    fn from(txn_info: TransactionInfo) -> Self {
        Self::from(&txn_info)
    }
}

impl TryFrom<crate::proto::types::TransactionToCommit> for TransactionToCommit {
    type Error = Error;

    fn try_from(proto: crate::proto::types::TransactionToCommit) -> Result<Self> {
        let transaction = proto
            .transaction
            .ok_or_else(|| format_err!("Missing signed_transaction"))?
            .try_into()?;
        let num_account_states = proto.account_states.len();
        let account_states = proto
            .account_states
            .into_iter()
            .map(|x| {
                Ok((
                    AccountAddress::try_from(x.address)?,
                    AccountStateBlob::from(x.blob),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        ensure!(
            account_states.len() == num_account_states,
            "account_states should have no duplication."
        );
        let events = proto
            .events
            .into_iter()
            .map(ContractEvent::try_from)
            .collect::<Result<Vec<_>>>()?;
        let gas_used = proto.gas_used;
        let major_status =
            StatusCode::try_from(proto.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS);

        Ok(TransactionToCommit::new(
            transaction,
            account_states,
            events,
            gas_used,
            major_status,
        ))
    }
}

impl From<TransactionToCommit> for crate::proto::types::TransactionToCommit {
    fn from(txn: TransactionToCommit) -> Self {
        Self {
            transaction: Some(txn.transaction().into()),
            account_states: txn
                .account_states()
                .iter()
                .map(|(address, blob)| crate::proto::types::AccountState {
                    address: address.as_ref().to_vec(),
                    blob: blob.into(),
                })
                .collect(),
            events: txn.events().iter().map(Into::into).collect(),
            gas_used: txn.gas_used(),
            major_status: txn.major_status().into(),
        }
    }
}

impl TryFrom<crate::proto::types::TransactionListWithProof> for TransactionListWithProof {
    type Error = Error;

    fn try_from(mut proto: crate::proto::types::TransactionListWithProof) -> Result<Self> {
        let transactions = proto
            .transactions
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let events = proto
            .events_for_versions
            .take() // Option<EventsForVersions>
            .map(|events_for_versions| {
                // EventsForVersion
                events_for_versions
                    .events_for_version
                    .into_iter()
                    .map(|events_for_version| {
                        events_for_version
                            .events
                            .into_iter()
                            .map(ContractEvent::try_from)
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        let first_transaction_version = proto.first_transaction_version;

        let proof = proto
            .proof
            .ok_or_else(|| format_err!("Missing proof."))?
            .try_into()?;

        Ok(Self::new(
            transactions,
            events,
            first_transaction_version,
            proof,
        ))
    }
}

impl From<TransactionListWithProof> for crate::proto::types::TransactionListWithProof {
    fn from(txn: TransactionListWithProof) -> Self {
        let transactions = txn.transactions.into_iter().map(Into::into).collect();

        let events_for_versions =
            txn.events
                .map(|all_events| crate::proto::types::EventsForVersions {
                    events_for_version: all_events
                        .into_iter()
                        .map(|events_for_version| crate::proto::types::EventsList {
                            events: events_for_version
                                .into_iter()
                                .map(ContractEvent::into)
                                .collect::<Vec<_>>(),
                        })
                        .collect::<Vec<_>>(),
                });

        Self {
            transactions,
            events_for_versions,
            first_transaction_version: txn.first_transaction_version,
            proof: Some(txn.proof.into()),
        }
    }
}

impl TryFrom<crate::proto::types::Transaction> for Transaction {
    type Error = Error;

    fn try_from(proto: crate::proto::types::Transaction) -> Result<Self> {
        lcs::from_bytes(&proto.transaction).map_err(Into::into)
    }
}

impl From<&Transaction> for crate::proto::types::Transaction {
    fn from(txn: &Transaction) -> Self {
        let bytes = lcs::to_bytes(&txn).expect("Serialization should not fail.");
        Self { transaction: bytes }
    }
}

impl From<Transaction> for crate::proto::types::Transaction {
    fn from(txn: Transaction) -> Self {
        Self::from(&txn)
    }
}
