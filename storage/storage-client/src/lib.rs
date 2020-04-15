// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements a client library for storage that wraps the protobuf storage client. The
//! main motivation is to hide storage implementation details. For example, if we later want to
//! expand state store to multiple machines and enable sharding, we only need to tweak the client
//! library implementation and protobuf interface, and the interface between the rest of the system
//! and the client library will remain the same, so we won't need to change other components.

use anyhow::{format_err, Error, Result};
use futures::{
    executor::block_on,
    stream::{BoxStream, StreamExt},
};
use libra_crypto::HashValue;
use libra_secure_net::NetworkClient;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    event::EventKey,
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccumulatorConsistencyProof, SparseMerkleProof, SparseMerkleRangeProof},
    transaction::{TransactionListWithProof, TransactionToCommit, TransactionWithProof, Version},
    validator_change::ValidatorChangeProof,
};
use serde::de::DeserializeOwned;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use storage_interface::{DbReader, DbWriter};
use storage_proto::{
    proto::storage::{
        storage_client::StorageClient, GetLatestStateRootRequest, GetStartupInfoRequest,
    },
    BackupAccountStateRequest, BackupAccountStateResponse, BackupTransactionInfoRequest,
    BackupTransactionInfoResponse, BackupTransactionRequest, BackupTransactionResponse,
    GetAccountStateRangeProofRequest, GetAccountStateRangeProofResponse,
    GetAccountStateWithProofByVersionRequest, GetAccountStateWithProofByVersionResponse,
    GetEpochChangeLedgerInfosRequest, GetLatestAccountStateRequest, GetLatestAccountStateResponse,
    GetLatestStateRootResponse, GetStartupInfoResponse, GetTransactionsRequest,
    GetTransactionsResponse, SaveTransactionsRequest, StartupInfo,
};
use tokio::runtime::Runtime;

pub struct SimpleStorageClient {
    network_client: Mutex<NetworkClient>,
}

impl SimpleStorageClient {
    pub fn new(server_address: &SocketAddr) -> Self {
        Self {
            network_client: Mutex::new(NetworkClient::new(*server_address)),
        }
    }

    fn request<T: DeserializeOwned>(
        &self,
        input: storage_interface::StorageRequest,
    ) -> Result<T, storage_interface::Error> {
        let input_message = lcs::to_bytes(&input)?;
        let mut client = self.network_client.lock().unwrap();
        client.write(&input_message)?;
        let result = client.read()?;
        lcs::from_bytes(&result)?
    }

    pub fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof), storage_interface::Error> {
        self.request(
            storage_interface::StorageRequest::GetAccountStateWithProofByVersionRequest(Box::new(
                storage_interface::GetAccountStateWithProofByVersionRequest::new(address, version),
            )),
        )
    }

    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>, storage_interface::Error> {
        self.request(storage_interface::StorageRequest::GetStartupInfoRequest)
    }

    pub fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), storage_interface::Error> {
        self.request(storage_interface::StorageRequest::SaveTransactionsRequest(
            Box::new(storage_interface::SaveTransactionsRequest::new(
                txns_to_commit,
                first_version,
                ledger_info_with_sigs,
            )),
        ))
    }
}

/// This provides storage read interfaces backed by real storage service.
pub struct StorageReadServiceClient {
    http_addr: String,
    client: Mutex<Option<StorageClient<tonic::transport::Channel>>>,
}

impl StorageReadServiceClient {
    /// Constructs a `StorageReadServiceClient` with given SocketAddr.
    pub fn new(address: &SocketAddr) -> Self {
        let http_addr = format!("http://{}", address.to_string());

        Self {
            client: Mutex::new(None),
            http_addr,
        }
    }

    async fn client(&self) -> Result<StorageClient<tonic::transport::Channel>, tonic::Status> {
        if self.client.lock().unwrap().is_none() {
            let client = StorageClient::connect(self.http_addr.clone())
                .await
                .map_err(|e| tonic::Status::new(tonic::Code::Unavailable, e.to_string()))?;
            *self.client.lock().unwrap() = Some(client);
        }

        // client is guaranteed to be populated by the time we reach here
        Ok(self.client.lock().unwrap().clone().unwrap())
    }
}

