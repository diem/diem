// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements the storage [GRPC](http://grpc.io) service.
//!
//! The user of storage service is supposed to use it via client lib provided in
//! [`storage-client`](../storage-client/index.html) instead of via
//! [`StorageClient`](../storage-proto/proto/storage_grpc/struct.StorageClient.html) directly.

#[cfg(feature = "fuzzing")]
pub mod mocks;
mod storage_service;
pub use crate::storage_service::start_storage_service_and_return_service;

use anyhow::Result;
use grpc_helpers::{provide_grpc_response, spawn_service_thread_with_drop_closure, ServerHandle};
use libra_config::config::NodeConfig;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_metrics::counters::SVC_COUNTERS;
use libra_types::proto::types::{Transaction as TransactionProto, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use libradb::LibraDB;
use std::{
    convert::{TryFrom, From},
    ops::Deref,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};
use storage_proto::proto::storage::{
    create_storage, GetAccountStateWithProofByVersionRequest,
    GetAccountStateWithProofByVersionResponse, GetEpochChangeLedgerInfosRequest,
    GetEpochChangeLedgerInfosResponse, GetHistoryStartupInfoByBlockIdRequest,
    GetStartupInfoRequest, GetStartupInfoResponse, GetTransactionsRequest, GetTransactionsResponse,
    RollbackRequest, SaveTransactionsRequest, Version, LatestVersionResponse,
    SaveTransactionsResponse, Storage, GetTransactionListRequest, GetTransactionListResponse,
    GetTransactionByVersionResponse
};
/// Starts storage service according to config.
pub fn start_storage_service(config: &NodeConfig) -> ServerHandle {
    let (storage_service, shutdown_receiver) = StorageService::new(&config.storage.dir());
    spawn_service_thread_with_drop_closure(
        create_storage(storage_service),
        config.storage.address.clone(),
        config.storage.port,
        "storage",
        config.storage.grpc_max_receive_len,
        move || {
            shutdown_receiver
                .recv()
                .expect("Failed to receive on shutdown channel when storage service was dropped")
        },
    )
}

/// The implementation of the storage [GRPC](http://grpc.io) service.
///
/// It serves [`LibraDB`] APIs over the network. See API documentation in [`storage-proto`] and
/// [`LibraDB`].
#[derive(Clone)]
pub struct StorageService {
    db: Arc<LibraDBWrapper>,
}

/// When dropping GRPC server we want to wait until LibraDB is dropped first, so the RocksDB
/// instance held by GRPC threads is closed before the main function of GRPC server
/// finishes. Otherwise, if we don't manually guarantee this, some thread(s) may still be
/// alive holding an Arc pointer to LibraDB after main function of GRPC server returns.
/// Having this wrapper with a channel gives us a way to signal the receiving end that all GRPC
/// server threads are joined so RocksDB is closed.
///
/// See these links for more details.
///   https://github.com/pingcap/grpc-rs/issues/227
///   https://github.com/facebook/rocksdb/issues/649
struct LibraDBWrapper {
    db: Option<LibraDB>,
    shutdown_sender: Mutex<mpsc::Sender<()>>,
}

impl LibraDBWrapper {
    pub fn new<P: AsRef<Path>>(path: &P) -> (Self, mpsc::Receiver<()>) {
        let db = LibraDB::new(path);
        let (shutdown_sender, shutdown_receiver) = mpsc::channel();
        (
            Self {
                db: Some(db),
                shutdown_sender: Mutex::new(shutdown_sender),
            },
            shutdown_receiver,
        )
    }
}

impl Drop for LibraDBWrapper {
    fn drop(&mut self) {
        // Drop inner LibraDB instance.
        self.db.take();
        // Send the shutdown message after DB is dropped.
        self.shutdown_sender
            .lock()
            .expect("Failed to lock mutex.")
            .send(())
            .expect("Failed to send shutdown message.");
    }
}

impl Deref for LibraDBWrapper {
    type Target = LibraDB;

    fn deref(&self) -> &Self::Target {
        self.db.as_ref().expect("LibraDB is dropped unexptectedly")
    }
}

impl StorageService {
    /// This opens a [`LibraDB`] at `path` and returns a [`StorageService`] instance serving it.
    ///
    /// A receiver side of a channel is also returned through which one can receive a notice after
    /// all resources used by the service including the underlying [`LibraDB`] instance are
    /// fully dropped.
    ///
    /// example:
    /// ```no_run,
    ///    # use storage_service::*;
    ///    # use std::path::Path;
    ///    let (service, shutdown_receiver) = StorageService::new(&Path::new("path/to/db"));
    ///
    ///    drop(service);
    ///    shutdown_receiver.recv().expect("recv() should succeed.");
    ///
    ///    // LibraDB instance is guaranteed to be properly dropped at this point.
    /// ```
    pub fn new<P: AsRef<Path>>(path: &P) -> (Self, mpsc::Receiver<()>) {
        let (db_wrapper, shutdown_receiver) = LibraDBWrapper::new(path);
        (
            Self {
                db: Arc::new(db_wrapper),
            },
            shutdown_receiver,
        )
    }
}

impl StorageService {
    fn update_to_latest_ledger_inner(
        &self,
        req: UpdateToLatestLedgerRequest,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let rust_req = libra_types::get_with_proof::UpdateToLatestLedgerRequest::try_from(req)?;

        let (
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
            ledger_consistency_proof,
        ) = self
            .db
            .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;

        let rust_resp = libra_types::get_with_proof::UpdateToLatestLedgerResponse {
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
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
            &rust_req.ledger_info_with_signatures,
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
    ) -> Result<GetEpochChangeLedgerInfosResponse> {
        let rust_req = storage_proto::GetEpochChangeLedgerInfosRequest::try_from(req)?;
        let ledger_infos = self
            .db
            .get_epoch_change_ledger_infos(rust_req.start_epoch)?;
        let rust_resp = storage_proto::GetEpochChangeLedgerInfosResponse::new(ledger_infos);
        Ok(rust_resp.into())
    }

    fn get_history_startup_info_by_block_id_inner(
        &self,
        block_id: &HashValue,
    ) -> Result<GetStartupInfoResponse> {
        let info = self.db.get_history_startup_info_by_block_id(block_id)?;
        let rust_resp = storage_proto::GetStartupInfoResponse { info };
        Ok(rust_resp.into())
    }

    fn rollback_by_block_id_inner(&self, block_id: &HashValue) -> Result<()> {
        self.db.rollback_by_block_id(block_id)
    }
}

impl Storage for StorageService {
    fn save_transactions(
        &mut self,
        ctx: grpcio::RpcContext,
        req: SaveTransactionsRequest,
        sink: grpcio::UnarySink<SaveTransactionsResponse>,
    ) {
        debug!("[GRPC] Storage::save_transactions");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.save_transactions_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }

    fn update_to_latest_ledger(
        &mut self,
        ctx: grpcio::RpcContext<'_>,
        req: UpdateToLatestLedgerRequest,
        sink: grpcio::UnarySink<UpdateToLatestLedgerResponse>,
    ) {
        debug!("[GRPC] Storage::update_to_latest_ledger");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.update_to_latest_ledger_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }

    fn get_transactions(
        &mut self,
        ctx: grpcio::RpcContext,
        req: GetTransactionsRequest,
        sink: grpcio::UnarySink<GetTransactionsResponse>,
    ) {
        debug!("[GRPC] Storage::get_transactions");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.get_transactions_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }

    fn get_account_state_with_proof_by_version(
        &mut self,
        ctx: grpcio::RpcContext,
        req: GetAccountStateWithProofByVersionRequest,
        sink: grpcio::UnarySink<GetAccountStateWithProofByVersionResponse>,
    ) {
        debug!("[GRPC] Storage::get_account_state_with_proof_by_version");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.get_account_state_with_proof_by_version_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }

    fn get_startup_info(
        &mut self,
        ctx: grpcio::RpcContext,
        _req: GetStartupInfoRequest,
        sink: grpcio::UnarySink<GetStartupInfoResponse>,
    ) {
        debug!("[GRPC] Storage::get_startup_info");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.get_startup_info_inner();
        provide_grpc_response(resp, ctx, sink);
    }

    fn get_epoch_change_ledger_infos(
        &mut self,
        ctx: grpcio::RpcContext,
        req: GetEpochChangeLedgerInfosRequest,
        sink: grpcio::UnarySink<GetEpochChangeLedgerInfosResponse>,
    ) {
        debug!("[GRPC] Storage::get_epoch_change_ledger_infos");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.get_epoch_change_ledger_infos_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }

    fn get_history_startup_info_by_block_id(
        &mut self,
        ctx: grpcio::RpcContext,
        req: GetHistoryStartupInfoByBlockIdRequest,
        sink: grpcio::UnarySink<GetStartupInfoResponse>,
    ) {
        debug!("[GRPC] Storage::get_history_startup_info_by_block_id");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.get_history_startup_info_by_block_id_inner(
            &HashValue::from_slice(req.block_id.as_ref()).expect("parse err."),
        );
        provide_grpc_response(resp, ctx, sink);
    }

    fn rollback_by_block_id(
        &mut self,
        ctx: grpcio::RpcContext,
        req: RollbackRequest,
        sink: grpcio::UnarySink<()>,
    ) {
        debug!("[GRPC] Storage::rollback_by_block_id");
        self.rollback_by_block_id_inner(
            &HashValue::from_slice(req.block_id.as_ref()).expect("parse err."),
        )
        .expect("rollback err.");
        provide_grpc_response(Ok(()), ctx, sink);
    }

    fn latest_version(&mut self, ctx: ::grpcio::RpcContext,
                      _req: (), sink: ::grpcio::UnarySink<LatestVersionResponse>) {
        debug!("[GRPC] Storage::latest_version");
        let ver = self.db.get_latest_version();
        let mut resp = LatestVersionResponse::default();
        match ver {
            Ok(version) => {
                let mut v = Version::default();
                v.ver = version;
                resp.version = Some(v);
            },
            Err(e) => {
                warn!("{:?}", e);
            },
        }

        provide_grpc_response(Ok(resp), ctx, sink);
    }

    fn get_transaction_list(&mut self, ctx: ::grpcio::RpcContext, req: GetTransactionListRequest,
                            sink: ::grpcio::UnarySink<GetTransactionListResponse>) {
        debug!("[GRPC] Storage::get_transaction_list");
        let version = match req.version {
            Some(v) => {v.ver},
            None => {self.db.get_latest_version().expect("get latest version err.") + 1},
        };

        let end_version = if version > 0 {version - 1 } else {version};
        let start_version = if end_version > 10 {end_version - 10} else {0};
        let txn_vec = self.db.transactions(start_version, end_version);

        let mut resp = GetTransactionListResponse::default();
        match txn_vec {
            Ok(txns) => {
                let tmp = txns.iter().map(|txn| TransactionProto::from(txn.clone())).collect();
                resp.transactions = tmp;
            },
            Err(e) => {warn!("{:?}", e);}
        }

        provide_grpc_response(Ok(resp), ctx, sink);
    }

    fn get_transaction_by_version(&mut self, ctx: ::grpcio::RpcContext, req: Version,
                                  sink: ::grpcio::UnarySink<GetTransactionByVersionResponse>) {
        debug!("[GRPC] Storage::get_transaction_by_version");
        let mut resp = GetTransactionByVersionResponse::default();
        match self.db.get_transaction_by_version(req.ver) {
            Ok(txn) => {
                resp.txn = Some(TransactionProto::from(txn));
            },
            Err(e) => {warn!("{:?}", e);}
        }

        provide_grpc_response(Ok(resp), ctx, sink);
    }
}

#[cfg(test)]
mod storage_service_test;
