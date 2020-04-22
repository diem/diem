// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements the storage [GRPC](http://grpc.io) service.
//!
//! The user of storage service is supposed to use it via client lib provided in
//! [`storage-client`](../storage-client/index.html) instead of via
//! [`StorageClient`](../storage-proto/proto/storage_grpc/struct.StorageClient.html) directly.

#[cfg(any(feature = "testing", feature = "fuzzing"))]
pub mod mocks;

use anyhow::Result;
use futures::{channel::mpsc, sink::SinkExt};
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_types::proto::types::{
    EpochChangeProof, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
};
use libradb::LibraDB;
use std::{convert::TryFrom, path::Path, sync::Arc};
use storage_interface::{DbReader, DbReaderWriter, DbWriter, Error, StartupInfo};
use storage_proto::proto::storage::{
    storage_server::{Storage, StorageServer},
    BackupAccountStateRequest, BackupAccountStateResponse, BackupTransactionInfoRequest,
    BackupTransactionInfoResponse, BackupTransactionRequest, BackupTransactionResponse,
    GetAccountStateRangeProofRequest, GetAccountStateRangeProofResponse,
    GetAccountStateWithProofByVersionRequest, GetAccountStateWithProofByVersionResponse,
    GetEpochChangeLedgerInfosRequest, GetLatestAccountStateRequest, GetLatestAccountStateResponse,
    GetLatestStateRootRequest, GetLatestStateRootResponse, GetStartupInfoRequest,
    GetStartupInfoResponse, GetTransactionsRequest, GetTransactionsResponse,
    SaveTransactionsRequest, SaveTransactionsResponse,
};
use tokio::runtime::Runtime;

use libra_secure_net::NetworkServer;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleProof};
use std::thread::{self, JoinHandle};

/// Starts storage service with a given LibraDB
pub fn start_simple_storage_service_with_db(
    config: &NodeConfig,
    libra_db: Arc<LibraDB>,
) -> JoinHandle<()> {
    let storage_service = SimpleStorageService { db: libra_db };
    storage_service.run(config)
}

/// Starts storage service according to config.
pub fn start_simple_storage_service(config: &NodeConfig) -> JoinHandle<()> {
    let storage_service = SimpleStorageService::new(&config.storage.dir());
    storage_service.run(config)
}

#[derive(Clone)]
pub struct SimpleStorageService {
    db: Arc<LibraDB>,
}

impl SimpleStorageService {
    /// This opens a [`LibraDB`] at `path` and returns a [`StorageService`] instance serving it.
    pub fn new(path: &impl AsRef<Path>) -> Self {
        let db = Arc::new(LibraDB::new(path));
        Self { db }
    }

    fn handle_message(&self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;
        let output = match input {
            storage_interface::StorageRequest::GetAccountStateWithProofByVersionRequest(req) => {
                lcs::to_bytes(&self.get_account_state_with_proof_by_version(&req))
            }
            storage_interface::StorageRequest::GetStartupInfoRequest => {
                lcs::to_bytes(&self.get_startup_info())
            }
            storage_interface::StorageRequest::SaveTransactionsRequest(req) => {
                lcs::to_bytes(&self.save_transactions(&req))
            }
        };
        Ok(output?)
    }

