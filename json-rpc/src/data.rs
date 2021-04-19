// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::JsonRpcError,
    util::{transaction_data_view_from_transaction, vm_status_view_from_kept_vm_status},
    views::{
        AccountStateWithProofView, AccountView, BytesView, CurrencyInfoView, EventView,
        EventWithProofView, MetadataView, StateProofView, TransactionView, TransactionsProofsView,
        TransactionsWithProofsView,
    },
};
use anyhow::{format_err, Result};
use diem_crypto::{hash::CryptoHash, HashValue};
use diem_types::{
    account_address::AccountAddress,
    account_config::{
        diem_root_address, from_currency_code_string, resources::dual_attestation::Limit,
        AccountResource,
    },
    account_state::AccountState,
    chain_id::ChainId,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
};
use network::counters;
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
    ledger_version: u64,
    account_address: AccountAddress,
    version: u64,
) -> Result<Option<AccountView>, JsonRpcError> {
    let account_state = match get_account_state(db, account_address, version)? {
        Some(val) => val,
        None => return Ok(None),
    };

    let currencies = get_currencies(db, ledger_version)?;
    let currency_codes: Vec<_> = currencies
        .into_iter()
        .map(|info| from_currency_code_string(&info.code))
        .collect::<Result<_, _>>()?;

    Ok(Some(AccountView::try_from_account_state(
        account_address,
        account_state,
        &currency_codes,
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
) -> Result<Vec<TransactionView>, JsonRpcError> {
    if start_version > ledger_version {
        return Ok(vec![]);
    }

    let txs = db.get_transactions(start_version, limit, ledger_version, include_events)?;

    let mut result = vec![];
    if txs.is_empty() {
        return Ok(result);
    }

    let all_events = if include_events {
        txs.events
            .ok_or_else(|| format_err!("Storage layer didn't return events when requested!"))?
    } else {
        vec![]
    };

    let txs_with_info = txs
        .transactions
        .into_iter()
        .zip(txs.proof.transaction_infos().iter());

    for (v, (tx, info)) in txs_with_info.enumerate() {
        let events = if include_events {
            all_events
                .get(v)
                .ok_or_else(|| format_err!("Missing events for version: {}", v))?
                .iter()
                .cloned()
                .map(|x| (start_version + v as u64, x).try_into())
                .collect::<Result<Vec<EventView>>>()?
        } else {
            vec![]
        };

        result.push(TransactionView {
            version: start_version + v as u64,
            hash: tx.hash(),
            bytes: bcs::to_bytes(&tx)?.into(),
            transaction: transaction_data_view_from_transaction(tx),
            events,
            vm_status: vm_status_view_from_kept_vm_status(info.status()),
            gas_used: info.gas_used(),
        });
    }
    Ok(result)
}

/// Returns transactions by range with proofs
pub fn get_transactions_with_proofs(
    db: &dyn DbReader,
    ledger_version: u64,
    start_version: u64,
    limit: u64,
) -> Result<Option<TransactionsWithProofsView>, JsonRpcError> {
    if start_version > ledger_version {
        return Ok(None);
    }

    // We do not fetch events since they don't come with proofs.
    let txs = db.get_transactions(start_version, limit, ledger_version, false)?;

    let mut blobs = vec![];
    for t in txs.transactions.iter() {
        let bv = bcs::to_bytes(t)?;
        blobs.push(BytesView::from(bv));
    }

    let (proofs, tx_info) = txs.proof.unpack();

    Ok(Some(TransactionsWithProofsView {
        serialized_transactions: blobs,
        proofs: TransactionsProofsView {
            ledger_info_to_transaction_infos_proof: BytesView::from(bcs::to_bytes(&proofs)?),
            transaction_infos: BytesView::from(bcs::to_bytes(&tx_info)?),
        },
    }))
}

/// Returns account transaction by account and sequence_number
pub fn get_account_transaction(
    db: &dyn DbReader,
    ledger_version: u64,
    account: AccountAddress,
    sequence_number: u64,
    include_events: bool,
) -> Result<Option<TransactionView>, JsonRpcError> {
    let tx = db.get_txn_by_account(account, sequence_number, ledger_version, include_events)?;

    if let Some(tx) = tx {
        if include_events && tx.events.is_none() {
            return Err(JsonRpcError::internal_error(
                "Storage layer didn't return events when requested!".to_owned(),
            ));
        }
        let tx_version = tx.version;

        let events = tx
            .events
            .unwrap_or_default()
            .into_iter()
            .map(|x| (tx_version, x).try_into())
            .collect::<Result<Vec<EventView>>>()?;

        Ok(Some(TransactionView {
            version: tx_version,
            hash: tx.transaction.hash(),
            bytes: bcs::to_bytes(&tx.transaction)?.into(),
            transaction: transaction_data_view_from_transaction(tx.transaction),
            events,
            vm_status: vm_status_view_from_kept_vm_status(tx.proof.transaction_info().status()),
            gas_used: tx.proof.transaction_info().gas_used(),
        }))
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
            .get_txn_by_account(account, seq, ledger_version, include_events)?
            .ok_or_else(|| format_err!("Can not find transaction for seq {}!", seq))?;

        let tx_version = tx.version;
        let events = if include_events {
            if tx.events.is_none() {
                return Err(JsonRpcError::internal_error(
                    "Storage layer didn't return events when requested!".to_owned(),
                ));
            }
            tx.events
                .unwrap_or_default()
                .into_iter()
                .map(|x| ((tx_version, x).try_into()))
                .collect::<Result<Vec<EventView>>>()?
        } else {
            vec![]
        };

        all_txs.push(TransactionView {
            version: tx.version,
            hash: tx.transaction.hash(),
            bytes: bcs::to_bytes(&tx.transaction)?.into(),
            transaction: transaction_data_view_from_transaction(tx.transaction),
            events,
            vm_status: vm_status_view_from_kept_vm_status(tx.proof.transaction_info().status()),
            gas_used: tx.proof.transaction_info().gas_used(),
        });
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

    let mut results = vec![];

    for event in events_with_proofs
        .into_iter()
        .filter(|e| e.transaction_version <= ledger_version)
    {
        results.push(EventWithProofView {
            event_with_proof: bcs::to_bytes(&event)?.into(),
        });
    }

    Ok(results)
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
pub fn get_network_status(role: &str) -> Result<u64, JsonRpcError> {
    let peers = counters::DIEM_NETWORK_PEERS
        .get_metric_with_label_values(&[role, "connected"])
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    Ok(peers.get() as u64)
}

/// Returns proof of new state relative to version known to client
pub fn get_state_proof(
    db: &dyn DbReader,
    version: u64,
    ledger_info: &LedgerInfoWithSignatures,
) -> Result<StateProofView, JsonRpcError> {
    let proofs = db.get_state_proof_with_ledger_info(version, ledger_info.clone())?;
    StateProofView::try_from((ledger_info.clone(), proofs.0, proofs.1)).map_err(Into::into)
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
