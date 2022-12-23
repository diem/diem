// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::JsonRpcError,
    views::{
        AccountStateWithProofView, AccountTransactionsWithProofView, AccountView,
        AccumulatorConsistencyProofView, CurrencyInfoView, EventByVersionWithProofView, EventView,
        EventWithProofView, MetadataView, StateProofView, TransactionListView, TransactionView,
        TransactionsWithProofsView,
    },
};
use anyhow::Result;
use diem_types::{
    account_address::AccountAddress, account_config::diem_root_address,
    account_state::AccountState, chain_id::ChainId, event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
};
use resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};
use storage_interface::{MoveDbReader, Order};

pub fn get_account_state(
    db: &dyn MoveDbReader,
    address: AccountAddress,
    version: u64,
) -> Result<Option<AccountState>> {
    if let Some(blob) = db
        .get_account_state_with_proof_by_version(address, version)?
        .0
    {
        Ok(Some(AccountState::try_from(&blob)?))
    } else {
        Ok(None)
    }
}

/// Returns the blockchain metadata for a specified version. If no version is specified, default to
/// returning the current blockchain metadata
/// Can be used to verify that target Full Node is up-to-date
pub fn get_metadata(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    chain_id: ChainId,
    version: u64,
) -> Result<MetadataView, JsonRpcError> {
    let accumulator_root_hash = db.get_accumulator_root_hash(version)?;
    let timestamp = db.get_block_timestamp(version)?;
    let mut metadata_view =
        MetadataView::new(version, accumulator_root_hash, timestamp, chain_id.id());
    if version == ledger_version {
        if let Some(diem_root) = get_account_state(db, diem_root_address(), version)? {
            metadata_view.with_diem_root(&diem_root)?;
        }
    }
    Ok(metadata_view)
}

/// Returns account state (AccountView) by given address
pub fn get_account(
    db: &dyn MoveDbReader,
    account_address: AccountAddress,
    version: u64,
) -> Result<Option<AccountView>, JsonRpcError> {
    let account_state = match get_account_state(db, account_address, version)? {
        Some(val) => val,
        None => return Ok(None),
    };

    Ok(Some(AccountView::try_from_account_state(
        account_address,
        account_state,
        version,
    )?))
}

/// Returns transactions by range
pub fn get_transactions(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    start_version: u64,
    limit: u64,
    include_events: bool,
) -> Result<TransactionListView, JsonRpcError> {
    if start_version > ledger_version || limit == 0 {
        return Ok(TransactionListView::empty());
    }
    let txs = db.get_transactions(start_version, limit, ledger_version, include_events)?;
    Ok(TransactionListView::try_from(txs)?)
}

/// Returns transactions by range with proofs
pub fn get_transactions_with_proofs(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    start_version: u64,
    limit: u64,
    include_events: bool,
) -> Result<Option<TransactionsWithProofsView>, JsonRpcError> {
    if start_version > ledger_version || limit == 0 {
        return Ok(None);
    }
    let txs = db.get_transactions(start_version, limit, ledger_version, include_events)?;
    Ok(Some(TransactionsWithProofsView::try_from(&txs)?))
}

/// Returns account transaction by account and sequence_number
pub fn get_account_transaction(
    db: &dyn MoveDbReader,
    account: AccountAddress,
    seq_num: u64,
    include_events: bool,
    ledger_version: u64,
) -> Result<Option<TransactionView>, JsonRpcError> {
    let tx = db
        .get_account_transaction(account, seq_num, include_events, ledger_version)?
        .map(|tx| {
            TransactionView::try_from_tx_and_events(
                tx.version,
                tx.transaction,
                tx.proof.transaction_info,
                tx.events.unwrap_or_default(),
            )
        })
        .transpose()?;
    Ok(tx)
}

/// Returns all account transactions
pub fn get_account_transactions(
    db: &dyn MoveDbReader,
    account: AccountAddress,
    start_seq_num: u64,
    limit: u64,
    include_events: bool,
    ledger_version: u64,
) -> Result<Vec<TransactionView>, JsonRpcError> {
    let acct_txs = db.get_account_transactions(
        account,
        start_seq_num,
        limit,
        include_events,
        ledger_version,
    )?;
    let txs = TransactionListView::try_from(acct_txs)?;
    Ok(txs.0)
}

/// Return a serialized list of an account's transactions along with a proof for
/// each transaction.
pub fn get_account_transactions_with_proofs(
    db: &dyn MoveDbReader,
    account: AccountAddress,
    start: u64,
    limit: u64,
    include_events: bool,
    ledger_version: u64,
) -> Result<AccountTransactionsWithProofView, JsonRpcError> {
    let acct_txns_with_proof =
        db.get_account_transactions(account, start, limit, include_events, ledger_version)?;
    Ok(AccountTransactionsWithProofView::try_from(
        &acct_txns_with_proof,
    )?)
}

