// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module contains RPC method handlers for Full Node JSON-RPC interface
use crate::{
    errors::JsonRpcError,
    views::{
        AccountStateWithProofView, AccountView, BlockMetadata, EventView, StateProofView,
        TransactionView,
    },
};
use anyhow::{ensure, format_err, Error, Result};
use core::future::Future;
use debug_interface::prelude::*;
use futures::{channel::oneshot, SinkExt};
use hex;
use libra_mempool::MempoolClientSender;
use libra_types::{
    account_address::AccountAddress, account_state::AccountState, event::EventKey,
    mempool_status::MempoolStatusCode, transaction::SignedTransaction,
};
use libradb::LibraDBTrait;
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, pin::Pin, str::FromStr, sync::Arc};

#[derive(Clone)]
pub(crate) struct JsonRpcService {
    db: Arc<dyn LibraDBTrait>,
    mempool_sender: MempoolClientSender,
}

impl JsonRpcService {
    pub fn new(db: Arc<dyn LibraDBTrait>, mempool_sender: MempoolClientSender) -> Self {
        Self { db, mempool_sender }
    }
}

type RpcHandler =
    Box<fn(JsonRpcService, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>>;

pub(crate) type RpcRegistry = HashMap<String, RpcHandler>;

/// Submits transaction to full node
async fn submit(mut service: JsonRpcService, params: Vec<Value>) -> Result<()> {
    let txn_payload: String = serde_json::from_value(params[0].clone())?;
    let transaction: SignedTransaction = lcs::from_bytes(&hex::decode(txn_payload)?)?;
    trace_code_block!("json-rpc::submit", {"txn", transaction.sender(), transaction.sequence_number()});

    let (req_sender, callback) = oneshot::channel();
    service
        .mempool_sender
        .send((transaction, req_sender))
        .await?;
    let (mempool_status, vm_status) = callback.await??;

    if let Some(vm_error) = vm_status {
        Err(Error::new(JsonRpcError::vm_error(vm_error)))
    } else if mempool_status.code == MempoolStatusCode::Accepted {
        Ok(())
    } else {
        Err(format_err!(
            "Mempool insertion failed: {:?}",
            mempool_status
        ))
    }
}

/// Returns account state (AccountView) by given address
async fn get_account_state(
    service: JsonRpcService,
    params: Vec<Value>,
) -> Result<Option<AccountView>> {
    let address: String = serde_json::from_value(params[0].clone())?;
    let account_address = AccountAddress::from_str(&address)?;
    let response = service.db.get_latest_account_state(account_address)?;
    if let Some(blob) = response {
        let account_state = AccountState::try_from(&blob)?;
        if let Some(account) = account_state.get_account_resource()? {
            if let Some(balance) = account_state.get_balance_resource()? {
                return Ok(Some(AccountView::new(&account, &balance)));
            }
        }
    }
    Ok(None)
}

/// Returns the current blockchain metadata
/// Can be used to verify that target Full Node is up-to-date
async fn get_metadata(service: JsonRpcService, _: Vec<Value>) -> Result<BlockMetadata> {
    let (version, timestamp) = service.db.get_latest_commit_metadata()?;
    Ok(BlockMetadata { version, timestamp })
}

/// Returns transactions by range
async fn get_transactions(
    service: JsonRpcService,
    params: Vec<Value>,
) -> Result<Vec<TransactionView>> {
    let start_version: u64 = serde_json::from_value(params[0].clone())?;
    let limit: u64 = serde_json::from_value(params[1].clone())?;
    let include_events: bool = serde_json::from_value(params[2].clone())?;

    ensure!(
        limit > 0 && limit <= 1000,
        "limit must be smaller than 1000"
    );

    let txs = service.db.get_transactions(
        start_version,
        limit,
        service.db.get_latest_version()?,
        include_events,
    )?;

    let mut result = vec![];

    let all_events = if include_events {
        txs.events
            .ok_or_else(|| format_err!("Storage layer didn't return events when requested!"))?
    } else {
        vec![]
    };

    for (v, tx) in txs.transactions.into_iter().enumerate() {
        let events = if include_events {
            all_events
                .get(v)
                .ok_or_else(|| format_err!("Missing events for version: {}", v))?
                .iter()
                .cloned()
                .map(|x| (start_version + v as u64, x).into())
                .collect()
        } else {
            vec![]
        };

        result.push(TransactionView {
            version: start_version + v as u64,
            transaction: tx.into(),
            events,
        });
    }
    Ok(result)
}

/// Returns account transaction by account and sequence_number
async fn get_account_transaction(
    service: JsonRpcService,
    params: Vec<Value>,
) -> Result<Option<TransactionView>> {
    let p_account: String = serde_json::from_value(params[0].clone())?;
    let sequence: u64 = serde_json::from_value(params[1].clone())?;
    let include_events: bool = serde_json::from_value(params[2].clone())?;

    let account = AccountAddress::try_from(p_account)?;

    let version = service.db.get_latest_version()?;

    let tx = service
        .db
        .get_txn_by_account(account, sequence, version, include_events)?;

    if let Some(tx) = tx {
        if include_events {
            ensure!(
                tx.events.is_some(),
                "Storage layer didn't return events when requested!"
            );
        }
        let tx_version = tx.version;

        let events = tx
            .events
            .unwrap_or_default()
            .into_iter()
            .map(|x| ((tx_version, x).into()))
            .collect();

        Ok(Some(TransactionView {
            version: tx_version,
            transaction: tx.transaction.into(),
            events,
        }))
    } else {
        Ok(None)
    }
}

/// Returns events by given access path
async fn get_events(service: JsonRpcService, params: Vec<Value>) -> Result<Vec<EventView>> {
    let raw_event_key: String = serde_json::from_value(params[0].clone())?;
    let start: u64 = serde_json::from_value(params[1].clone())?;
    let limit: u64 = serde_json::from_value(params[2].clone())?;

    let event_key = EventKey::try_from(&hex::decode(raw_event_key)?[..])?;
    let events_with_proof = service.db.get_events(&event_key, start, limit)?;

    let mut events = vec![];
    for (version, event) in events_with_proof {
        events.push((version, event).into());
    }
    Ok(events)
}

/// Returns proof of new state relative to version known to client
async fn get_state_proof(service: JsonRpcService, params: Vec<Value>) -> Result<StateProofView> {
    let known_version: u64 = serde_json::from_value(params[0].clone())?;
    StateProofView::try_from(service.db.get_state_proof(known_version)?)
}

async fn get_account_state_with_proof(
    service: JsonRpcService,
    params: Vec<Value>,
) -> Result<AccountStateWithProofView> {
    let address: String = serde_json::from_value(params[0].clone())?;
    let account_address = AccountAddress::from_str(&address)?;
    let version: u64 = serde_json::from_value(params[1].clone())?;

    let account_state_with_proof =
        service
            .db
            .get_account_state_with_proof(account_address, version, version)?;
    Ok(AccountStateWithProofView::try_from(
        account_state_with_proof,
    )?)
}

/// Builds registry of all available RPC methods
/// To register new RPC method, add it via `register_rpc_method!` macros call
/// Note that RPC method name will equal to name of function
pub(crate) fn build_registry() -> RpcRegistry {
    let mut registry = RpcRegistry::new();
    register_rpc_method!(registry, submit, 1);
    register_rpc_method!(registry, get_metadata, 0);
    register_rpc_method!(registry, get_account_state, 1);
    register_rpc_method!(registry, get_transactions, 3);
    register_rpc_method!(registry, get_account_transaction, 3);
    register_rpc_method!(registry, get_events, 3);

    register_rpc_method!(registry, get_state_proof, 1);
    register_rpc_method!(registry, get_account_state_with_proof, 3);

    registry
}
