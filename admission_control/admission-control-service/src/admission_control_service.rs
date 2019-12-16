// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Admission Control (AC) is a module acting as the only public end point. It receives api requests
//! from external clients (such as wallets) and performs necessary processing before sending them to
//! next step.

use admission_control_proto::proto::admission_control::{
    AdmissionControl, SubmitTransactionRequest, SubmitTransactionResponse,
};
use anyhow::{format_err, Result};
use futures::{
    channel::{mpsc, oneshot},
    executor::block_on,
    SinkExt,
};
use grpc_helpers::provide_grpc_response;
use libra_logger::prelude::*;
use libra_metrics::counters::SVC_COUNTERS;
use libra_types::proto::types::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use std::convert::TryFrom;
use std::sync::Arc;
use storage_client::StorageRead;

/// Struct implementing trait (service handle) AdmissionControlService.
#[derive(Clone)]
pub struct AdmissionControlService {
    ac_sender: mpsc::Sender<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
    /// gRPC client to send read requests to Storage.
    storage_read_client: Arc<dyn StorageRead>,
}

impl AdmissionControlService {
    /// Constructs a new AdmissionControlService instance.
    pub fn new(
        ac_sender: mpsc::Sender<(
            SubmitTransactionRequest,
            oneshot::Sender<Result<SubmitTransactionResponse>>,
        )>,
        storage_read_client: Arc<dyn StorageRead>,
    ) -> Self {
        AdmissionControlService {
            ac_sender,
            storage_read_client,
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
            validator_change_proof,
            ledger_consistency_proof,
        ) = self
            .storage_read_client
            .update_to_latest_ledger(rust_req.client_known_version, rust_req.requested_items)?;
        let rust_resp = libra_types::get_with_proof::UpdateToLatestLedgerResponse::new(
            response_items,
            ledger_info_with_sigs,
            validator_change_proof,
            ledger_consistency_proof,
        );
        Ok(rust_resp.into())
    }
}

impl AdmissionControl for AdmissionControlService {
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

        let (req_sender, res_receiver) = oneshot::channel();
        let sent_result = block_on(self.ac_sender.send((req, req_sender)));
        let resp = match sent_result {
            Ok(()) => {
                let result = block_on(res_receiver);
                result.unwrap_or_else(|e| {
                    Err(format_err!(
                        "[admission-control] Submitting transaction failed with error: {:?}",
                        e
                    ))
                })
            }
            Err(e) => Err(format_err!(
                "[admission-control] Failed to submit write request with error: {:?}",
                e
            )),
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
