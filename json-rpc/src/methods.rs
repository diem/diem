// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module contains RPC method handlers for Full Node JSON-RPC interface
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
use diem_config::config::RoleType;
use diem_crypto::{hash::CryptoHash, HashValue};
use diem_json_rpc_types::request::{
    GetAccountParams, GetAccountStateWithProofParams, GetAccountTransactionParams,
    GetAccountTransactionsParams, GetCurrenciesParams, GetEventsParams, GetEventsWithProofsParams,
    GetMetadataParams, GetNetworkStatusParams, GetStateProofParams, GetTransactionsParams,
    GetTransactionsWithProofsParams, MethodRequest, SubmitParams,
};
use diem_mempool::{MempoolClientSender, SubmissionStatus};
use diem_types::{
    account_address::AccountAddress,
    account_config::{
        diem_root_address, from_currency_code_string, resources::dual_attestation::Limit,
        AccountResource,
    },
    account_state::AccountState,
    chain_id::ChainId,
    ledger_info::LedgerInfoWithSignatures,
    mempool_status::MempoolStatusCode,
    transaction::SignedTransaction,
};
use fail::fail_point;
use futures::{channel::oneshot, SinkExt};
use network::counters;
use serde_json::Value;
use std::{
    cmp::min,
    convert::{TryFrom, TryInto},
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

    pub async fn mempool_request(
        &self,
        transaction: SignedTransaction,
    ) -> Result<SubmissionStatus> {
        let (req_sender, callback) = oneshot::channel();

        self.mempool_sender
            .clone()
            .send((transaction, req_sender))
            .await?;

        callback.await?
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

pub(crate) struct Handler<'a> {
    service: &'a JsonRpcService,
    ledger_info: &'a LedgerInfoWithSignatures,
}

impl<'a> Handler<'a> {
    pub fn new(service: &'a JsonRpcService, ledger_info: &'a LedgerInfoWithSignatures) -> Self {
        Self {
            service,
            ledger_info,
        }
    }

    fn version(&self) -> u64 {
        self.ledger_info.ledger_info().version()
    }

    fn version_param(&self, version: Option<u64>, name: &str) -> Result<u64, JsonRpcError> {
        let version = if let Some(version) = version {
            if version > self.version() {
                return Err(JsonRpcError::invalid_param(&format!(
                    "{} should be <= known latest version {}",
                    name,
                    self.version()
                )));
            }
            version
        } else {
            self.version()
        };
        Ok(version)
    }

    pub async fn handle(&self, method_request: MethodRequest) -> Result<Value, JsonRpcError> {
        let response: Value = match method_request {
            MethodRequest::Submit(params) => self.submit(params).await?.into(),
            MethodRequest::GetMetadata(params) => {
                serde_json::to_value(self.get_metadata(params).await?)?
            }
            MethodRequest::GetAccount(params) => {
                serde_json::to_value(self.get_account(params).await?)?
            }
            MethodRequest::GetTransactions(params) => {
                serde_json::to_value(self.get_transactions(params).await?)?
            }
            MethodRequest::GetAccountTransaction(params) => {
                serde_json::to_value(self.get_account_transaction(params).await?)?
            }
            MethodRequest::GetAccountTransactions(params) => {
                serde_json::to_value(self.get_account_transactions(params).await?)?
            }
            MethodRequest::GetEvents(params) => {
                serde_json::to_value(self.get_events(params).await?)?
            }
            MethodRequest::GetCurrencies(params) => {
                serde_json::to_value(self.get_currencies(params).await?)?
            }
            MethodRequest::GetNetworkStatus(params) => {
                serde_json::to_value(self.get_network_status(params).await?)?
            }
            MethodRequest::GetStateProof(params) => {
                serde_json::to_value(self.get_state_proof(params).await?)?
            }
            MethodRequest::GetAccountStateWithProof(params) => {
                serde_json::to_value(self.get_account_state_with_proof(params).await?)?
            }
            MethodRequest::GetTransactionsWithProofs(params) => {
                serde_json::to_value(self.get_transactions_with_proofs(params).await?)?
            }
            MethodRequest::GetEventsWithProofs(params) => {
                serde_json::to_value(self.get_events_with_proofs(params).await?)?
            }
        };
        Ok(response)
    }

    async fn submit(&self, params: SubmitParams) -> Result<(), JsonRpcError> {
        let (mempool_status, vm_status_opt) = self.service.mempool_request(params.data).await?;

        if let Some(vm_status) = vm_status_opt {
            Err(JsonRpcError::vm_status(vm_status))
        } else if mempool_status.code == MempoolStatusCode::Accepted {
            Ok(())
        } else {
            Err(JsonRpcError::mempool_error(mempool_status)?)
        }
    }

    /// Returns the blockchain metadata for a specified version. If no version is specified, default to
    /// returning the current blockchain metadata
    /// Can be used to verify that target Full Node is up-to-date
    async fn get_metadata(&self, params: GetMetadataParams) -> Result<MetadataView, JsonRpcError> {
        let chain_id = self.service.chain_id().id();
        let version = self.version_param(params.version, "version")?;

        let mut script_hash_allow_list: Option<Vec<HashValue>> = None;
        let mut module_publishing_allowed: Option<bool> = None;
        let mut diem_version: Option<u64> = None;
        let mut dual_attestation_limit: Option<u64> = None;
        if version == self.version() {
            if let Some(account) = self
                .service
                .get_account_state(diem_root_address(), version)?
            {
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
            accumulator_root_hash: self.service.db.get_accumulator_root_hash(version)?,
            timestamp: self.service.db.get_block_timestamp(version)?,
            chain_id,
            script_hash_allow_list,
            module_publishing_allowed,
            diem_version,
            dual_attestation_limit,
        })
    }

    /// Returns account state (AccountView) by given address
    async fn get_account(
        &self,
        params: GetAccountParams,
    ) -> Result<Option<AccountView>, JsonRpcError> {
        let account_address = params.account;
        let version = self.version_param(params.version, "version")?;

        let account_state = match self.service.get_account_state(account_address, version)? {
            Some(val) => val,
            None => return Ok(None),
        };
        let account_resource = account_state
            .get_account_resource()?
            .ok_or_else(|| format_err!("invalid account data: no account resource"))?;
        let freezing_bit = account_state
            .get_freezing_bit()?
            .ok_or_else(|| format_err!("invalid account data: no freezing bit"))?;

        let currency_info = self.get_currencies(GetCurrenciesParams).await?;
        let currencies: Vec<_> = currency_info
            .into_iter()
            .map(|info| from_currency_code_string(&info.code))
            .collect::<Result<_, _>>()?;

        let account_role = account_state
            .get_account_role(&currencies)?
            .ok_or_else(|| format_err!("invalid account data: no account role"))?;

        let balances = account_state.get_balance_resources(&currencies)?;

        Ok(Some(AccountView::new(
            account_address,
            &account_resource,
            balances,
            account_role,
            freezing_bit,
            version,
        )))
    }

    /// Returns transactions by range
    async fn get_transactions(
        &self,
        params: GetTransactionsParams,
    ) -> Result<Vec<TransactionView>, JsonRpcError> {
        let GetTransactionsParams {
            start_version,
            limit,
            include_events,
        } = params;

        self.service.validate_page_size_limit(limit as usize)?;

        if start_version > self.version() {
            return Ok(vec![]);
        }

        let txs = self.service.db.get_transactions(
            start_version,
            limit,
            self.version(),
            include_events,
        )?;

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
    async fn get_transactions_with_proofs(
        &self,
        params: GetTransactionsWithProofsParams,
    ) -> Result<Option<TransactionsWithProofsView>, JsonRpcError> {
        let GetTransactionsWithProofsParams {
            start_version,
            limit,
        } = params;

        let req_version = self.version();

        // Notice limit is a u16 normally, but some APIs require u64 below
        self.service.validate_page_size_limit(limit as usize)?;

        if start_version > req_version {
            return Ok(None);
        }

        // We do not fetch events since they don't come with proofs.
        let txs = self
            .service
            .db
            .get_transactions(start_version, limit, req_version, false)?;

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
    async fn get_account_transaction(
        &self,
        params: GetAccountTransactionParams,
    ) -> Result<Option<TransactionView>, JsonRpcError> {
        let GetAccountTransactionParams {
            account,
            sequence_number,
            include_events,
        } = params;

        let tx = self.service.db.get_txn_by_account(
            account,
            sequence_number,
            self.version(),
            include_events,
        )?;

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
    async fn get_account_transactions(
        &self,
        params: GetAccountTransactionsParams,
    ) -> Result<Vec<TransactionView>, JsonRpcError> {
        let GetAccountTransactionsParams {
            account,
            start,
            limit,
            include_events,
        } = params;

        self.service.validate_page_size_limit(limit as usize)?;

        let account_state = self
            .service
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
            let tx = self
                .service
                .db
                .get_txn_by_account(account, seq, self.version(), include_events)?
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
    async fn get_events(&self, params: GetEventsParams) -> Result<Vec<EventView>, JsonRpcError> {
        let GetEventsParams { key, start, limit } = params;

        self.service.validate_page_size_limit(limit as usize)?;

        let events_raw = self
            .service
            .db
            .get_events(&key, start, Order::Ascending, limit)?;

        let req_version = self.version();
        let events = events_raw
            .into_iter()
            .filter(|(version, _event)| version <= &req_version)
            .map(|event| event.try_into())
            .collect::<Result<Vec<EventView>>>()?;
        Ok(events)
    }

    /// Returns events by given access path along with their proofs
    async fn get_events_with_proofs(
        &self,
        params: GetEventsWithProofsParams,
    ) -> Result<Vec<EventWithProofView>, JsonRpcError> {
        let GetEventsWithProofsParams { key, start, limit } = params;

        self.service.validate_page_size_limit(limit as usize)?;

        let req_version = self.version();

        let events_with_proofs = self.service.db.get_events_with_proofs(
            &key,
            start,
            Order::Ascending,
            limit,
            Some(req_version),
        )?;

        let mut results = vec![];

        for event in events_with_proofs
            .into_iter()
            .filter(|e| e.transaction_version <= req_version)
        {
            results.push(EventWithProofView {
                event_with_proof: bcs::to_bytes(&event)?.into(),
            });
        }

        Ok(results)
    }

    /// Returns meta information about supported currencies
    async fn get_currencies(
        &self,
        _params: GetCurrenciesParams,
    ) -> Result<Vec<CurrencyInfoView>, JsonRpcError> {
        if let Some(account_state) = self
            .service
            .get_account_state(diem_root_address(), self.version())?
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

    /// Returns the number of peers this node is connected to
    async fn get_network_status(
        &self,
        _params: GetNetworkStatusParams,
    ) -> Result<u64, JsonRpcError> {
        let peers = counters::DIEM_NETWORK_PEERS
            .get_metric_with_label_values(&[self.service.role.as_str(), "connected"])
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
        Ok(peers.get() as u64)
    }

    /// Returns proof of new state relative to version known to client
    async fn get_state_proof(
        &self,
        params: GetStateProofParams,
    ) -> Result<StateProofView, JsonRpcError> {
        let version = self.version_param(Some(params.version), "version")?;
        let proofs = self
            .service
            .db
            .get_state_proof_with_ledger_info(version, self.ledger_info.clone())?;
        StateProofView::try_from((self.ledger_info.clone(), proofs.0, proofs.1)).map_err(Into::into)
    }

    /// Returns the account state to the client, alongside a proof relative to the version and
    /// ledger_version specified by the client. If version or ledger_version are not specified,
    /// the latest known versions will be used.
    async fn get_account_state_with_proof(
        &self,
        params: GetAccountStateWithProofParams,
    ) -> Result<AccountStateWithProofView, JsonRpcError> {
        let account_address = params.account;
        // If versions are specified by the request parameters, use them, otherwise use the defaults
        let version = self.version_param(params.version, "version")?;
        let ledger_version = self.version_param(params.ledger_version, "ledger_version")?;

        if version > ledger_version {
            return Err(JsonRpcError::invalid_request_with_msg(format!(
                "version({}) should <= ledger version({})",
                version, ledger_version
            )));
        }

        let account_state_with_proof = self.service.db.get_account_state_with_proof(
            account_address,
            version,
            ledger_version,
        )?;
        Ok(AccountStateWithProofView::try_from(
            account_state_with_proof,
        )?)
    }
}
