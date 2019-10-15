use std::{
    collections::HashSet,
    convert::TryFrom,
    sync::{Arc, Mutex},
};
use config::config::NodeConfig;
use libra_types::{
    account_address::AccountAddress,
    transaction::SignedTransaction,
};

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    proto::{
        mempool::{
            AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
            HealthCheckRequest, HealthCheckResponse,
        },
        mempool_client::MempoolClientTrait,
    },
};

/// Client for CoreMemPool
#[derive(Clone)]
pub struct CoreMemPoolClient {
    core_mempool: Arc<Mutex<CoreMempool>>,
}

impl CoreMemPoolClient {
    /// Create CoreMemPoolClient
    pub fn new(config: &NodeConfig) -> Self {
        let core_mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
        CoreMemPoolClient { core_mempool }
    }

    /// remove txn from mock chain
    pub fn remove_txn(&self, exclude_transactions: HashSet<((AccountAddress, u64))> ) {
        let mut lock = self.core_mempool.lock().expect("get lock err.");
        exclude_transactions
            .iter()
            .for_each(|(addr, seq_num)| lock.remove_transaction(addr, seq_num.clone(), true));
    }
    ///TODO doc
    pub fn get_block(&self, batch_size: u64, seen: HashSet<TxnPointer>) -> Vec<SignedTransaction> {
        self
            .core_mempool
            .lock()
            .expect("[get_block] acquire mempool lock")
            .get_block(batch_size, seen)
    }
}

impl MempoolClientTrait for CoreMemPoolClient {
    fn add_transaction_with_validation(
        &self,
        req: &AddTransactionWithValidationRequest,
    ) -> ::grpcio::Result<AddTransactionWithValidationResponse> {
        //TODO fix unwrap
        let transaction = SignedTransaction::try_from(req.signed_txn.clone().unwrap())
            .expect("SignedTransaction from proto err.");
        let insertion_result = self
            .core_mempool
            .lock()
            .expect("[add txn] acquire mempool lock")
            .add_txn(
                transaction,
                req.max_gas_cost,
                req.latest_sequence_number,
                req.account_balance,
                TimelineState::NotReady,
            );
        let mut response = crate::proto::mempool::AddTransactionWithValidationResponse::default();
        response.status = Some(insertion_result.into());
        Ok(response)
    }

    fn health_check(&self, _req: &HealthCheckRequest) -> ::grpcio::Result<HealthCheckResponse> {
        let pool = self
            .core_mempool
            .lock()
            .expect("[health_check] acquire mempool lock");
        let mut response = HealthCheckResponse::default();
        response.is_healthy = pool.health_check();
        Ok(response)
    }
}
