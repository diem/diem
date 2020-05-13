// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Admission Control (AC) is a module acting as the only public end point. It receives api requests
//! from external clients (such as wallets) and performs necessary processing before sending them to
//! next step.

use crate::counters;
use admission_control_proto::{
    proto::admission_control::{
        admission_control_server::{AdmissionControl, AdmissionControlServer},
        submit_transaction_response::Status,
        SubmitTransactionRequest, SubmitTransactionResponse,
    },
    AdmissionControlStatus,
};
use anyhow::Result;
use debug_interface::prelude::*;
use futures::{channel::oneshot, SinkExt};
use grpc_types::proto::types::{
    MempoolStatus as MempoolStatusProto, SignedTransaction as SignedTransactionProto,
    UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse, VmStatus as VmStatusProto,
};
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_mempool::MempoolClientSender;
use libra_types::{mempool_status::MempoolStatusCode, transaction::SignedTransaction};
use std::{convert::TryFrom, sync::Arc};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Runtime};

/// Struct implementing trait (service handle) AdmissionControlService.
#[derive(Clone)]
pub struct AdmissionControlService {
    ac_sender: MempoolClientSender,
    db: Arc<dyn DbReader>,
}

impl AdmissionControlService {
    /// Constructs a new AdmissionControlService instance.
    pub fn new(ac_sender: MempoolClientSender, db: Arc<dyn DbReader>) -> Self {
        AdmissionControlService { ac_sender, db }
    }

    /// Creates and spins up AdmissionControlService on runtime
    /// Returns the runtime on which Admission Control Service is newly spawned
    pub fn bootstrap(
        config: &NodeConfig,
        db: Arc<dyn DbReader>,
        ac_sender: MempoolClientSender,
    ) -> Runtime {
        let runtime = Builder::new()
            .thread_name("ac-service-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[admission control] failed to create runtime");

        let admission_control_service = AdmissionControlService::new(ac_sender, db);

        runtime.spawn(
            tonic::transport::Server::builder()
                .add_service(AdmissionControlServer::new(admission_control_service))
                .serve(config.admission_control.address),
        );
        runtime
    }
    /// Pass the UpdateToLatestLedgerRequest to Storage for read query.
    fn update_to_latest_ledger_inner(
        &self,
        req: UpdateToLatestLedgerRequest,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let rust_req = libra_types::get_with_proof::UpdateToLatestLedgerRequest::try_from(req)?;
        let (response_items, ledger_info_with_sigs, epoch_change_proof, ledger_consistency_proof) =
            self.db
                .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;
        let rust_resp = libra_types::get_with_proof::UpdateToLatestLedgerResponse::new(
            response_items,
            ledger_info_with_sigs,
            epoch_change_proof,
            ledger_consistency_proof,
        );
        Ok(rust_resp.into())
    }
}

#[tonic::async_trait]
impl AdmissionControl for AdmissionControlService {
    /// Submit a transaction to the validator this AC instance connecting to.
    /// The specific transaction will be first validated by VM and then passed
    /// to Mempool for further processing.
    async fn submit_transaction(
        &self,
        request: tonic::Request<SubmitTransactionRequest>,
    ) -> Result<tonic::Response<SubmitTransactionResponse>, tonic::Status> {
        trace!("[GRPC] AdmissionControl::submit_transaction");
        counters::REQUESTS
            .with_label_values(&["submit_transaction"])
            .inc();
        let txn_proto = request
            .into_inner()
            .transaction
            .unwrap_or_else(SignedTransactionProto::default);

        let txn = SignedTransaction::try_from(txn_proto).map_err(|e| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!(
                    "[admission-control] Submitting transaction failed with error: {:?}",
                    e
                ),
            )
        })?;

        trace_code_block!("admission_control_service::submit_transaction", {"txn", txn.sender(), txn.sequence_number()});
        let (req_sender, res_receiver) = oneshot::channel();
        self.ac_sender
            .clone()
            .send((txn, req_sender))
            .await
            .map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Internal,
                    format!(
                        "[admission-control] Failed to submit write request with error: {:?}",
                        e
                    ),
                )
            })?;

        let response = res_receiver
            .await
            .unwrap()
            .map(|(mempool_status, vm_status)| {
                let mut resp = SubmitTransactionResponse::default();
                if let Some(vm_error) = vm_status {
                    resp.status = Some(Status::VmStatus(VmStatusProto::from(vm_error)));
                } else if mempool_status.code == MempoolStatusCode::Accepted {
                    resp.status = Some(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
                } else {
                    resp.status = Some(Status::MempoolStatus(MempoolStatusProto::from(
                        mempool_status,
                    )));
                }
                resp
            })
            .map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Internal,
                    format!(
                        "[admission-control] Submitting transaction failed with error: {:?}",
                        e
                    ),
                )
            })?;
        Ok(tonic::Response::new(response))
    }

    /// This API is used to update the client to the latest ledger version and optionally also
    /// request 1..n other pieces of data.  This allows for batch queries.  All queries return
    /// proofs that a client should check to validate the data.
    /// Note that if a client only wishes to update to the latest LedgerInfo and receive the proof
    /// of this latest version, they can simply omit the requested_items (or pass an empty list).
    /// AC will not directly process this request but pass it to Storage instead.
    async fn update_to_latest_ledger(
        &self,
        request: tonic::Request<UpdateToLatestLedgerRequest>,
    ) -> Result<tonic::Response<UpdateToLatestLedgerResponse>, tonic::Status> {
        trace!("[GRPC] AdmissionControl::update_to_latest_ledger");
        counters::REQUESTS
            .with_label_values(&["update_to_latest_ledger"])
            .inc();
        let req = request.into_inner();
        let resp = self
            .update_to_latest_ledger_inner(req)
            .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, e.to_string()))?;
        Ok(tonic::Response::new(resp))
    }
}
