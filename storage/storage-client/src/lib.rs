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
use futures::{compat::Future01CompatExt, executor::block_on, prelude::*};
use futures_01::future::Future as Future01;
use grpcio::{ChannelBuilder, Environment};
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeEventWithProof},
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    proof::AccumulatorConsistencyProof,
    proof::SparseMerkleProof,
    transaction::{TransactionListWithProof, TransactionToCommit, Version},
    explorer::{
        LatestVersionResponse, GetTransactionListRequest,
        GetTransactionListResponse, GetTransactionByVersionResponse
    }
};
use rand::Rng;
use std::convert::TryFrom;
use std::{pin::Pin, sync::Arc};
use storage_proto::{
    proto::storage::{GetStartupInfoRequest, StorageClient},
    GetAccountStateWithProofByVersionRequest,
    GetAccountStateWithProofByVersionResponse, GetEpochChangeLedgerInfosRequest,
    GetEpochChangeLedgerInfosResponse, GetHistoryStartupInfoByBlockIdRequest,
    GetStartupInfoResponse, GetTransactionsRequest, GetTransactionsResponse,
    RollbackRequest, SaveTransactionsRequest, StartupInfo,
};

pub use crate::state_view::VerifiedStateView;
use libra_crypto::HashValue;

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

fn convert_grpc_response<T>(
    response: grpcio::Result<impl Future01<Item = T, Error = grpcio::Error>>,
) -> impl Future<Output = Result<T>> {
    future::ready(response.map_err(convert_grpc_err))
        .map_ok(Future01CompatExt::compat)
        .and_then(|x| x.map_err(convert_grpc_err))
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

impl StorageRead for StorageReadServiceClient {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeEventWithProof,
        AccumulatorConsistencyProof,
    )> {
        block_on(self.update_to_latest_ledger_async(client_known_version, requested_items))
    }

    fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(
                        Vec<ResponseItem>,
                        LedgerInfoWithSignatures,
                        ValidatorChangeEventWithProof,
                        AccumulatorConsistencyProof,
                    )>,
                > + Send,
        >,
    > {
        let req = UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        };
        convert_grpc_response(self.client().update_to_latest_ledger_async(&req.into()))
            .map(|resp| {
                let rust_resp = UpdateToLatestLedgerResponse::try_from(resp?)?;
                Ok((
                    rust_resp.response_items,
                    rust_resp.ledger_info_with_sigs,
                    rust_resp.validator_change_events,
                    rust_resp.ledger_consistency_proof,
                ))
            })
            .boxed()
    }

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        block_on(self.get_transactions_async(
            start_version,
            batch_size,
            ledger_version,
            fetch_events,
        ))
    }

    fn get_transactions_async(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
        let req =
            GetTransactionsRequest::new(start_version, batch_size, ledger_version, fetch_events);
        convert_grpc_response(self.client().get_transactions_async(&req.into()))
            .map(|resp| {
                let rust_resp = GetTransactionsResponse::try_from(resp?)?;
                Ok(rust_resp.txn_list_with_proof)
            })
            .boxed()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        block_on(self.get_account_state_with_proof_by_version_async(address, version))
    }

    fn get_account_state_with_proof_by_version_async(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>
    {
        let req = GetAccountStateWithProofByVersionRequest::new(address, version);
        convert_grpc_response(
            self.client()
                .get_account_state_with_proof_by_version_async(&req.into()),
        )
        .map(|resp| {
            let resp = GetAccountStateWithProofByVersionResponse::try_from(resp?)?;
            Ok(resp.into())
        })
        .boxed()
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        block_on(self.get_startup_info_async())
    }

    fn get_history_startup_info_by_block_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<StartupInfo>> {
        let req = GetHistoryStartupInfoByBlockIdRequest { block_id };
        let startup_info_resp = self
            .client()
            .get_history_startup_info_by_block_id(&req.into());
        let resp = GetStartupInfoResponse::try_from(startup_info_resp?)?;
        Ok(resp.info)
    }

    fn get_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StartupInfo>>> + Send>> {
        let proto_req = GetStartupInfoRequest::default();
        convert_grpc_response(self.client().get_startup_info_async(&proto_req))
            .map(|resp| {
                let resp = GetStartupInfoResponse::try_from(resp?)?;
                Ok(resp.info)
            })
            .boxed()
    }

    fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures>> {
        block_on(self.get_epoch_change_ledger_infos_async(start_epoch))
    }

    fn get_epoch_change_ledger_infos_async(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LedgerInfoWithSignatures>>> + Send>> {
        let proto_req = GetEpochChangeLedgerInfosRequest::new(start_epoch);
        convert_grpc_response(
            self.client()
                .get_epoch_change_ledger_infos_async(&proto_req.into()),
        )
        .map(|resp| {
            let resp = GetEpochChangeLedgerInfosResponse::try_from(resp?)?;
            Ok(resp.into())
        })
        .boxed()
    }

    fn latest_version(&mut self) -> Result<LatestVersionResponse> {
        unimplemented!()
    }

    fn get_transaction_list(&mut self, req: GetTransactionListRequest)
                            -> Result<GetTransactionListResponse>{
        unimplemented!()
    }

    fn get_transaction_by_version(&mut self, req: Version)
                                  -> Result<GetTransactionByVersionResponse>{
        unimplemented!()
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

impl StorageWrite for StorageWriteServiceClient {
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        block_on(self.save_transactions_async(txns_to_commit, first_version, ledger_info_with_sigs))
    }

    fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let req =
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs);
        convert_grpc_response(self.client().save_transactions_async(&req.into()))
            .map_ok(|_| ())
            .boxed()
    }

    fn rollback_by_block_id(&self, block_id: HashValue) {
        let req = RollbackRequest { block_id };
        self.client()
            .rollback_by_block_id(&req.into())
            .expect("rollback err.");
    }
}

