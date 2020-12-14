// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module contains RPC method handlers for Full Node JSON-RPC interface
use crate::{
    errors::JsonRpcError,
    views::{
        AccountStateWithProofView, AccountView, BytesView, CurrencyInfoView, EventView,
        MetadataView, StateProofView, TransactionView, TransactionsProofsView,
        TransactionsWithProofsView,
    },
};
use anyhow::{ensure, format_err, Error, Result};
use core::future::Future;
use diem_config::config::RoleType;
use diem_crypto::hash::CryptoHash;
use diem_mempool::MempoolClientSender;
use diem_trace::prelude::*;
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
    mempool_status::MempoolStatusCode,
    transaction::SignedTransaction,
};
use fail::fail_point;
use futures::{channel::oneshot, SinkExt};
use network::counters;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    cmp::min,
    collections::HashMap,
    convert::{TryFrom, TryInto},
    pin::Pin,
    sync::Arc,
};
use storage_interface::{DbReader, Order};

#[derive(Clone)]
pub(crate) struct JsonRpcService {
    db: Arc<dyn DbReader>,
    mempool_sender: MempoolClientSender,
    role: RoleType,
    chain_id: ChainId,
    batch_size_limit: u16,
    page_size_limit: u16,
}

impl JsonRpcService {
    pub fn new(
        db: Arc<dyn DbReader>,
        mempool_sender: MempoolClientSender,
        role: RoleType,
        chain_id: ChainId,
        batch_size_limit: u16,
        page_size_limit: u16,
    ) -> Self {
        Self {
            db,
            mempool_sender,
            role,
            chain_id,
            batch_size_limit,
            page_size_limit,
        }
    }

