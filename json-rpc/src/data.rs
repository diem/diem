// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::JsonRpcError,
    views::{
        AccountStateWithProofView, AccountView, AccumulatorConsistencyProofView, CurrencyInfoView,
        EventView, EventWithProofView, MetadataView, StateProofView, TransactionListView,
        TransactionView, TransactionsWithProofsView,
    },
};
use anyhow::{format_err, Result};
use diem_crypto::HashValue;
use diem_types::{
    account_address::AccountAddress,
    account_config::{diem_root_address, resources::dual_attestation::Limit, AccountResource},
    account_state::AccountState,
    chain_id::ChainId,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
};
use std::{
    cmp::min,
    convert::{TryFrom, TryInto},
};
use storage_interface::{DbReader, Order};

pub fn get_account_state(
    db: &dyn DbReader,
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
    db: &dyn DbReader,
    ledger_version: u64,
    chain_id: ChainId,
    version: u64,
) -> Result<MetadataView, JsonRpcError> {
    let mut script_hash_allow_list: Option<Vec<HashValue>> = None;
    let mut module_publishing_allowed: Option<bool> = None;
    let mut diem_version: Option<u64> = None;
    let mut dual_attestation_limit: Option<u64> = None;
    if version == ledger_version {
        if let Some(account) = get_account_state(db, diem_root_address(), version)? {
            if let Some(vm_publishing_option) = account.get_vm_publishing_option()? {
                script_hash_allow_list = Some(vm_publishing_option.script_allow_list);

                module_publishing_allowed = Some(vm_publishing_option.is_open_module);
            }
            if let Some(v) = account.get_diem_version()? {
                diem_version = Some(v.major)
            }
            if let Some(limit) = account.get_resource::<Limit>()? {
                dual_attestation_limit = Some(limit.micro_xdx_limit)
            }
        }
    }
    Ok(MetadataView {
        version,
        accumulator_root_hash: db.get_accumulator_root_hash(version)?,
        timestamp: db.get_block_timestamp(version)?,
        chain_id: chain_id.id(),
        script_hash_allow_list,
        module_publishing_allowed,
        diem_version,
        dual_attestation_limit,
    })
}

/// Returns account state (AccountView) by given address
pub fn get_account(
    db: &dyn DbReader,
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
    db: &dyn DbReader,
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
    db: &dyn DbReader,
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
    db: &dyn DbReader,
    ledger_version: u64,
    account: AccountAddress,
    sequence_number: u64,
    include_events: bool,
) -> Result<Option<TransactionView>, JsonRpcError> {
    let tx =
        db.get_account_transaction(account, sequence_number, ledger_version, include_events)?;

    if let Some(tx) = tx {
        Ok(Some(TransactionView::try_from_tx_and_events(
            tx.version,
            tx.transaction,
            tx.proof.transaction_info,
            tx.events.unwrap_or_default(),
        )?))
    } else {
        Ok(None)
    }
}

/// Returns all account transactions
pub fn get_account_transactions(
    db: &dyn DbReader,
    ledger_version: u64,
    account: AccountAddress,
    start: u64,
    limit: u64,
    include_events: bool,
) -> Result<Vec<TransactionView>, JsonRpcError> {
    let account_state = db.get_latest_account_state(account)?.ok_or_else(|| {
        JsonRpcError::invalid_request_with_msg(format!(
            "could not find account by address {}",
            account
        ))
    })?;
    let account_seq = AccountResource::try_from(&account_state)?.sequence_number();

    if start >= account_seq {
        return Ok(vec![]);
    }

    let mut all_txs = vec![];
    let end = min(
        start
            .checked_add(limit)
            .ok_or_else(|| format_err!("overflow!"))?,
        account_seq,
    );

    for seq in start..end {
        let tx = db
            .get_account_transaction(account, seq, ledger_version, include_events)?
            .ok_or_else(|| format_err!("Can not find transaction for seq {}!", seq))?;

        let tx_view = TransactionView::try_from_tx_and_events(
            tx.version,
            tx.transaction,
            tx.proof.transaction_info,
            tx.events.unwrap_or_default(),
        )?;
        all_txs.push(tx_view);
    }

    Ok(all_txs)
}

/// Returns events by given access path
pub fn get_events(
    db: &dyn DbReader,
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
    db: &dyn DbReader,
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

/// Returns meta information about supported currencies
pub fn get_currencies(
    db: &dyn DbReader,
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
    db: &dyn DbReader,
    version: u64,
    ledger_info: &LedgerInfoWithSignatures,
) -> Result<StateProofView, JsonRpcError> {
    let proofs = db.get_state_proof_with_ledger_info(version, &ledger_info)?;
    StateProofView::try_from((ledger_info.clone(), proofs.0, proofs.1)).map_err(Into::into)
}

/// Returns a proof that allows a client to extend their accumulator summary from
/// `client_known_version` (or pre-genesis if `None`) to `ledger_version`.
///
/// See [`DbReader::get_accumulator_consistency_proof`]
pub fn get_accumulator_consistency_proof(
    db: &dyn DbReader,
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
    db: &dyn DbReader,
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