/// This trait defines interfaces to be implemented by a storage read client.
///
/// There is a 1-1 mapping between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
pub trait StorageRead: Send + Sync {
    /// See [`LibraDB::update_to_latest_ledger`].
    ///
    /// [`LibraDB::update_to_latest_ledger`]:
    /// ../libradb/struct.LibraDB.html#method.update_to_latest_ledger
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeEventWithProof,
        AccumulatorConsistencyProof,
    )>;

    /// See [`LibraDB::update_to_latest_ledger`].
    ///
    /// [`LibraDB::update_to_latest_ledger`]:../libradb/struct.LibraDB.html#method.
    /// update_to_latest_ledger
    fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(
                        Vec<ResponseItem>,
                        LedgerInfoWithSignatures,
                        ValidatorChangeEventWithProof,
                        AccumulatorConsistencyProof,
                    )>,
                > + Send,
        >,
    >;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions_async(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>>;

    /// See [`LibraDB::get_account_state_with_proof_by_version`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_version`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_version
    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// See [`LibraDB::get_account_state_with_proof_by_version`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_version`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_version
    fn get_account_state_with_proof_by_version_async(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>;

    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    /// history startup info
    fn get_history_startup_info_by_block_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<StartupInfo>>;

    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    fn get_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StartupInfo>>> + Send>>;

    /// See [`LibraDB::get_epoch_change_ledger_infos`].
    ///
    /// [`LibraDB::get_epoch_change_ledger_infos`]:
    /// ../libradb/struct.LibraDB.html#method.get_epoch_change_ledger_infos
    fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures>>;

    /// See [`LibraDB::get_epoch_change_ledger_infos`].
    ///
    /// [`LibraDB::get_epoch_change_ledger_infos`]:
    /// ../libradb/struct.LibraDB.html#method.get_epoch_change_ledger_infos
    fn get_epoch_change_ledger_infos_async(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LedgerInfoWithSignatures>>> + Send>>;

    fn latest_version(&mut self) -> Result<LatestVersionResponse>;

    fn get_transaction_list(&mut self, req: GetTransactionListRequest)
        -> Result<GetTransactionListResponse>;

    fn get_transaction_by_version(&mut self, req: Version) -> Result<GetTransactionByVersionResponse>;
}

/// This trait defines interfaces to be implemented by a storage write client.
///
/// There is a 1-1 mappings between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
pub trait StorageWrite: Send + Sync {
    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()>;

    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Rollback
    fn rollback_by_block_id(&self, block_id: HashValue);
}

fn convert_grpc_err(e: grpcio::Error) -> Error {
    format_err!("grpc error: {}", e)
}
