// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Admission Control (AC) is a module acting as the only public end point. It receives api requests
//! from external clients (such as wallets) and performs necessary processing before sending them to
//! next step.

use crate::OP_COUNTERS;
use admission_control_proto::{
    proto::{
        admission_control::{SubmitTransactionRequest, SubmitTransactionResponse},
        admission_control_grpc::AdmissionControl,
    },
    AdmissionControlStatus,
};
use failure::prelude::*;
use futures::future::Future;
use futures03::executor::block_on;
use grpc_helpers::provide_grpc_response;
use logger::prelude::*;
use mempool::proto::{
    mempool::{AddTransactionWithValidationRequest, HealthCheckRequest},
    mempool_client::MempoolClientTrait,
    shared::mempool_status::{
        MempoolAddTransactionStatus,
        MempoolAddTransactionStatusCode::{self, MempoolIsFull},
    },
};
use metrics::counters::SVC_COUNTERS;
use proto_conv::{FromProto, IntoProto};
use std::sync::Arc;
use storage_client::StorageRead;
use types::{
    proto::get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
    transaction::SignedTransaction,
};
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
    ) -> Self {
        AdmissionControlService {
            mempool_client,
            storage_read_client,
            vm_validator,
            need_to_check_mempool_before_validation,
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
            let mut response = SubmitTransactionResponse::new();
            let mut status = MempoolAddTransactionStatus::new();
            status.set_code(MempoolIsFull);
            status.set_message("Mempool is full".to_string());
            response.set_mempool_status(status);
            return Ok(response);
        }

        let signed_txn_proto = req.get_signed_txn();

        let signed_txn = match SignedTransaction::from_proto(signed_txn_proto.clone()) {
            Ok(t) => t,
            Err(e) => {
                security_log(SecurityEvent::InvalidTransactionAC)
                    .error(&e)
                    .data(&signed_txn_proto)
                    .log();
                let mut response = SubmitTransactionResponse::new();
                response.set_ac_status(
                    AdmissionControlStatus::Rejected("submit txn rejected".to_string())
                        .into_proto(),
                );
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
            let mut response = SubmitTransactionResponse::new();
            OP_COUNTERS.inc_by("submit_txn.vm_validation.failure", 1);
            debug!(
                "txn failed in vm validation, status: {:?}, txn: {:?}",
                validation_status, signed_txn
            );
            response.set_vm_status(validation_status.into_proto());
            return Ok(response);
        }
        let sender = signed_txn.sender();
        let account_state = block_on(get_account_state(self.storage_read_client.clone(), sender));
        let mut add_transaction_request = AddTransactionWithValidationRequest::new();
        add_transaction_request.signed_txn = req.signed_txn.clone();
        add_transaction_request.set_max_gas_cost(gas_cost);

        if let Ok((sequence_number, balance)) = account_state {
            add_transaction_request.set_account_balance(balance);
            add_transaction_request.set_latest_sequence_number(sequence_number);
        }

        self.add_txn_to_mempool(add_transaction_request)
    }

    fn can_send_txn_to_mempool(&self) -> Result<bool> {
        if self.need_to_check_mempool_before_validation {
            let req = HealthCheckRequest::new();
            let is_mempool_healthy = match &self.mempool_client {
                Some(client) => client.health_check(&req)?.get_is_healthy(),
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
                let mut mempool_result =
                    mempool_client.add_transaction_with_validation(&add_transaction_request)?;

                debug!("[GRPC] Done with transaction submission request");
                let mut response = SubmitTransactionResponse::new();
                if mempool_result.get_status().get_code() == MempoolAddTransactionStatusCode::Valid
                {
                    OP_COUNTERS.inc_by("submit_txn.txn_accepted", 1);
                    response.set_ac_status(AdmissionControlStatus::Accepted.into_proto());
                } else {
                    debug!(
                        "txn failed in mempool, status: {:?}, txn: {:?}",
                        mempool_result,
                        add_transaction_request.get_signed_txn()
                    );
                    OP_COUNTERS.inc_by("submit_txn.mempool.failure", 1);
                    response.set_mempool_status(mempool_result.take_status());
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
        let rust_req = types::get_with_proof::UpdateToLatestLedgerRequest::from_proto(req)?;
        let (response_items, ledger_info_with_sigs, validator_change_events) = self
            .storage_read_client
            .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;
        let rust_resp = types::get_with_proof::UpdateToLatestLedgerResponse::new(
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
        );
        Ok(rust_resp.into_proto())
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
            None => Err(format_err!("Node doesn't accept write requests")),
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
        req: types::proto::get_with_proof::UpdateToLatestLedgerRequest,
        sink: grpcio::UnarySink<types::proto::get_with_proof::UpdateToLatestLedgerResponse>,
    ) {
        debug!("[GRPC] AdmissionControl::update_to_latest_ledger");
        let _timer = SVC_COUNTERS.req(&ctx);
        let resp = self.update_to_latest_ledger_inner(req);
        provide_grpc_response(resp, ctx, sink);
    }
}
