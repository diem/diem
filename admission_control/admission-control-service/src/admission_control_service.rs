// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Admission Control (AC) is a module acting as the only public end point. It receives api requests
//! from external clients (such as wallets) and performs necessary processing before sending them to
//! next step.

use crate::OP_COUNTERS;
use admission_control_proto::{
    proto::admission_control::{
        submit_transaction_response::Status, AdmissionControl, SubmitTransactionRequest,
        SubmitTransactionResponse,
    },
    AdmissionControlStatus,
};
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    executor::block_on,
};
use futures_01::future::Future;
use grpc_helpers::provide_grpc_response;
use libra_logger::prelude::*;
use libra_mempool::proto::{
    mempool::{AddTransactionWithValidationRequest, HealthCheckRequest},
    mempool_client::MempoolClientTrait,
};
use libra_mempool_shared_proto::proto::mempool_status::{
    MempoolAddTransactionStatus,
    MempoolAddTransactionStatusCode::{self, MempoolIsFull},
};
use libra_types::{
    proto::types::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
    transaction::SignedTransaction,
};
use metrics::counters::SVC_COUNTERS;
use std::convert::TryFrom;
use std::sync::Arc;
use storage_client::StorageRead;
use vm_validator::vm_validator::{get_account_state, TransactionValidation};

#[cfg(test)]
#[path = "unit_tests/admission_control_service_test.rs"]
mod admission_control_service_test;

#[cfg(any(feature = "fuzzing", test))]
#[path = "admission_control_fuzzing.rs"]
/// fuzzing module for admission control
pub mod fuzzing;

/// Struct implementing trait (service handle) AdmissionControlService.
#[derive(Clone)]
pub struct AdmissionControlService<M, V> {
    /// gRPC client connecting Mempool.
    mempool_client: Option<Arc<M>>,
    /// gRPC client to send read requests to Storage.
    storage_read_client: Arc<dyn StorageRead>,
    /// VM validator instance to validate transactions sent from wallets.
    vm_validator: Arc<V>,
    /// Flag indicating whether we need to check mempool before validation, drop txn if check
    /// fails.
    need_to_check_mempool_before_validation: bool,
    /// mpsc sender connection to send transaction message to upstream proxy
    upstream_proxy_sender: mpsc::UnboundedSender<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
}

