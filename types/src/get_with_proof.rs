// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::AccountResource,
    account_state_blob::AccountStateWithProof,
    contract_event::EventWithProof,
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::AccumulatorConsistencyProof,
    transaction::{TransactionListWithProof, TransactionWithProof, Version},
    trusted_state::{TrustedState, TrustedStateChange},
};
use anyhow::{bail, ensure, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{cmp, convert::TryFrom, mem};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct UpdateToLatestLedgerRequest {
    pub client_known_version: u64,
    pub requested_items: Vec<RequestItem>,
}

impl UpdateToLatestLedgerRequest {
    pub fn new(client_known_version: u64, requested_items: Vec<RequestItem>) -> Self {
        UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpdateToLatestLedgerResponse {
    pub response_items: Vec<ResponseItem>,
    pub ledger_info_with_sigs: LedgerInfoWithSignatures,
    pub epoch_change_proof: EpochChangeProof,
    pub ledger_consistency_proof: AccumulatorConsistencyProof,
}

impl UpdateToLatestLedgerResponse {
    /// Constructor.
    pub fn new(
        response_items: Vec<ResponseItem>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        epoch_change_proof: EpochChangeProof,
        ledger_consistency_proof: AccumulatorConsistencyProof,
    ) -> Self {
        UpdateToLatestLedgerResponse {
            response_items,
            ledger_info_with_sigs,
            epoch_change_proof,
            ledger_consistency_proof,
        }
    }

    /// Verifies that the response has items corresponding to request items and each of them are
    /// supported by proof it carries and is what the request item asks for.
    ///
    /// After calling this one can trust the info in the response items without further
    /// verification.
    pub fn verify<'a>(
        &'a self,
        trusted_state: &TrustedState,
        request: &UpdateToLatestLedgerRequest,
    ) -> Result<TrustedStateChange<'a>> {
        verify_update_to_latest_ledger_response(
            trusted_state,
            request.client_known_version,
            &request.requested_items,
            &self.response_items,
            &self.ledger_info_with_sigs,
            &self.epoch_change_proof,
        )
    }
}

/// Verifies content of an [`UpdateToLatestLedgerResponse`] against the proofs it
/// carries and the content of the corresponding [`UpdateToLatestLedgerRequest`]
/// Return EpochInfo if there're epoch change events.
pub fn verify_update_to_latest_ledger_response<'a>(
    trusted_state: &TrustedState,
    req_client_known_version: u64,
    req_request_items: &[RequestItem],
    response_items: &[ResponseItem],
    ledger_info_with_sigs: &'a LedgerInfoWithSignatures,
    epoch_change_proof: &'a EpochChangeProof,
) -> Result<TrustedStateChange<'a>> {
    let ledger_info = ledger_info_with_sigs.ledger_info();

    // Verify that the same or a newer ledger info is returned.
    ensure!(
        ledger_info.version() >= req_client_known_version,
        "Got stale ledger_info with version {}, known version: {}.",
        ledger_info.version(),
        req_client_known_version,
    );

    // Verify any epoch change proof and latest ledger info. Provide an
    // updated trusted state if the proof is correct and not stale.
    let trusted_state_change =
        trusted_state.verify_and_ratchet(ledger_info_with_sigs, epoch_change_proof)?;

    // Verify each sub response.
    ensure!(
        req_request_items.len() == response_items.len(),
        "Number of request items ({}) does not match that of response items ({}).",
        req_request_items.len(),
        response_items.len(),
    );
    itertools::zip_eq(req_request_items, response_items)
        .map(|(req, res)| verify_response_item(ledger_info, req, res))
        .collect::<Result<Vec<_>>>()?;

    Ok(trusted_state_change)
}

