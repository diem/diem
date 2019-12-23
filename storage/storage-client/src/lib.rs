// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements a client library for storage that wraps the protobuf storage client. The
//! main motivation is to hide storage implementation details. For example, if we later want to
//! expand state store to multiple machines and enable sharding, we only need to tweak the client
//! library implementation and protobuf interface, and the interface between the rest of the system
//! and the client library will remain the same, so we won't need to change other components.

mod state_view;

use anyhow::{format_err, Error, Result};
use futures::{compat::Stream01CompatExt, prelude::*, stream::BoxStream};
use futures_01::stream::Stream as Stream01;
use grpc_helpers::convert_grpc_response;
use grpcio::{ChannelBuilder, Environment};
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
use rand::Rng;
use std::convert::TryFrom;
use std::sync::Arc;
use storage_proto::{
    proto::storage::{GetLatestStateRootRequest, GetStartupInfoRequest, StorageClient},
    BackupAccountStateRequest, BackupAccountStateResponse, GetAccountStateRangeProofRequest,
    GetAccountStateRangeProofResponse, GetAccountStateWithProofByVersionRequest,
    GetAccountStateWithProofByVersionResponse, GetEpochChangeLedgerInfosRequest,
    GetLatestAccountStateRequest, GetLatestAccountStateResponse, GetLatestStateRootResponse,
    GetStartupInfoResponse, GetTransactionsRequest, GetTransactionsResponse,
    SaveTransactionsRequest, StartupInfo,
};

pub use crate::state_view::VerifiedStateView;

fn pick<T>(items: &[T]) -> &T {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0, items.len());
    &items[index]
}

fn make_clients(
    env: Arc<Environment>,
    host: &str,
    port: u16,
    client_type: &str,
    max_receive_len: Option<i32>,
) -> Vec<StorageClient> {
    let num_clients = env.completion_queues().len();
    (0..num_clients)
        .map(|i| {
            let mut builder = ChannelBuilder::new(env.clone())
                .primary_user_agent(format!("grpc/storage-{}-{}", client_type, i).as_str());
            if let Some(m) = max_receive_len {
                builder = builder.max_receive_message_len(m);
            }
            let channel = builder.connect(&format!("{}:{}", host, port));
            StorageClient::new(channel)
        })
        .collect::<Vec<StorageClient>>()
}

fn convert_grpc_stream<T>(
    stream: impl Stream01<Item = T, Error = grpcio::Error>,
) -> impl Stream<Item = Result<T, Error>> {
    stream.map_err(convert_grpc_err).compat()
}

/// This provides storage read interfaces backed by real storage service.
#[derive(Clone)]
pub struct StorageReadServiceClient {
    clients: Vec<StorageClient>,
}

impl StorageReadServiceClient {
    /// Constructs a `StorageReadServiceClient` with given host and port.
    pub fn new(env: Arc<Environment>, host: &str, port: u16) -> Self {
        let clients = make_clients(env, host, port, "read", None);
        StorageReadServiceClient { clients }
    }

    fn client(&self) -> &StorageClient {
        pick(&self.clients)
    }
}