impl<M: 'static, V> AdmissionControlService<M, V>
where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    /// Constructs a new AdmissionControlService instance.
    pub fn new(
        mempool_client: Option<Arc<M>>,
        storage_read_client: Arc<dyn StorageRead>,
        vm_validator: Arc<V>,
        need_to_check_mempool_before_validation: bool,
        upstream_proxy_sender: mpsc::UnboundedSender<(
            SubmitTransactionRequest,
            oneshot::Sender<failure::Result<SubmitTransactionResponse>>,
        )>,
    ) -> Self {
        AdmissionControlService {
            mempool_client,
            storage_read_client,
            vm_validator,
            need_to_check_mempool_before_validation,
            upstream_proxy_sender,
        }
    }

    /// Validate transaction signature, then via VM, and add it to Mempool if it passes VM check.
    pub(crate) fn submit_transaction_inner(
        &self,
        req: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse> {
        // Drop requests first if mempool is full (validator is lagging behind) so not to consume
        // unnecessary resources.
        if !self.can_send_txn_to_mempool()? {
            debug!("Mempool is full");
            OP_COUNTERS.inc_by("submit_txn.rejected.mempool_full", 1);
            let mut response = SubmitTransactionResponse::default();
            let mut status = MempoolAddTransactionStatus::default();
            status.set_code(MempoolIsFull);
            status.message = "Mempool is full".to_string();
            response.status = Some(Status::MempoolStatus(status));
            return Ok(response);
        }

        let signed_txn_proto = req.signed_txn.clone().unwrap_or_else(Default::default);

        let signed_txn = match SignedTransaction::try_from(signed_txn_proto.clone()) {
            Ok(t) => t,
            Err(e) => {
                security_log(SecurityEvent::InvalidTransactionAC)
                    .error(&e)
                    .data(&signed_txn_proto)
                    .log();
                let mut response = SubmitTransactionResponse::default();
                response.status = Some(Status::AcStatus(
                    AdmissionControlStatus::Rejected("submit txn rejected".to_string()).into(),
                ));
                OP_COUNTERS.inc_by("submit_txn.rejected.invalid_txn", 1);
                return Ok(response);
            }
        };

        let gas_cost = signed_txn.max_gas_amount();
        let validation_status = self
            .vm_validator
            .validate_transaction(signed_txn.clone())
            .wait()
            .map_err(|e| {
                security_log(SecurityEvent::InvalidTransactionAC)
                    .error(&e)
                    .data(&signed_txn)
                    .log();
                e
            })?;
        if let Some(validation_status) = validation_status {
            let mut response = SubmitTransactionResponse::default();
            OP_COUNTERS.inc_by("submit_txn.vm_validation.failure", 1);
            debug!(
                "txn failed in vm validation, status: {:?}, txn: {:?}",
                validation_status, signed_txn
            );
            response.status = Some(Status::VmStatus(validation_status.into()));
            return Ok(response);
        }
        let sender = signed_txn.sender();
        let account_state = block_on(get_account_state(self.storage_read_client.clone(), sender));
        let mut add_transaction_request = AddTransactionWithValidationRequest::default();
        add_transaction_request.signed_txn = req.signed_txn.clone();
        add_transaction_request.max_gas_cost = gas_cost;

        if let Ok((sequence_number, balance)) = account_state {
            add_transaction_request.account_balance = balance;
            add_transaction_request.latest_sequence_number = sequence_number;
        }

        self.add_txn_to_mempool(add_transaction_request)
    }

    fn can_send_txn_to_mempool(&self) -> Result<bool> {
        if self.need_to_check_mempool_before_validation {
            let req = HealthCheckRequest::default();
            let is_mempool_healthy = match &self.mempool_client {
                Some(client) => client.health_check(&req)?.is_healthy,
                None => false,
            };
            return Ok(is_mempool_healthy);
        }
        Ok(true)
    }

    /// Add signed transaction to mempool once it passes vm check
    fn add_txn_to_mempool(
        &self,
        add_transaction_request: AddTransactionWithValidationRequest,
    ) -> Result<SubmitTransactionResponse> {
        match &self.mempool_client {
            Some(mempool_client) => {
                let mempool_result =
                    mempool_client.add_transaction_with_validation(&add_transaction_request)?;

                debug!("[GRPC] Done with transaction submission request");
                let mut response = SubmitTransactionResponse::default();
                if let Some(status) = mempool_result.status {
                    if status.code() == MempoolAddTransactionStatusCode::Valid {
                        OP_COUNTERS.inc_by("submit_txn.txn_accepted", 1);
                        response.status =
                            Some(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
                    } else {
                        debug!(
                            "txn failed in mempool, status: {:?}, txn: {:?}",
                            status, add_transaction_request.signed_txn
                        );
                        OP_COUNTERS.inc_by("submit_txn.mempool.failure", 1);
                        response.status = Some(Status::MempoolStatus(status));
                    }
                }
                Ok(response)
            }
            None => Err(format_err!("Mempool is not initialized")),
        }
    }

    /// Pass the UpdateToLatestLedgerRequest to Storage for read query.
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
            .storage_read_client
            .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;
        let rust_resp = libra_types::get_with_proof::UpdateToLatestLedgerResponse::new(
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
            ledger_consistency_proof,
        );
        Ok(rust_resp.into())
    }
}

impl<M: 'static, V> AdmissionControl for AdmissionControlService<M, V>
where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    /// Submit a transaction to the validator this AC instance connecting to.
    /// The specific transaction will be first validated by VM and then passed
    /// to Mempool for further processing.
    fn submit_transaction(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        req: SubmitTransactionRequest,
        sink: ::grpcio::UnarySink<SubmitTransactionResponse>,
    ) {
        debug!("[GRPC] AdmissionControl::submit_transaction");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = match self.mempool_client {
            None => {
                let (req_sender, res_receiver) = oneshot::channel();
                let sent_result = self.upstream_proxy_sender.unbounded_send((req, req_sender));
                match sent_result {
                    Ok(()) => {
                        let result = block_on(res_receiver);
                        match result {
                            Ok(res) => res,
                            Err(e) => Err(format_err!(
                                "[admission-control] Upstream transaction failed with error: {:?}",
                                e
                            )),
                        }
                    }
                    Err(e) => Err(format_err!(
                        "[admission-control] Failed to submit write request with error: {:?}",
                        e
                    )),
                }
            }
            Some(_) => self.submit_transaction_inner(req),
        };
        provide_grpc_response(resp, ctx, sink);
    }

    /// This API is used to update the client to the latest ledger version and optionally also
    /// request 1..n other pieces of data.  This allows for batch queries.  All queries return
    /// proofs that a client should check to validate the data.
    /// Note that if a client only wishes to update to the latest LedgerInfo and receive the proof
    /// of this latest version, they can simply omit the requested_items (or pass an empty list).
    /// AC will not directly process this request but pass it to Storage instead.
    fn update_to_latest_ledger(
        &mut self,
        ctx: grpcio::RpcContext<'_>,
        req: libra_types::proto::types::UpdateToLatestLedgerRequest,
        sink: grpcio::UnarySink<libra_types::proto::types::UpdateToLatestLedgerResponse>,
    ) {
        debug!("[GRPC] AdmissionControl::update_to_latest_ledger");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.update_to_latest_ledger_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }
}
