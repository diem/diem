// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module contains RPC method handlers for Full Node JSON-RPC interface
use crate::views::{AccountView, BlockMetadata};
use anyhow::{ensure, Result};
use core::future::Future;
use libra_types::{account_address::AccountAddress, account_config::AccountResource};
use libradb::LibraDB;
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, pin::Pin, str::FromStr, sync::Arc};

type RpcHandler =
    Box<fn(Arc<LibraDB>, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>>;
pub(crate) type RpcRegistry = HashMap<String, RpcHandler>;

/// Returns account state (AccountView) by given address
async fn get_account_state(
    libra_db: Arc<LibraDB>,
    params: Vec<Value>,
) -> Result<Option<AccountView>> {
    let address: String = serde_json::from_value(params[0].clone())?;

    let account_address = AccountAddress::from_str(&address)?;
    let response = libra_db.get_latest_account_state(account_address)?;
    if let Some(blob) = response {
        if let Ok(account) = AccountResource::try_from(&blob) {
            return Ok(Some(AccountView::new(account)));
        }
    }
    Ok(None)
}

/// Returns the current blockchain metadata
/// Can be used to verify that target Full Node is up-to-date
async fn get_metadata(libra_db: Arc<LibraDB>, _params: Vec<Value>) -> Result<BlockMetadata> {
    let (version, timestamp) = libra_db.get_latest_commit_metadata()?;
    Ok(BlockMetadata { version, timestamp })
}

/// Builds registry of all available RPC methods
/// To register new RPC method, add it via `register_rpc_method!` macros call
/// Note that RPC method name will equal to name of function
pub(crate) fn build_registry() -> RpcRegistry {
    let mut registry = RpcRegistry::new();
    register_rpc_method!(registry, get_account_state, 1);
    register_rpc_method!(registry, get_metadata, 0);

    registry
}