#[async_trait::async_trait]
impl StorageRead for StorageReadServiceClient {
    async fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        let req: libra_types::proto::types::UpdateToLatestLedgerRequest =
            UpdateToLatestLedgerRequest {
                client_known_version,
                requested_items,
            }
            .into();
        let resp = self
            .client()
            .await?
            .update_to_latest_ledger(req)
            .await?
            .into_inner();
        let rust_resp = UpdateToLatestLedgerResponse::try_from(resp)?;
        Ok((
            rust_resp.response_items,
            rust_resp.ledger_info_with_sigs,
            rust_resp.validator_change_proof,
            rust_resp.ledger_consistency_proof,
        ))
    }

    async fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        let req: storage_proto::proto::storage::GetTransactionsRequest =
            GetTransactionsRequest::new(start_version, batch_size, ledger_version, fetch_events)
                .into();
        let resp = self
            .client()
            .await?
            .get_transactions(req)
            .await?
            .into_inner();
        let rust_resp = GetTransactionsResponse::try_from(resp)?;
        Ok(rust_resp.txn_list_with_proof)
    }

    async fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let req = GetLatestStateRootRequest::default();
        let resp = self
            .client()
            .await?
            .get_latest_state_root(req)
            .await?
            .into_inner();
        let rust_resp = GetLatestStateRootResponse::try_from(resp)?;
        Ok(rust_resp.into())
    }

    async fn get_latest_account_state(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        let req: storage_proto::proto::storage::GetLatestAccountStateRequest =
            GetLatestAccountStateRequest::new(address).into();
        let resp = self
            .client()
            .await?
            .get_latest_account_state(req)
            .await?
            .into_inner();
        let rust_resp = GetLatestAccountStateResponse::try_from(resp)?;
        Ok(rust_resp.account_state_blob)
    }

    async fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        let req: storage_proto::proto::storage::GetAccountStateWithProofByVersionRequest =
            GetAccountStateWithProofByVersionRequest::new(address, version).into();
        let resp = self
            .client()
            .await?
            .get_account_state_with_proof_by_version(req)
            .await?
            .into_inner();
        let resp = GetAccountStateWithProofByVersionResponse::try_from(resp)?;
        Ok(resp.into())
    }

    async fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        let proto_req = GetStartupInfoRequest::default();
        let resp = self
            .client()
            .await?
            .get_startup_info(proto_req)
            .await?
            .into_inner();
        let resp = GetStartupInfoResponse::try_from(resp)?;
        Ok(resp.info)
    }

    async fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        let proto_req: storage_proto::proto::storage::GetEpochChangeLedgerInfosRequest =
            GetEpochChangeLedgerInfosRequest::new(start_epoch, end_epoch).into();
        let resp = self
            .client()
            .await?
            .get_epoch_change_ledger_infos(proto_req)
            .await?
            .into_inner();
        let resp = ValidatorChangeProof::try_from(resp)?;
        Ok(resp)
    }

    async fn backup_account_state(
        &self,
        version: Version,
    ) -> Result<BoxStream<'_, Result<BackupAccountStateResponse, Error>>> {
        let proto_req: storage_proto::proto::storage::BackupAccountStateRequest =
            BackupAccountStateRequest::new(version).into();
        let stream = self
            .client()
            .await?
            .backup_account_state(proto_req)
            .await?
            .into_inner()
            .map(|resp| {
                let resp = BackupAccountStateResponse::try_from(resp?)?;
                Ok(resp)
            })
            .boxed();
        Ok(stream)
    }

    async fn get_account_state_range_proof(
        &self,
        rightmost_key: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        let req: storage_proto::proto::storage::GetAccountStateRangeProofRequest =
            GetAccountStateRangeProofRequest::new(rightmost_key, version).into();
        let resp = self
            .client()
            .await?
            .get_account_state_range_proof(req)
            .await?
            .into_inner();

        Ok(GetAccountStateRangeProofResponse::try_from(resp)?.into())
    }

    async fn backup_transaction(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<BoxStream<'_, Result<BackupTransactionResponse, Error>>> {
        let proto_req: storage_proto::proto::storage::BackupTransactionRequest =
            BackupTransactionRequest::new(start_version, num_transactions).into();
        let stream = self
            .client()
            .await?
            .backup_transaction(proto_req)
            .await?
            .into_inner()
            .map(|resp| {
                let resp = BackupTransactionResponse::try_from(resp?)?;
                Ok(resp)
            })
            .boxed();
        Ok(stream)
    }

    async fn backup_transaction_info(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<BoxStream<'_, Result<BackupTransactionInfoResponse, Error>>> {
        let proto_req: storage_proto::proto::storage::BackupTransactionInfoRequest =
            BackupTransactionInfoRequest::new(start_version, num_transactions).into();
        let stream = self
            .client()
            .await?
            .backup_transaction_info(proto_req)
            .await?
            .into_inner()
            .map(|resp| {
                let resp = BackupTransactionInfoResponse::try_from(resp?)?;
                Ok(resp)
            })
            .boxed();
        Ok(stream)
    }

    // TODO migrate this to `ConfigStorage` trait as non-async
    // and implement for DbReader once `StorageRead` is deprecated
    async fn batch_fetch_config(&self, access_paths: Vec<AccessPath>) -> Result<Vec<Vec<u8>>> {
        // some access paths can have the same address, so don't duplicate requests for them
        let addresses: Vec<AccountAddress> = access_paths
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|path| path.address)
            .collect();

        let requests: Vec<RequestItem> = addresses
            .iter()
            .map(|addr| RequestItem::GetAccountState { address: *addr })
            .collect();

        let responses = self.update_to_latest_ledger(0, requests).await?.0;

        // Account address --> AccountState
        let account_states = addresses
            .into_iter()
            .zip(responses)
            .map(|(addr, resp)| {
                let account_state = AccountState::try_from(
                    &resp
                        .into_get_account_state_response()?
                        .blob
                        .ok_or_else(|| {
                            format_err!("missing blob in account state/account does not exist")
                        })?,
                )?;
                Ok((addr, account_state))
            })
            .collect::<Result<HashMap<_, AccountState>>>()?;

        access_paths
            .into_iter()
            .map(|path| {
                Ok(account_states
                    .get(&path.address)
                    .ok_or_else(|| format_err!("missing account state for queried access path"))?
                    .get(&path.path)
                    .ok_or_else(|| format_err!("no value found in account state"))?
                    .clone())
            })
            .collect()
    }
}