    fn get_account_state_with_proof_by_version(
        &self,
        req: &storage_interface::GetAccountStateWithProofByVersionRequest,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof), Error> {
        Ok(self
            .db
            .get_account_state_with_proof_by_version(req.address, req.version)?)
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>, Error> {
        Ok(self.db.get_startup_info()?)
    }

    fn save_transactions(
        &self,
        req: &storage_interface::SaveTransactionsRequest,
    ) -> Result<(), Error> {
        Ok(self.db.save_transactions(
            &req.txns_to_commit,
            req.first_version,
            req.ledger_info_with_signatures.as_ref(),
        )?)
    }

    fn run(self, config: &NodeConfig) -> JoinHandle<()> {
        let mut network_server = NetworkServer::new(config.storage.address);
        thread::spawn(move || loop {
            if let Err(e) = self.process_one_message(&mut network_server) {
                warn!("Failed to process message: {}", e);
            }
        })
    }

    fn process_one_message(&self, network_server: &mut NetworkServer) -> Result<(), Error> {
        let request = network_server.read()?;
        let response = self.handle_message(request)?;
        network_server.write(&response)?;
        Ok(())
    }
}

pub fn init_libra_db(config: &NodeConfig) -> (Arc<LibraDB>, DbReaderWriter) {
    DbReaderWriter::wrap(LibraDB::new(&config.storage.dir()))
}

/// Starts storage service with a given LibraDB
pub fn start_storage_service_with_db(config: &NodeConfig, libra_db: Arc<LibraDB>) -> Runtime {
    let storage_service = StorageService { db: libra_db };
    start_storage_service_runtime(config, storage_service)
}

/// Starts storage service according to config.
pub fn start_storage_service(config: &NodeConfig) -> Runtime {
    let storage_service = StorageService::new(&config.storage.dir());
    start_storage_service_runtime(config, storage_service)
}

fn start_storage_service_runtime(config: &NodeConfig, storage_service: StorageService) -> Runtime {
    let mut rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .thread_name("tokio-storage")
        .build()
        .unwrap();

    rt.spawn(
        tonic::transport::Server::builder()
            .add_service(StorageServer::new(storage_service))
            .serve(config.storage.address),
    );

    let addr = format!("http://{}", config.storage.address);
    for _i in 0..100 {
        if rt
            .block_on(
                storage_proto::proto::storage::storage_client::StorageClient::connect(addr.clone()),
            )
            .is_ok()
        {
            return rt;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    panic!("Failed to start storage service.");
}

/// The implementation of the storage [GRPC](http://grpc.io) service.
///
/// It serves [`LibraDB`] APIs over the network. See API documentation in [`storage-proto`] and
/// [`LibraDB`].
#[derive(Clone)]
pub struct StorageService {
    db: Arc<LibraDB>,
}

impl StorageService {
    /// This opens a [`LibraDB`] at `path` and returns a [`StorageService`] instance serving it.
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        let db = Arc::new(LibraDB::new(path));
        Self { db }
    }
}

impl StorageService {
    fn update_to_latest_ledger_inner(
        &self,
        req: UpdateToLatestLedgerRequest,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let rust_req = libra_types::get_with_proof::UpdateToLatestLedgerRequest::try_from(req)?;

        let (response_items, ledger_info_with_sigs, epoch_change_proof, ledger_consistency_proof) =
            self.db
                .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;

        let rust_resp = libra_types::get_with_proof::UpdateToLatestLedgerResponse {
            response_items,
            ledger_info_with_sigs,
            epoch_change_proof,
            ledger_consistency_proof,
        };

        Ok(rust_resp.into())
    }

    fn get_transactions_inner(
        &self,
        req: GetTransactionsRequest,
    ) -> Result<GetTransactionsResponse> {
        let rust_req = storage_proto::GetTransactionsRequest::try_from(req)?;

        let txn_list_with_proof = self.db.get_transactions(
            rust_req.start_version,
            rust_req.batch_size,
            rust_req.ledger_version,
            rust_req.fetch_events,
        )?;

        let rust_resp = storage_proto::GetTransactionsResponse::new(txn_list_with_proof);

        Ok(rust_resp.into())
    }

    fn get_latest_state_root_inner(
        &self,
        _req: GetLatestStateRootRequest,
    ) -> Result<GetLatestStateRootResponse> {
        let (version, state_root_hash) = self.db.get_latest_state_root()?;
        let rust_resp = storage_proto::GetLatestStateRootResponse::new(version, state_root_hash);
        Ok(rust_resp.into())
    }

    fn get_latest_account_state_inner(
        &self,
        req: GetLatestAccountStateRequest,
    ) -> Result<GetLatestAccountStateResponse> {
        let rust_req = storage_proto::GetLatestAccountStateRequest::try_from(req)?;
        let account_state_blob = self.db.get_latest_account_state(rust_req.address)?;
        let rust_resp = storage_proto::GetLatestAccountStateResponse::new(account_state_blob);
        Ok(rust_resp.into())
    }

    fn get_account_state_with_proof_by_version_inner(
        &self,
        req: GetAccountStateWithProofByVersionRequest,
    ) -> Result<GetAccountStateWithProofByVersionResponse> {
        let rust_req = storage_proto::GetAccountStateWithProofByVersionRequest::try_from(req)?;

        let (account_state_blob, sparse_merkle_proof) = self
            .db
            .get_account_state_with_proof_by_version(rust_req.address, rust_req.version)?;

        let rust_resp = storage_proto::GetAccountStateWithProofByVersionResponse {
            account_state_blob,
            sparse_merkle_proof,
        };

        Ok(rust_resp.into())
    }

    fn save_transactions_inner(
        &self,
        req: SaveTransactionsRequest,
    ) -> Result<SaveTransactionsResponse> {
        let rust_req = storage_proto::SaveTransactionsRequest::try_from(req)?;
        self.db.save_transactions(
            &rust_req.txns_to_commit,
            rust_req.first_version,
            rust_req.ledger_info_with_signatures.as_ref(),
        )?;
        Ok(SaveTransactionsResponse::default())
    }

    fn get_startup_info_inner(&self) -> Result<GetStartupInfoResponse> {
        let info = self.db.get_startup_info()?;
        let rust_resp = storage_proto::GetStartupInfoResponse { info };
        Ok(rust_resp.into())
    }

    fn get_epoch_change_ledger_infos_inner(
        &self,
        req: GetEpochChangeLedgerInfosRequest,
    ) -> Result<EpochChangeProof> {
        let rust_req = storage_proto::GetEpochChangeLedgerInfosRequest::try_from(req)?;
        let (ledger_infos, more) = self
            .db
            .get_epoch_change_ledger_infos(rust_req.start_epoch, rust_req.end_epoch)?;
        let rust_resp = libra_types::epoch_change::EpochChangeProof::new(ledger_infos, more);
        Ok(rust_resp.into())
    }

    fn get_account_state_range_proof_inner(
        &self,
        req: GetAccountStateRangeProofRequest,
    ) -> Result<GetAccountStateRangeProofResponse> {
        let rust_req = storage_proto::GetAccountStateRangeProofRequest::try_from(req)?;
        let proof = self
            .db
            .get_backup_handler()
            .get_account_state_range_proof(rust_req.rightmost_key, rust_req.version)?;
        let rust_resp = storage_proto::GetAccountStateRangeProofResponse::new(proof);
        Ok(rust_resp.into())
    }
}

#[tonic::async_trait]
impl Storage for StorageService {
    async fn save_transactions(
        &self,
        request: tonic::Request<SaveTransactionsRequest>,
    ) -> Result<tonic::Response<SaveTransactionsResponse>, tonic::Status> {
        trace!("[GRPC] Storage::save_transactions");
        let req = request.into_inner();
        let resp = self
            .save_transactions_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn update_to_latest_ledger(
        &self,
        request: tonic::Request<UpdateToLatestLedgerRequest>,
    ) -> Result<tonic::Response<UpdateToLatestLedgerResponse>, tonic::Status> {
        trace!("[GRPC] Storage::update_to_latest_ledger");
        let req = request.into_inner();
        let resp = self
            .update_to_latest_ledger_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_transactions(
        &self,
        request: tonic::Request<GetTransactionsRequest>,
    ) -> Result<tonic::Response<GetTransactionsResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_transactions");
        let req = request.into_inner();
        let resp = self
            .get_transactions_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_latest_state_root(
        &self,
        request: tonic::Request<GetLatestStateRootRequest>,
    ) -> Result<tonic::Response<GetLatestStateRootResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_latest_state_root");
        let req = request.into_inner();
        let resp = self
            .get_latest_state_root_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_latest_account_state(
        &self,
        request: tonic::Request<GetLatestAccountStateRequest>,
    ) -> Result<tonic::Response<GetLatestAccountStateResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_latest_account_state");
        let req = request.into_inner();
        let resp = self
            .get_latest_account_state_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_account_state_with_proof_by_version(
        &self,
        request: tonic::Request<GetAccountStateWithProofByVersionRequest>,
    ) -> Result<tonic::Response<GetAccountStateWithProofByVersionResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_account_state_with_proof_by_version");
        let req = request.into_inner();
        let resp = self
            .get_account_state_with_proof_by_version_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_startup_info(
        &self,
        _request: tonic::Request<GetStartupInfoRequest>,
    ) -> Result<tonic::Response<GetStartupInfoResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_startup_info");
        let resp = self
            .get_startup_info_inner()
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    async fn get_epoch_change_ledger_infos(
        &self,
        request: tonic::Request<GetEpochChangeLedgerInfosRequest>,
    ) -> Result<tonic::Response<EpochChangeProof>, tonic::Status> {
        trace!("[GRPC] Storage::get_epoch_change_ledger_infos");
        let req = request.into_inner();
        let resp = self
            .get_epoch_change_ledger_infos_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    type BackupAccountStateStream =
        mpsc::Receiver<Result<BackupAccountStateResponse, tonic::Status>>;

    async fn backup_account_state(
        &self,
        request: tonic::Request<BackupAccountStateRequest>,
    ) -> Result<tonic::Response<Self::BackupAccountStateStream>, tonic::Status> {
        trace!("[GRPC] Storage::backup_account_state");
        let req = request.into_inner();
        let iter = self
            .db
            .get_backup_handler()
            .get_account_iter(req.version)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;

        let iter = iter.map(|res| match res {
            Ok((hash, blob)) => {
                let resp: BackupAccountStateResponse =
                    storage_proto::BackupAccountStateResponse::new(hash, blob).into();
                Ok(resp)
            }
            Err(e) => Err(tonic::Status::new(tonic::Code::Internal, e.to_string())),
        });

        // Channel to buffer the stream
        let (mut tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            for resp in iter {
                tx.send(resp).await.unwrap();
            }
        });

        Ok(tonic::Response::new(rx))
    }

    async fn get_account_state_range_proof(
        &self,
        request: tonic::Request<GetAccountStateRangeProofRequest>,
    ) -> Result<tonic::Response<GetAccountStateRangeProofResponse>, tonic::Status> {
        trace!("[GRPC] Storage::get_account_state_range_proof");
        let req = request.into_inner();
        let resp = self
            .get_account_state_range_proof_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }

    type BackupTransactionStream = mpsc::Receiver<Result<BackupTransactionResponse, tonic::Status>>;

    async fn backup_transaction(
        &self,
        request: tonic::Request<BackupTransactionRequest>,
    ) -> Result<tonic::Response<Self::BackupTransactionStream>, tonic::Status> {
        trace!("[GRPC] Storage::backup_transaction");
        let req = request.into_inner();

        let backup_handler = self.db.get_backup_handler();
        let (mut tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let iter = match backup_handler
                .get_transaction_iter(req.start_version, req.num_transactions)
            {
                Ok(iter) => iter,
                Err(e) => {
                    tx.send(Err(tonic::Status::new(
                        tonic::Code::Internal,
                        e.to_string(),
                    )))
                    .await
                    .unwrap();
                    return;
                }
            };

            let iter = iter.map(|res| match res {
                Ok(transaction) => {
                    let resp: BackupTransactionResponse =
                        storage_proto::BackupTransactionResponse { transaction }.into();
                    Ok(resp)
                }
                Err(e) => Err(tonic::Status::new(tonic::Code::Internal, e.to_string())),
            });

            for resp in iter {
                tx.send(resp).await.unwrap();
            }
        });

        Ok(tonic::Response::new(rx))
    }

    type BackupTransactionInfoStream =
        mpsc::Receiver<Result<BackupTransactionInfoResponse, tonic::Status>>;

    async fn backup_transaction_info(
        &self,
        request: tonic::Request<BackupTransactionInfoRequest>,
    ) -> Result<tonic::Response<Self::BackupTransactionInfoStream>, tonic::Status> {
        trace!("[GRPC] Storage::backup_transaction_info");
        let req = request.into_inner();

        let backup_handler = self.db.get_backup_handler();
        let (mut tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let iter = match backup_handler
                .get_transaction_info_iter(req.start_version, req.num_transactions)
            {
                Ok(iter) => iter,
                Err(e) => {
                    tx.send(Err(tonic::Status::new(
                        tonic::Code::Internal,
                        e.to_string(),
                    )))
                    .await
                    .unwrap();
                    return;
                }
            };

            let iter = iter.map(|res| match res {
                Ok(transaction_info) => {
                    let resp: BackupTransactionInfoResponse =
                        storage_proto::BackupTransactionInfoResponse { transaction_info }.into();
                    Ok(resp)
                }
                Err(e) => Err(tonic::Status::new(tonic::Code::Internal, e.to_string())),
            });

            for resp in iter {
                tx.send(resp).await.unwrap();
            }
        });

        Ok(tonic::Response::new(rx))
    }
}

#[cfg(test)]
mod storage_service_test;
