use crate::admission_control_service::AdmissionControlService;
use admission_control_proto::proto::admission_control::{
    SubmitTransactionRequest, SubmitTransactionResponse,
};
use libra_types::proto::types::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use std::sync::Arc;
use crate::UpstreamProxyData;
use libra_mempool::core_mempool_client::CoreMemPoolClient;
use vm_validator::vm_validator::VMValidator;
use tokio::runtime::TaskExecutor;

/// AdmissionControlClient
#[derive(Clone)]
pub struct AdmissionControlMockClient {
    ac_service: Arc<AdmissionControlService>,
    proxy: UpstreamProxyData<CoreMemPoolClient, VMValidator>,
    executor: TaskExecutor,
}

impl AdmissionControlMockClient {
    /// AdmissionControlService Wrapper
    pub fn new(ac_service: AdmissionControlService, proxy: UpstreamProxyData<CoreMemPoolClient, VMValidator>, executor: TaskExecutor) -> Self {
        AdmissionControlMockClient {
            ac_service: Arc::new(ac_service),
            proxy,
            executor
        }
    }

    /// TODO doc
    pub fn submit_transaction(
        &self,
        req: &SubmitTransactionRequest,
    ) -> ::grpcio::Result<SubmitTransactionResponse> {
        self.ac_service
            .submit_transaction_inner(self.executor.clone(), self.proxy.clone(), req.clone())
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