/// This provides storage write interfaces backed by real storage service.
pub struct StorageWriteServiceClient {
    http_addr: String,
    client: Mutex<Option<StorageClient<tonic::transport::Channel>>>,
}

impl StorageWriteServiceClient {
    /// Constructs a `StorageWriteServiceClient` with given SocketAddr.
    pub fn new(address: &SocketAddr) -> Self {
        let http_addr = format!("http://{}", address.to_string());

        Self {
            client: Mutex::new(None),
            http_addr,
        }
    }

    async fn client(&self) -> Result<StorageClient<tonic::transport::Channel>, tonic::Status> {
        if self.client.lock().unwrap().is_none() {
            let client = StorageClient::connect(self.http_addr.clone())
                .await
                .map_err(|e| tonic::Status::new(tonic::Code::Unavailable, e.to_string()))?;
            *self.client.lock().unwrap() = Some(client);
        }

        // client is guaranteed to be populated by the time we reach here
        Ok(self.client.lock().unwrap().clone().unwrap())
    }
}

#[async_trait::async_trait]
impl StorageWrite for StorageWriteServiceClient {
    async fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let req: storage_proto::proto::storage::SaveTransactionsRequest =
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs)
                .into();
        self.client().await?.save_transactions(req).await?;
        Ok(())
    }
}

