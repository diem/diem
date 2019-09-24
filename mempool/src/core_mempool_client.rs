use crate::core_mempool::CoreMempool;
use std::sync::{Arc, Mutex};
use config::config::NodeConfig;
use crate::proto::mempool_client::MempoolClientTrait;
use crate::proto::mempool::{AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
                     GetBlockRequest, GetBlockResponse,
                     CommitTransactionsRequest, CommitTransactionsResponse,
                     HealthCheckRequest, HealthCheckResponse};
use futures::Future;

/// Client for CoreMemPool
#[derive(Clone)]
pub struct CoreMemPoolClient {
    mem_pool: Arc<Mutex<CoreMempool>>
}

impl CoreMemPoolClient {
    /// Create CoreMemPoolClient
    pub fn new(config: &NodeConfig) -> Self {
        let mem_pool = Arc::new(Mutex::new(CoreMempool::new(&config)));
        CoreMemPoolClient { mem_pool }
    }
}

impl MempoolClientTrait for CoreMemPoolClient {

    fn add_transaction_with_validation(&self, req: &AddTransactionWithValidationRequest)
                                       -> ::grpcio::Result<AddTransactionWithValidationResponse> {
        unimplemented!();
    }

    fn add_transaction_with_validation_async(&self, req: &AddTransactionWithValidationRequest)
                                             -> ::grpcio::Result<Box<Future<Item=AddTransactionWithValidationResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }

    fn get_block(&self, req: &GetBlockRequest)
                 -> ::grpcio::Result<GetBlockResponse> {
        unimplemented!();
    }

    fn get_block_async(&self, req: &GetBlockRequest)
                       -> ::grpcio::Result<Box<Future<Item=GetBlockResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }

    fn commit_transactions(&self, req: &CommitTransactionsRequest)
                           -> ::grpcio::Result<CommitTransactionsResponse> {
        unimplemented!();
    }

    fn commit_transactions_async(&self, req: &CommitTransactionsRequest)
                                 -> ::grpcio::Result<Box<Future<Item=CommitTransactionsResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }

    fn health_check(&self, req: &HealthCheckRequest)
                    -> ::grpcio::Result<HealthCheckResponse> {
        unimplemented!();
    }

    fn health_check_async(&self, req: &HealthCheckRequest)
                          -> ::grpcio::Result<Box<Future<Item=HealthCheckResponse, Error=::grpcio::Error> + Send>> {
        unimplemented!();
    }
}