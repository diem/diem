use crate::admission_control_service::AdmissionControlService;
use admission_control_proto::proto::admission_control::{
    SubmitTransactionRequest, SubmitTransactionResponse,
};
use libra_types::proto::types::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use std::sync::Arc;

/// AdmissionControlClient
#[derive(Clone)]
pub struct AdmissionControlMockClient {
    ac_service: Arc<AdmissionControlService>,
}

impl AdmissionControlMockClient {
    /// AdmissionControlService Wrapper
    pub fn new(ac_service: AdmissionControlService) -> Self {
        AdmissionControlMockClient {
            ac_service: Arc::new(ac_service),
        }
    }

    /// TODO doc
    pub fn submit_transaction(
        &self,
        req: &SubmitTransactionRequest,
    ) -> ::grpcio::Result<SubmitTransactionResponse> {
        self.ac_service
            .submit_transaction_inner(req.clone())
            .map_err(|e| ::grpcio::Error::InvalidMetadata(e.to_string()))
    }

    /// TODO doc
    pub fn update_to_latest_ledger(
        &self,
        req: &UpdateToLatestLedgerRequest,
    ) -> ::grpcio::Result<UpdateToLatestLedgerResponse> {
        self.ac_service
            .update_to_latest_ledger_inner(req.clone())
            .map_err(|e| ::grpcio::Error::InvalidMetadata(e.to_string()))
    }
}
