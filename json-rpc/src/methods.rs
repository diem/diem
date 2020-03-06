// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module contains RPC method handlers for Full Node JSON-RPC interface
use crate::views::{AccountView, BlockMetadata, EventView, TransactionView};
use anyhow::{ensure, format_err, Result};
use core::future::Future;
use futures::{channel::oneshot, SinkExt};
use hex;
use libra_mempool::MempoolClientSender;
use libra_types::{
    account_address::AccountAddress, account_config::AccountResource, event::EventKey,
    mempool_status::MempoolStatusCode,
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
    let transaction = lcs::from_bytes(&hex::decode(txn_payload)?)?;

    let (req_sender, callback) = oneshot::channel();
    service
        .mempool_sender
        .send((transaction, req_sender))
        .await?;
    let (mempool_status, vm_status) = callback.await??;

    if let Some(vm_error) = vm_status {
        Err(format_err!("VM validation error: {:?}", vm_error))
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
        if let Ok(account) = AccountResource::try_from(&blob) {
            return Ok(Some(AccountView::new(&account)));
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

    for (v, tx) in txs.transactions.into_iter().enumerate() {
        result.push(TransactionView {
            version: start_version + v as u64,
            transaction: tx.into(),
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

    Ok(service
        .db
        .get_txn_by_account(
            account,
            sequence,
            service.db.get_latest_version()?,
            include_events,
        )?
        .map(|tx| TransactionView {
            version: tx.version,
            transaction: tx.transaction.into(),
        }))
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
        events.push(EventView::try_from((version, event))?);
    }
    Ok(events)
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
    registry
}