fn verify_response_item(
    ledger_info: &LedgerInfo,
    req: &RequestItem,
    res: &ResponseItem,
) -> Result<()> {
    match (req, res) {
        // GetAccountState
        (
            RequestItem::GetAccountState { address },
            ResponseItem::GetAccountState {
                account_state_with_proof,
            },
        ) => account_state_with_proof.verify(ledger_info, ledger_info.version(), *address),
        // GetAccountTransactionBySequenceNumber
        (
            RequestItem::GetAccountTransactionBySequenceNumber {
                account,
                sequence_number,
                fetch_events,
            },
            ResponseItem::GetAccountTransactionBySequenceNumber {
                transaction_with_proof,
                proof_of_current_sequence_number,
            },
        ) => verify_get_txn_by_seq_num_resp(
            ledger_info,
            *account,
            *sequence_number,
            *fetch_events,
            transaction_with_proof.as_ref(),
            proof_of_current_sequence_number.as_ref(),
        ),
        // GetEventsByEventAccessPath
        (
            RequestItem::GetEventsByEventAccessPath {
                access_path,
                start_event_seq_num,
                ascending,
                limit,
            },
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            },
        ) => verify_get_events_by_access_path_resp(
            ledger_info,
            access_path,
            *start_event_seq_num,
            *ascending,
            *limit,
            events_with_proof,
            proof_of_latest_event,
        ),
        // GetTransactions
        (
            RequestItem::GetTransactions {
                start_version,
                limit,
                fetch_events,
            },
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            },
        ) => verify_get_txns_resp(
            ledger_info,
            *start_version,
            *limit,
            *fetch_events,
            txn_list_with_proof,
        ),
        // Request-response item types mismatch.
        _ => bail!(
            "RequestItem/ResponseItem types mismatch. request: {:?}, response: {:?}",
            mem::discriminant(req),
            mem::discriminant(res),
        ),
    }
}

fn verify_get_txn_by_seq_num_resp(
    ledger_info: &LedgerInfo,
    req_account: AccountAddress,
    req_sequence_number: u64,
    req_fetch_events: bool,
    transaction_with_proof: Option<&TransactionWithProof>,
    proof_of_current_sequence_number: Option<&AccountStateWithProof>,
) -> Result<()> {
    match (transaction_with_proof, proof_of_current_sequence_number) {
        (Some(transaction_with_proof), None) => {
            ensure!(
                req_fetch_events == transaction_with_proof.events.is_some(),
                "Bad GetAccountTxnBySeqNum response. Events requested: {}, events returned: {}.",
                req_fetch_events,
                transaction_with_proof.events.is_some(),
            );
            transaction_with_proof.verify_user_txn(
                ledger_info,
                transaction_with_proof.version,
                req_account,
                req_sequence_number,
            )
        },
        (None, Some(proof_of_current_sequence_number)) => {
            let sequence_number_in_ledger = {
                if let Some(blob) = &proof_of_current_sequence_number.blob {
                    AccountResource::try_from(blob)?.sequence_number()
                } else {
                    // Account does not exist. From the sequence number perspective, it's
                    // equivalent to the situation when the account does exist but has never sent
                    // a transaction. Use default value of sequence number.
                    0
                }
            };
            ensure!(
                sequence_number_in_ledger <= req_sequence_number,
                "Server returned no transactions while it should. Seq num requested: {}, latest seq num in ledger: {}.",
                req_sequence_number,
                sequence_number_in_ledger
            );
            proof_of_current_sequence_number.verify(ledger_info, ledger_info.version(), req_account)
        },
        _ => bail!(
            "Bad GetAccountTxnBySeqNum response. txn_proof.is_none():{}, cur_seq_num_proof.is_none():{}",
            transaction_with_proof.is_none(),
            proof_of_current_sequence_number.is_none(),
        )
    }
}

fn verify_get_events_by_access_path_resp(
    ledger_info: &LedgerInfo,
    req_access_path: &AccessPath,
    req_start_seq_num: u64,
    req_ascending: bool,
    req_limit: u64,
    events_with_proof: &[EventWithProof],
    proof_of_latest_event: &AccountStateWithProof,
) -> Result<()> {
    proof_of_latest_event.verify(ledger_info, ledger_info.version(), req_access_path.address)?;

    let (expected_event_key_opt, seq_num_upper_bound) =
        proof_of_latest_event.get_event_key_and_count_by_query_path(&req_access_path.path)?;

    let cursor =
        if !req_ascending && req_start_seq_num == u64::max_value() && seq_num_upper_bound > 0 {
            seq_num_upper_bound - 1
        } else {
            req_start_seq_num
        };

    let expected_seq_nums = if cursor >= seq_num_upper_bound {
        // Unreachable, so empty.
        Vec::new()
    } else if req_ascending {
        // Ascending, from start to upper bound or limit.
        (cursor..cmp::min(cursor + req_limit, seq_num_upper_bound)).collect()
    } else if cursor + 1 < req_limit {
        // Descending and hitting 0.
        (0..=cursor).rev().collect()
    } else {
        // Descending and hitting limit.
        (cursor + 1 - req_limit..=cursor).rev().collect()
    };

    ensure!(
        expected_seq_nums.len() == events_with_proof.len(),
        "Expecting {} events, got {}.",
        expected_seq_nums.len(),
        events_with_proof.len(),
    );
    if let Some(expected_event_key) = expected_event_key_opt {
        itertools::zip_eq(events_with_proof, expected_seq_nums)
            .map(|(e, seq_num)| {
                e.verify(
                    ledger_info,
                    &expected_event_key,
                    seq_num,
                    e.transaction_version,
                    e.event_index,
                )
            })
            .collect::<Result<Vec<_>>>()?;
    } else if !events_with_proof.is_empty() {
        bail!("Bad events_with_proof: nonempty event list for nonexistent account")
    } // else, empty event list for nonexistent account, which is fine

    Ok(())
}