/// This trait defines interfaces to be implemented by a storage read client.
///
/// There is a 1-1 mapping between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
#[async_trait::async_trait]
pub trait StorageRead: Send + Sync {
    /// See [`LibraDB::update_to_latest_ledger`].
    ///
    /// [`LibraDB::update_to_latest_ledger`]:../libradb/struct.LibraDB.html#method.
    /// update_to_latest_ledger
    async fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )>;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    async fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// See [`LibraDB::get_latest_state_root`].
    ///
    /// [`LibraDB::get_latest_state_root`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_state_root
    async fn get_latest_state_root(&self) -> Result<(Version, HashValue)>;

    /// See [`LibraDB::get_latest_account_state`].
    ///
    /// [`LibraDB::get_latest_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_account_state
    async fn get_latest_account_state(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>>;

    /// See [`LibraDB::get_account_state_with_proof_by_version`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_version`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_version
    async fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    async fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    /// See [`LibraDB::get_epoch_change_ledger_infos`].
    ///
    /// [`LibraDB::get_epoch_change_ledger_infos`]:
    /// ../libradb/struct.LibraDB.html#method.get_epoch_change_ledger_infos
    async fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof>;

    /// See [`LibraDB::backup_account_state`].
    ///
    /// [`LibraDB::backup_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.backup_account_state
    async fn backup_account_state(
        &self,
        version: u64,
    ) -> Result<BoxStream<'_, Result<BackupAccountStateResponse, Error>>>;

    /// See [`LibraDB::get_account_state_range_proof`].
    ///
    /// [`LibraDB::get_account_state_range_proof`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_range_proof
    async fn get_account_state_range_proof(
        &self,
        rightmost_key: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof>;

    /// Gets a stream of transactions (for backup purpose).
    async fn backup_transaction(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<BoxStream<'_, Result<BackupTransactionResponse, Error>>>;

    /// Gets a stream of transaction infos (for backup purpose).
    async fn backup_transaction_info(
        &self,
        start_version: Version,
        num_transaction_infos: u64,
    ) -> Result<BoxStream<'_, Result<BackupTransactionInfoResponse, Error>>>;

    /// Returns a vector of on-chain configs as serialized byte array
    /// Order of on-chain configs returned matches the order of `access_path`
    async fn batch_fetch_config(&self, access_paths: Vec<AccessPath>) -> Result<Vec<Vec<u8>>>;
}

/// This trait defines interfaces to be implemented by a storage write client.
///
/// There is a 1-1 mappings between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
#[async_trait::async_trait]
pub trait StorageWrite: Send + Sync {
    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    async fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()>;
}

pub struct SyncStorageClient {
    reader: Arc<StorageReadServiceClient>,
    writer: Arc<StorageWriteServiceClient>,
    rt: Runtime,
}

impl SyncStorageClient {
    pub fn new(address: &SocketAddr) -> Self {
        let rt = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .thread_name("tokio-executor")
            .build()
            .unwrap();
        Self {
            reader: Arc::new(StorageReadServiceClient::new(address)),
            writer: Arc::new(StorageWriteServiceClient::new(address)),
            rt,
        }
    }
}

impl DbReader for SyncStorageClient {
    fn get_transactions(
        &self,
        _start_version: u64,
        _batch_size: u64,
        _ledger_version: u64,
        _fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        unimplemented!()
    }

    fn get_events(
        &self,
        _event_key: &EventKey,
        _start: u64,
        _ascending: bool,
        _limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        unimplemented!()
    }

    fn get_latest_account_state(
        &self,
        _address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        unimplemented!()
    }

    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        let reader = Arc::clone(&self.reader);
        block_on(
            self.rt
                .spawn(async move { reader.get_startup_info().await }),
        )?
    }

    fn get_txn_by_account(
        &self,
        _address: AccountAddress,
        _seq_num: u64,
        _ledger_version: u64,
        _fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>> {
        unimplemented!()
    }

    fn get_state_proof_with_ledger_info(
        &self,
        _known_version: u64,
        _ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(ValidatorChangeProof, AccumulatorConsistencyProof)> {
        unimplemented!()
    }

    fn get_state_proof(
        &self,
        _known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        unimplemented!()
    }

    fn get_account_state_with_proof(
        &self,
        _address: AccountAddress,
        _version: u64,
        _ledger_version: u64,
    ) -> Result<AccountStateWithProof> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: u64,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        let reader = Arc::clone(&self.reader);
        block_on(self.rt.spawn(async move {
            reader
                .get_account_state_with_proof_by_version(address, version)
                .await
        }))?
    }

    fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let reader = Arc::clone(&self.reader);
        block_on(
            self.rt
                .spawn(async move { reader.get_latest_state_root().await }),
        )?
    }
}

impl DbWriter for SyncStorageClient {
    fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: u64,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let writer = Arc::clone(&self.writer);
        let txns_to_commit_clone = txns_to_commit.to_vec();
        let ledger_info_with_sigs_clone = ledger_info_with_sigs.cloned();
        block_on(self.rt.spawn(async move {
            writer
                .save_transactions(
                    txns_to_commit_clone,
                    first_version,
                    ledger_info_with_sigs_clone,
                )
                .await
        }))?
    }
}