#[async_trait::async_trait]
impl StorageRead for StorageReadServiceClient {
    async fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        let req = UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        };
        let resp =
            convert_grpc_response(self.client().update_to_latest_ledger_async(&req.into())).await?;
        let rust_resp = UpdateToLatestLedgerResponse::try_from(resp)?;
        Ok((
            rust_resp.response_items,
            rust_resp.ledger_info_with_sigs,
            rust_resp.validator_change_proof,
            rust_resp.ledger_consistency_proof,
        ))
    }

    async fn get_transactions_async(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        let req =
            GetTransactionsRequest::new(start_version, batch_size, ledger_version, fetch_events);
        let resp = convert_grpc_response(self.client().get_transactions_async(&req.into())).await?;
        let rust_resp = GetTransactionsResponse::try_from(resp)?;
        Ok(rust_resp.txn_list_with_proof)
    }

    async fn get_latest_state_root_async(&self) -> Result<(Version, HashValue)> {
        let req = GetLatestStateRootRequest::default();
        let resp = convert_grpc_response(self.client().get_latest_state_root_async(&req)).await?;
        let rust_resp = GetLatestStateRootResponse::try_from(resp)?;
        Ok(rust_resp.into())
    }

    async fn get_latest_account_state_async(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        let req = GetLatestAccountStateRequest::new(address);
        let resp = convert_grpc_response(self.client().get_latest_account_state_async(&req.into()))
            .await?;
        let rust_resp = GetLatestAccountStateResponse::try_from(resp)?;
        Ok(rust_resp.account_state_blob)
    }

    async fn get_account_state_with_proof_by_version_async(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        let req = GetAccountStateWithProofByVersionRequest::new(address, version);
        let resp = convert_grpc_response(
            self.client()
                .get_account_state_with_proof_by_version_async(&req.into()),
        )
        .await?;
        let resp = GetAccountStateWithProofByVersionResponse::try_from(resp)?;
        Ok(resp.into())
    }

    async fn get_startup_info_async(&self) -> Result<Option<StartupInfo>> {
        let proto_req = GetStartupInfoRequest::default();
        let resp = convert_grpc_response(self.client().get_startup_info_async(&proto_req)).await?;
        let resp = GetStartupInfoResponse::try_from(resp)?;
        Ok(resp.info)
    }

    async fn get_epoch_change_ledger_infos_async(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        let proto_req = GetEpochChangeLedgerInfosRequest::new(start_epoch, end_epoch);
        let resp = convert_grpc_response(
            self.client()
                .get_epoch_change_ledger_infos_async(&proto_req.into()),
        )
        .await?;
        let resp = ValidatorChangeProof::try_from(resp)?;
        Ok(resp)
    }

    fn backup_account_state(
        &self,
        version: Version,
    ) -> Result<BoxStream<'_, Result<BackupAccountStateResponse, Error>>> {
        let proto_req = BackupAccountStateRequest::new(version);
        Ok(
            convert_grpc_stream(self.client().backup_account_state(&proto_req.into())?)
                .map(|resp| {
                    let resp = BackupAccountStateResponse::try_from(resp?)?;
                    Ok(resp)
                })
                .boxed(),
        )
    }

    async fn get_account_state_range_proof(
        &self,
        rightmost_key: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        let req = GetAccountStateRangeProofRequest::new(rightmost_key, version);
        let resp = convert_grpc_response(
            self.client()
                .get_account_state_range_proof_async(&req.into()),
        )
        .await?;
        Ok(GetAccountStateRangeProofResponse::try_from(resp)?.into())
    }
}

/// This provides storage write interfaces backed by real storage service.
#[derive(Clone)]
pub struct StorageWriteServiceClient {
    clients: Vec<StorageClient>,
}

impl StorageWriteServiceClient {
    /// Constructs a `StorageWriteServiceClient` with given host and port.
    pub fn new(
        env: Arc<Environment>,
        host: &str,
        port: u16,
        grpc_max_receive_len: Option<i32>,
    ) -> Self {
        let clients = make_clients(env, host, port, "write", grpc_max_receive_len);
        StorageWriteServiceClient { clients }
    }

    fn client(&self) -> &StorageClient {
        pick(&self.clients)
    }
}

#[async_trait::async_trait]
impl StorageWrite for StorageWriteServiceClient {
    async fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let req =
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs);
        convert_grpc_response(self.client().save_transactions_async(&req.into())).await?;
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
    async fn update_to_latest_ledger_async(
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
    async fn get_transactions_async(
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
    async fn get_latest_state_root_async(&self) -> Result<(Version, HashValue)>;

    /// See [`LibraDB::get_latest_account_state`].
    ///
    /// [`LibraDB::get_latest_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_account_state
    async fn get_latest_account_state_async(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>>;

    /// See [`LibraDB::get_account_state_with_proof_by_version`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_version`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_version
    async fn get_account_state_with_proof_by_version_async(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    async fn get_startup_info_async(&self) -> Result<Option<StartupInfo>>;

    /// See [`LibraDB::get_epoch_change_ledger_infos`].
    ///
    /// [`LibraDB::get_epoch_change_ledger_infos`]:
    /// ../libradb/struct.LibraDB.html#method.get_epoch_change_ledger_infos
    async fn get_epoch_change_ledger_infos_async(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof>;

    /// See [`LibraDB::backup_account_state`].
    ///
    /// [`LibraDB::backup_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.backup_account_state
    fn backup_account_state(
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
    async fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()>;
}

fn convert_grpc_err(e: grpcio::Error) -> Error {
    format_err!("grpc error: {}", e)
}