    fn get_account_state(
        &self,
        address: AccountAddress,
        version: u64,
    ) -> Result<Option<AccountState>> {
        if let Some(blob) = self
            .db
            .get_account_state_with_proof_by_version(address, version)?
            .0
        {
            Ok(Some(AccountState::try_from(&blob)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        fail_point!("jsonrpc::get_latest_ledger_info", |_| {
            Err(anyhow::anyhow!(
                "Injected error for get latest ledger info error"
            ))
        });

        self.db.get_latest_ledger_info()
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn validate_batch_size_limit(&self, size: usize) -> Result<(), JsonRpcError> {
        self.validate_size_limit("batch size", self.batch_size_limit, size)
    }

    pub fn validate_page_size_limit(&self, size: usize) -> Result<(), JsonRpcError> {
        self.validate_size_limit("page size", self.page_size_limit, size)
    }

    fn validate_size_limit(&self, name: &str, limit: u16, size: usize) -> Result<(), JsonRpcError> {
        if size > limit as usize {
            Err(JsonRpcError::invalid_request_with_msg(format!(
                "{} = {}, exceed limit {}",
                name, size, limit
            )))
        } else {
            Ok(())
        }
    }
}

type RpcHandler =
    Box<fn(JsonRpcService, JsonRpcRequest) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>>;

pub(crate) type RpcRegistry = HashMap<String, RpcHandler>;

pub(crate) struct JsonRpcRequest {
    pub trace_id: String,
    pub params: Vec<Value>,
    pub ledger_info: LedgerInfoWithSignatures,
}

impl JsonRpcRequest {
    /// Returns the request parameter at the given index.
    /// Returns Null if given index is out of bounds.
    fn get_param(&self, index: usize) -> Value {
        self.get_param_with_default(index, Value::Null)
    }

    /// Returns the request parameter at the given index.
    /// Returns default Value if given index is out of bounds.
    fn get_param_with_default(&self, index: usize, default: Value) -> Value {
        if self.params.len() > index {
            return self.params[index].clone();
        }
        default
    }

    fn version(&self) -> u64 {
        self.ledger_info.ledger_info().version()
    }

    /// Return AccountAddress by try parse from params[index]
    fn parse_account_address(&self, index: usize) -> Result<AccountAddress, JsonRpcError> {
        self.try_parse_param(index, "account address")
    }

    /// Return type T instance with 2 steps parsing:
    ///   1. deserialize params[index] into String type.
    ///   2. call TryFrom<String> to create target T instance.
    /// The name argument is for creating helpful error messsage in case deserialization
    /// failed.
    fn try_parse_param<T>(&self, index: usize, name: &str) -> Result<T, JsonRpcError>
    where
        T: TryFrom<String>,
    {
        let raw_str: String = self.parse_param(index, name)?;
        Ok(T::try_from(raw_str).map_err(|_| invalid_param(index, name))?)
    }

    /// Return native type of params[index] deserialized by from json value.
    /// The name argument is for creating helpful error messsage in case deserialization
    /// failed.
    fn parse_param<T>(&self, index: usize, name: &str) -> Result<T, JsonRpcError>
    where
        T: DeserializeOwned,
    {
        Ok(
            serde_json::from_value(self.get_param(index))
                .map_err(|_| invalid_param(index, name))?,
        )
    }

    fn parse_version_param(&self, index: usize, name: &str) -> Result<u64, JsonRpcError> {
        if self.get_param(index).is_null() {
            return Ok(self.version());
        }
        let version: u64 = self.parse_param(index, name)?;
        if version > self.version() {
            return Err(JsonRpcError::invalid_param(
                index,
                name,
                format!("<= known latest version {}", self.version()).as_str(),
            ));
        }
        Ok(version)
    }

    fn parse_signed_transaction(
        &self,
        index: usize,
        name: &str,
    ) -> Result<SignedTransaction, JsonRpcError> {
        Ok(self
            ._parse_signed_transaction(self.get_param(index))
            .map_err(|_| invalid_param(index, name))?)
    }

    fn parse_event_key(&self, index: usize, name: &str) -> Result<EventKey, JsonRpcError> {
        Ok(self
            ._parse_event_key(self.get_param(index))
            .map_err(|_| invalid_param(index, name))?)
    }

    // the following methods should not be called directly as they return error causes internal error
    // call related wrapper method for parsing param.
    fn _parse_event_key(&self, val: Value) -> Result<EventKey> {
        let raw: String = serde_json::from_value(val)?;
        Ok(EventKey::try_from(&hex::decode(raw)?[..])?)
    }

    fn _parse_signed_transaction(&self, val: Value) -> Result<SignedTransaction> {
        let raw: String = serde_json::from_value(val)?;
        Ok(bcs::from_bytes(&hex::decode(raw)?)?)
    }
}

/// Submits transaction to full node
async fn submit(mut service: JsonRpcService, request: JsonRpcRequest) -> Result<()> {
    let transaction = request.parse_signed_transaction(0, "data")?;

    trace_code_block!("json-rpc::submit", {"txn", transaction.sender(), transaction.sequence_number()});

    let (req_sender, callback) = oneshot::channel();

    fail_point!("jsonrpc::method::submit::mempool_sender", |_| {
        Err(anyhow::anyhow!("Injected error for mempool_sender call error").into())
    });

    service
        .mempool_sender
        .send((transaction, req_sender))
        .await?;
    let (mempool_status, vm_status_opt) = callback.await??;

    if let Some(vm_status) = vm_status_opt {
        Err(Error::new(JsonRpcError::vm_status(vm_status)))
    } else if mempool_status.code == MempoolStatusCode::Accepted {
        Ok(())
    } else {
        Err(Error::new(JsonRpcError::mempool_error(mempool_status)?))
    }
}

/// Returns account state (AccountView) by given address
async fn get_account(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Option<AccountView>> {
    let account_address: AccountAddress = request.parse_account_address(0)?;

    let account_state = match service.get_account_state(account_address, request.version())? {
        Some(val) => val,
        None => return Ok(None),
    };
    let account_resource = account_state
        .get_account_resource()?
        .ok_or_else(|| format_err!("invalid account data: no account resource"))?;
    let freezing_bit = account_state
        .get_freezing_bit()?
        .ok_or_else(|| format_err!("invalid account data: no freezing bit"))?;

    let currency_info = get_currencies(service, request).await?;
    let currencies: Vec<_> = currency_info
        .into_iter()
        .map(|info| from_currency_code_string(&info.code))
        .collect::<Result<_, _>>()?;

    let account_role = account_state
        .get_account_role(&currencies)?
        .ok_or_else(|| format_err!("invalid account data: no account role"))?;

    let balances = account_state.get_balance_resources(&currencies)?;

    Ok(Some(AccountView::new(
        &account_address,
        &account_resource,
        balances,
        account_role,
        freezing_bit,
    )))
}

/// Returns the blockchain metadata for a specified version. If no version is specified, default to
/// returning the current blockchain metadata
/// Can be used to verify that target Full Node is up-to-date
async fn get_metadata(service: JsonRpcService, request: JsonRpcRequest) -> Result<MetadataView> {
    let chain_id = service.chain_id().id();
    let version = if !request.params.is_empty() {
        request.parse_version_param(0, "version")?
    } else {
        request.version()
    };

    let mut script_hash_allow_list: Option<Vec<BytesView>> = None;
    let mut module_publishing_allowed: Option<bool> = None;
    let mut diem_version: Option<u64> = None;
    let mut dual_attestation_limit: Option<u64> = None;
    if version == request.version() {
        if let Some(account) = service.get_account_state(diem_root_address(), version)? {
            if let Some(vm_publishing_option) = account.get_vm_publishing_option()? {
                script_hash_allow_list = Some(
                    vm_publishing_option
                        .script_allow_list
                        .iter()
                        .map(|v| BytesView::from(v.to_vec()))
                        .collect(),
                );

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
        accumulator_root_hash: service.db.get_accumulator_root_hash(version)?.into(),
        timestamp: service.db.get_block_timestamp(version)?,
        chain_id,
        script_hash_allow_list,
        module_publishing_allowed,
        diem_version,
        dual_attestation_limit,
    })
}

/// Returns transactions by range
async fn get_transactions(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Vec<TransactionView>> {
    let start_version: u64 = request.parse_param(0, "start_version")?;
    let limit: u64 = request.parse_param(1, "limit")?;
    let include_events: bool = request.parse_param(2, "include_events")?;

    service.validate_page_size_limit(limit as usize)?;

    if start_version > request.version() {
        return Ok(vec![]);
    }

    let txs =
        service
            .db
            .get_transactions(start_version, limit, request.version(), include_events)?;

    let mut result = vec![];

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
            hash: tx.hash().into(),
            bytes: bcs::to_bytes(&tx)?.into(),
            transaction: tx.into(),
            events,
            vm_status: info.status().into(),
            gas_used: info.gas_used(),
        });
    }
    Ok(result)
}

/// Returns transactions by range with proofs
async fn get_transactions_with_proofs(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Option<TransactionsWithProofsView>> {
    let start_version: u64 = request.parse_param(0, "start_version")?;
    let limit: u64 = request.parse_param(1, "limit")?;

    // Notice limit is a u16 normally, but some APIs require u64 below
    service.validate_page_size_limit(limit as usize)?;

    if start_version > request.version() {
        return Ok(None);
    }

    // We do not fetch events since they don't come with proofs.
    let txs = service
        .db
        .get_transactions(start_version, limit, request.version(), false)?;

    let mut blobs = vec![];
    for t in txs.transactions.iter() {
        let bv = bcs::to_bytes(t)?;
        blobs.push(BytesView::from(bv));
    }

    let (proofs, tx_info) = txs.proof.unpack();

    Ok(Some(TransactionsWithProofsView {
        serialized_transactions: blobs,
        proofs: TransactionsProofsView {
            ledger_info_to_transaction_infos_proof: BytesView::from(&bcs::to_bytes(&proofs)?),
            transaction_infos: BytesView::from(&bcs::to_bytes(&tx_info)?),
        },
    }))
}

/// Returns account transaction by account and sequence_number
async fn get_account_transaction(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Option<TransactionView>> {
    let account: AccountAddress = request.parse_account_address(0)?;
    let sequence: u64 = request.parse_param(1, "account sequence number")?;
    let include_events: bool = request.parse_param(2, "include_events")?;

    let tx = service
        .db
        .get_txn_by_account(account, sequence, request.version(), include_events)?;

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
            .map(|x| (tx_version, x).try_into())
            .collect::<Result<Vec<EventView>>>()?;

        Ok(Some(TransactionView {
            version: tx_version,
            hash: tx.transaction.hash().into(),
            bytes: bcs::to_bytes(&tx.transaction)?.into(),
            transaction: tx.transaction.into(),
            events,
            vm_status: tx.proof.transaction_info().status().into(),
            gas_used: tx.proof.transaction_info().gas_used(),
        }))
    } else {
        Ok(None)
    }
}

/// Returns events by given access path
async fn get_events(service: JsonRpcService, request: JsonRpcRequest) -> Result<Vec<EventView>> {
    let event_key = request.parse_event_key(0, "event key")?;

    let start: u64 = request.parse_param(1, "start")?;
    let limit: u64 = request.parse_param(2, "limit")?;

    service.validate_page_size_limit(limit as usize)?;

    let events_with_proof = service
        .db
        .get_events(&event_key, start, Order::Ascending, limit)?;

    let req_version = request.version();
    let events = events_with_proof
        .into_iter()
        .filter(|(version, _event)| version <= &req_version)
        .map(|event| event.try_into())
        .collect::<Result<Vec<EventView>>>()?;
    Ok(events)
}

/// Returns meta information about supported currencies
async fn get_currencies(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Vec<CurrencyInfoView>> {
    if let Some(account_state) =
        service.get_account_state(diem_root_address(), request.version())?
    {
        Ok(account_state
            .get_registered_currency_info_resources()?
            .iter()
            .map(|info| info.into())
            .collect())
    } else {
        Ok(vec![])
    }
}

/// Returns all account transactions
async fn get_account_transactions(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<Vec<TransactionView>> {
    let account = request.parse_account_address(0)?;
    let start: u64 = request.parse_param(1, "start")?;
    let limit: u64 = request.parse_param(2, "limit")?;
    let include_events: bool = request.parse_param(3, "include_events")?;

    service.validate_page_size_limit(limit as usize)?;

    let account_state = service
        .db
        .get_latest_account_state(account)?
        .ok_or_else(|| {
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
        let tx = service
            .db
            .get_txn_by_account(account, seq, request.version(), include_events)?
            .ok_or_else(|| format_err!("Can not find transaction for seq {}!", seq))?;

        let tx_version = tx.version;
        let events = if include_events {
            ensure!(
                tx.events.is_some(),
                "Storage layer didn't return events when requested!"
            );
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
            hash: tx.transaction.hash().into(),
            bytes: bcs::to_bytes(&tx.transaction)?.into(),
            transaction: tx.transaction.into(),
            events,
            vm_status: tx.proof.transaction_info().status().into(),
            gas_used: tx.proof.transaction_info().gas_used(),
        });
    }

    Ok(all_txs)
}

/// Returns proof of new state relative to version known to client
async fn get_state_proof(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<StateProofView> {
    let known_version: u64 = request.parse_version_param(0, "version")?;
    let proofs = service
        .db
        .get_state_proof_with_ledger_info(known_version, request.ledger_info.clone())?;
    StateProofView::try_from((request.ledger_info, proofs.0, proofs.1))
}

/// Returns the account state to the client, alongside a proof relative to the version and
/// ledger_version specified by the client. If version or ledger_version are not specified,
/// the latest known versions will be used.
async fn get_account_state_with_proof(
    service: JsonRpcService,
    request: JsonRpcRequest,
) -> Result<AccountStateWithProofView, JsonRpcError> {
    let account_address = request.parse_account_address(0)?;

    // If versions are specified by the request parameters, use them, otherwise use the defaults
    let version = request.parse_version_param(1, "version")?;
    let ledger_version = request.parse_version_param(2, "ledger version for proof")?;

    if version > ledger_version {
        return Err(JsonRpcError::invalid_request_with_msg(format!(
            "version({}) should <= ledger version({})",
            version, ledger_version
        )));
    }

    let account_state_with_proof =
        service
            .db
            .get_account_state_with_proof(account_address, version, ledger_version)?;
    Ok(AccountStateWithProofView::try_from(
        account_state_with_proof,
    )?)
}

/// Returns the number of peers this node is connected to
async fn get_network_status(service: JsonRpcService, _request: JsonRpcRequest) -> Result<u64> {
    let peers = counters::DIEM_NETWORK_PEERS
        .get_metric_with_label_values(&[service.role.as_str(), "connected"])?;
    Ok(peers.get() as u64)
}

/// Builds registry of all available RPC methods
/// To register new RPC method, add it via `register_rpc_method!` macros call
/// Note that RPC method name will equal to name of function
#[allow(unused_comparisons)]
pub(crate) fn build_registry() -> RpcRegistry {
    let mut registry = RpcRegistry::new();
    register_rpc_method!(registry, "submit", submit, 1, 0);
    register_rpc_method!(registry, "get_metadata", get_metadata, 0, 1);
    register_rpc_method!(registry, "get_account", get_account, 1, 0);
    register_rpc_method!(registry, "get_transactions", get_transactions, 3, 0);
    register_rpc_method!(
        registry,
        "get_account_transaction",
        get_account_transaction,
        3,
        0
    );
    register_rpc_method!(
        registry,
        "get_account_transactions",
        get_account_transactions,
        4,
        0
    );
    register_rpc_method!(registry, "get_events", get_events, 3, 0);
    register_rpc_method!(registry, "get_currencies", get_currencies, 0, 0);
    register_rpc_method!(registry, "get_network_status", get_network_status, 0, 0);

    // Proof APIs
    register_rpc_method!(registry, "get_state_proof", get_state_proof, 1, 0);
    register_rpc_method!(
        registry,
        "get_account_state_with_proof",
        get_account_state_with_proof,
        3,
        0
    );
    register_rpc_method!(
        registry,
        "get_transactions_with_proofs",
        get_transactions_with_proofs,
        2,
        0
    );

    registry
}

/// Returns invalid param JsonRpcError for the param[index] with name and type info
fn invalid_param(index: usize, name: &str) -> JsonRpcError {
    let type_info = match name {
        "start" => "unsigned int64",
        "start_version" => "unsigned int64",
        "limit" => "unsigned int64",
        "account sequence number" => "unsigned int64",
        "include_events" => "boolean",
        "account address" => "hex-encoded string",
        "event key" => "hex-encoded string",
        "data" => "hex-encoded string of BCS serialized Diem SignedTransaction type",
        "version" => "unsigned int64",
        "ledger version for proof" => "unsigned int64",
        _ => "unknown",
    };
    JsonRpcError::invalid_param(index, name, type_info)
}