fn verify_get_txns_resp(
    ledger_info: &LedgerInfo,
    req_start_version: Version,
    req_limit: u64,
    req_fetch_events: bool,
    txn_list_with_proof: &TransactionListWithProof,
) -> Result<()> {
    if req_limit == 0 || req_start_version > ledger_info.version() {
        txn_list_with_proof.verify(ledger_info, None)
    } else {
        ensure!(
            req_fetch_events == txn_list_with_proof.events.is_some(),
            "Bad GetTransactions response. Events requested: {}, events returned: {}.",
            req_fetch_events,
            txn_list_with_proof.events.is_some(),
        );
        let num_txns = txn_list_with_proof.transactions.len();
        ensure!(
            cmp::min(req_limit, ledger_info.version() - req_start_version + 1) == num_txns as u64,
            "Number of transactions returned not expected. num_txns: {}, start version: {}, \
             latest version: {}",
            num_txns,
            req_start_version,
            ledger_info.version(),
        );
        txn_list_with_proof.verify(ledger_info, Some(req_start_version))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum RequestItem {
    GetAccountTransactionBySequenceNumber {
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    },
    // this can't be the first variant, tracked here https://github.com/AltSysrq/proptest/issues/141
    GetAccountState {
        address: AccountAddress,
    },
    GetEventsByEventAccessPath {
        access_path: AccessPath,
        start_event_seq_num: u64,
        ascending: bool,
        limit: u64,
    },
    GetTransactions {
        start_version: Version,
        limit: u64,
        fetch_events: bool,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum ResponseItem {
    GetAccountTransactionBySequenceNumber {
        transaction_with_proof: Option<TransactionWithProof>,
        proof_of_current_sequence_number: Option<AccountStateWithProof>,
    },
    // this can't be the first variant, tracked here https://github.com/AltSysrq/proptest/issues/141
    GetAccountState {
        account_state_with_proof: AccountStateWithProof,
    },
    GetEventsByEventAccessPath {
        events_with_proof: Vec<EventWithProof>,
        // TODO: Rename this field to proof_of_event_handle.
        proof_of_latest_event: AccountStateWithProof,
    },
    GetTransactions {
        txn_list_with_proof: TransactionListWithProof,
    },
}

impl ResponseItem {
    pub fn into_get_account_state_response(self) -> Result<AccountStateWithProof> {
        match self {
            ResponseItem::GetAccountState {
                account_state_with_proof,
            } => Ok(account_state_with_proof),
            _ => bail!("Not ResponseItem::GetAccountState."),
        }
    }

    pub fn into_get_account_txn_by_seq_num_response(
        self,
    ) -> Result<(Option<TransactionWithProof>, Option<AccountStateWithProof>)> {
        match self {
            ResponseItem::GetAccountTransactionBySequenceNumber {
                transaction_with_proof,
                proof_of_current_sequence_number,
            } => Ok((transaction_with_proof, proof_of_current_sequence_number)),
            _ => bail!("Not ResponseItem::GetAccountTransactionBySequenceNumber."),
        }
    }

    pub fn into_get_events_by_access_path_response(
        self,
    ) -> Result<(Vec<EventWithProof>, AccountStateWithProof)> {
        match self {
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            } => Ok((events_with_proof, proof_of_latest_event)),
            _ => bail!("Not ResponseItem::GetEventsByEventAccessPath."),
        }
    }

    pub fn into_get_transactions_response(self) -> Result<TransactionListWithProof> {
        match self {
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            } => Ok(txn_list_with_proof),
            _ => bail!("Not ResponseItem::GetTransactions."),
        }
    }
}