/// Returns events by given access path
pub fn get_events(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    event_key: EventKey,
    start: u64,
    limit: u64,
) -> Result<Vec<EventView>, JsonRpcError> {
    let events_raw = db.get_events(&event_key, start, Order::Ascending, limit)?;

    let events = events_raw
        .into_iter()
        .filter(|(version, _event)| version <= &ledger_version)
        .map(|event| event.try_into())
        .collect::<Result<Vec<EventView>>>()?;

    Ok(events)
}

/// Returns events by given access path along with their proofs
pub fn get_events_with_proofs(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    event_key: EventKey,
    start: u64,
    limit: u64,
) -> Result<Vec<EventWithProofView>, JsonRpcError> {
    let events_with_proofs = db.get_events_with_proofs(
        &event_key,
        start,
        Order::Ascending,
        limit,
        Some(ledger_version),
    )?;

    let views = events_with_proofs
        .iter()
        .map(EventWithProofView::try_from)
        .collect::<Result<Vec<_>>>()?;

    Ok(views)
}

/// Returns the latest event at or below the requested version along with proof.
pub fn get_event_by_version_with_proof(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    event_key: EventKey,
    version: u64,
) -> Result<EventByVersionWithProofView, JsonRpcError> {
    let event_by_version =
        db.get_event_by_version_with_proof(&event_key, version, ledger_version)?;
    EventByVersionWithProofView::try_from(&event_by_version).map_err(Into::into)
}

/// Returns meta information about supported currencies
pub fn get_currencies(
    db: &dyn MoveDbReader,
    ledger_version: u64,
) -> Result<Vec<CurrencyInfoView>, JsonRpcError> {
    if let Some(account_state) = get_account_state(db, diem_root_address(), ledger_version)? {
        Ok(account_state
            .get_registered_currency_info_resources()?
            .iter()
            .map(|info| info.into())
            .collect())
    } else {
        Ok(vec![])
    }
}

/// Returns the number of peers this node is connected to
pub fn get_network_status(_role: &str) -> Result<u64, JsonRpcError> {
    // TODO: The underlying metric is deprecated, and we need a different way of communicating this number that doesn't need the peer Id
    Ok(0)
}

/// Returns proof of new state relative to version known to client
pub fn get_state_proof(
    db: &dyn MoveDbReader,
    version: u64,
    ledger_info: LedgerInfoWithSignatures,
) -> Result<StateProofView, JsonRpcError> {
    let state_proof = db.get_state_proof_with_ledger_info(version, ledger_info)?;
    StateProofView::try_from(&state_proof).map_err(Into::into)
}

/// Returns a proof that allows a client to extend their accumulator summary from
/// `client_known_version` (or pre-genesis if `None`) to `ledger_version`.
///
/// See [`MoveDbReader::get_accumulator_consistency_proof`]
pub fn get_accumulator_consistency_proof(
    db: &dyn MoveDbReader,
    client_known_version: Option<u64>,
    ledger_version: u64,
) -> Result<AccumulatorConsistencyProofView, JsonRpcError> {
    if let Some(client_known_version) = client_known_version {
        if client_known_version > ledger_version {
            return Err(JsonRpcError::invalid_request_with_msg(format!(
                "client_known_version({}) should be <= ledger_version({})",
                client_known_version, ledger_version,
            )));
        }
    }
    let proof = db.get_accumulator_consistency_proof(client_known_version, ledger_version)?;
    AccumulatorConsistencyProofView::try_from(&proof).map_err(Into::into)
}

/// Returns the account state to the client, alongside a proof relative to the version and
/// ledger_version specified by the client. If version or ledger_version are not specified,
/// the latest known versions will be used.
pub fn get_account_state_with_proof(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    account_address: AccountAddress,
    version: u64,
) -> Result<AccountStateWithProofView, JsonRpcError> {
    if version > ledger_version {
        return Err(JsonRpcError::invalid_request_with_msg(format!(
            "version({}) should <= ledger version({})",
            version, ledger_version
        )));
    }
    let account_state_with_proof =
        db.get_account_state_with_proof(account_address, version, ledger_version)?;
    Ok(AccountStateWithProofView::try_from(
        account_state_with_proof,
    )?)
}

/// Get all resources stored under `account_address` at `version`
pub fn get_resources(
    db: &dyn MoveDbReader,
    ledger_version: u64,
    account_address: AccountAddress,
    version: u64,
) -> Result<BTreeMap<String, AnnotatedMoveStruct>, JsonRpcError> {
    let account_state_with_proof =
        db.get_account_state_with_proof(account_address, version, ledger_version)?;
    let mut resources = BTreeMap::new();
    if let Some(account_state_blob) = account_state_with_proof.blob {
        let account_state = AccountState::try_from(&account_state_blob)
            .map_err(|e| JsonRpcError::internal_error(format!("{:?}", e)))?;
        let annotator = MoveValueAnnotator::new(&db);
        for (typ, bytes) in account_state.get_resources() {
            let resource = annotator.view_resource(&typ, bytes)?;
            resources.insert(format!("{}", typ), resource);
        }
    }
    Ok(resources)
}
