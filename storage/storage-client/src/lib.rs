// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements a client library for storage that wraps the protobuf storage client. The
//! main motivation is to hide storage implementation details. For example, if we later want to
//! expand state store to multiple machines and enable sharding, we only need to tweak the client
//! library implementation and protobuf interface, and the interface between the rest of the system
//! and the client library will remain the same, so we won't need to change other components.

mod state_view;

pub use crate::state_view::VerifiedStateView;
use anyhow::{Error, Result};
use futures::stream::{BoxStream, StreamExt};
use libra_crypto::HashValue;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeProof},
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    proof::{AccumulatorConsistencyProof, SparseMerkleProof, SparseMerkleRangeProof},
    transaction::{TransactionListWithProof, TransactionToCommit, Version},
};
use std::convert::TryFrom;
use std::sync::Mutex;
use storage_proto::{
    proto::storage::{
        storage_client::StorageClient, GetLatestStateRootRequest, GetStartupInfoRequest,
    },
    BackupAccountStateRequest, BackupAccountStateResponse, GetAccountStateRangeProofRequest,
    GetAccountStateRangeProofResponse, GetAccountStateWithProofByVersionRequest,
    GetAccountStateWithProofByVersionResponse, GetEpochChangeLedgerInfosRequest,
    GetLatestAccountStateRequest, GetLatestAccountStateResponse, GetLatestStateRootResponse,
    GetStartupInfoResponse, GetTransactionsRequest, GetTransactionsResponse,
    SaveTransactionsRequest, StartupInfo,
};

/// This provides storage read interfaces backed by real storage service.
pub struct StorageReadServiceClient {
    addr: String,
    client: Mutex<Option<StorageClient<tonic::transport::Channel>>>,
}

impl StorageReadServiceClient {
    /// Constructs a `StorageReadServiceClient` with given host and port.
    pub fn new(host: &str, port: u16) -> Self {
        let addr = format!("http://{}:{}", host, port);

        Self {
            client: Mutex::new(None),
            addr,
        }
    }

    async fn client(&self) -> Result<StorageClient<tonic::transport::Channel>, tonic::Status> {
        if self.client.lock().unwrap().is_none() {
            let client = StorageClient::connect(self.addr.clone())
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
}

/// This provides storage write interfaces backed by real storage service.
pub struct StorageWriteServiceClient {
    addr: String,
    client: Mutex<Option<StorageClient<tonic::transport::Channel>>>,
}

impl StorageWriteServiceClient {
    /// Constructs a `StorageWriteServiceClient` with given host and port.
    pub fn new(host: &str, port: u16) -> Self {
        let addr = format!("http://{}:{}", host, port);

        Self {
            client: Mutex::new(None),
            addr,
        }
    }

    async fn client(&self) -> Result<StorageClient<tonic::transport::Channel>, tonic::Status> {
        if self.client.lock().unwrap().is_none() {
            let client = StorageClient::connect(self.addr.clone())
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
