use crate::admission_control_service::AdmissionControlService;
use admission_control_proto::proto::{admission_control_client::AdmissionControlClientTrait,
                                     admission_control::{SubmitTransactionRequest, SubmitTransactionResponse}};
use types::proto::get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use futures::Future;
use std::sync::Arc;
use mempool::proto::mempool_client::MempoolClientTrait;
use vm_validator::vm_validator::TransactionValidation;

/// AdmissionControlClient
#[derive(Clone)]
pub struct AdmissionControlClient<M, V> {
    ac_service: Arc<AdmissionControlService<M, V>>
}

impl<M: 'static, V> AdmissionControlClient<M, V> where M: MempoolClientTrait, V: TransactionValidation + Clone {
    /// AdmissionControlService Wrapper
    pub fn new(ac_service: AdmissionControlService<M, V>) -> Self {
        AdmissionControlClient { ac_service: Arc::new(ac_service) }
    }
}

impl<M: 'static, V> AdmissionControlClientTrait for AdmissionControlClient<M, V> where M: MempoolClientTrait, V: TransactionValidation + Clone {
    fn submit_transaction(&self, req: &SubmitTransactionRequest) -> ::grpcio::Result<SubmitTransactionResponse> {
        self.ac_service.submit_transaction_inner(req.clone()).map_err(|e| {
            ::grpcio::Error::InvalidMetadata(e.to_string())
        })
    }
    fn submit_transaction_async(&self, req: &SubmitTransactionRequest) -> ::grpcio::Result<Box<Future<Item=SubmitTransactionResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }

    fn update_to_latest_ledger(&self, req: &UpdateToLatestLedgerRequest) -> ::grpcio::Result<UpdateToLatestLedgerResponse> {
        self.ac_service.update_to_latest_ledger_inner(req.clone()).map_err(|e| {
            ::grpcio::Error::InvalidMetadata(e.to_string())
        })
    }
    fn update_to_latest_ledger_async(&self, req: &UpdateToLatestLedgerRequest) -> ::grpcio::Result<Box<Future<Item=UpdateToLatestLedgerResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }
}