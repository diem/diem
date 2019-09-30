use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    proto::{
        mempool::{
            AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
            CommitTransactionsRequest, CommitTransactionsResponse, GetBlockRequest,
            GetBlockResponse, HealthCheckRequest, HealthCheckResponse,
        },
        mempool_client::MempoolClientTrait,
    },
};
use config::config::NodeConfig;
use core::borrow::BorrowMut;
use futures::Future;
use proto_conv::{FromProto, IntoProto};
use std::{
    cmp,
    collections::HashSet,
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::Duration,
};
use types::{
    account_address::AccountAddress, proto::transaction::SignedTransactionsBlock,
    transaction::SignedTransaction,
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
    pub fn remove_txn(&self, req: &GetBlockRequest) {
        let exclude_transactions: HashSet<TxnPointer> = req
            .get_transactions()
            .iter()
            .map(|t| (AccountAddress::try_from(t.get_sender()), t.sequence_number))
            .filter(|(address, _)| address.is_ok())
            .map(|(address, seq)| (address.unwrap(), seq))
            .collect();

        let mut lock = self.core_mempool.lock().expect("get lock err.");
        exclude_transactions
            .iter()
            .for_each(|(addr, seq_num)| lock.remove_transaction(addr, seq_num.clone(), true));
    }
}

impl MempoolClientTrait for CoreMemPoolClient {
    fn add_transaction_with_validation(
        &self,
        req: &AddTransactionWithValidationRequest,
    ) -> ::grpcio::Result<AddTransactionWithValidationResponse> {
        let proto_transaction = req.clone().borrow_mut().take_signed_txn();
        let transaction = SignedTransaction::from_proto(proto_transaction)
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

        let mut response = AddTransactionWithValidationResponse::new();
        response.set_status(insertion_result.into_proto());
        Ok(response)
    }

    fn get_block(&self, req: &GetBlockRequest) -> ::grpcio::Result<GetBlockResponse> {
        let block_size = cmp::max(req.get_max_block_size(), 1);
        let exclude_transactions: HashSet<TxnPointer> = req
            .get_transactions()
            .iter()
            .map(|t| (AccountAddress::try_from(t.get_sender()), t.sequence_number))
            .filter(|(address, _)| address.is_ok())
            .map(|(address, seq)| (address.unwrap(), seq))
            .collect();

        let mut txns = self
            .core_mempool
            .lock()
            .expect("[get_block] acquire mempool lock")
            .get_block(block_size, exclude_transactions);

        let transactions = txns.drain(..).map(SignedTransaction::into_proto).collect();

        let mut block = SignedTransactionsBlock::new();
        block.set_transactions(::protobuf::RepeatedField::from_vec(transactions));
        let mut response = GetBlockResponse::new();
        response.set_block(block);
        Ok(response)
    }

    fn commit_transactions(
        &self,
        req: &CommitTransactionsRequest,
    ) -> ::grpcio::Result<CommitTransactionsResponse> {
        let mut pool = self
            .core_mempool
            .lock()
            .expect("[update status] acquire mempool lock");
        for transaction in req.get_transactions() {
            if let Ok(address) = AccountAddress::try_from(transaction.get_sender()) {
                let sequence_number = transaction.get_sequence_number();
                pool.remove_transaction(&address, sequence_number, transaction.get_is_rejected());
            }
        }
        let block_timestamp_usecs = req.get_block_timestamp_usecs();
        if block_timestamp_usecs > 0 {
            pool.gc_by_expiration_time(Duration::from_micros(block_timestamp_usecs));
        }
        let response = CommitTransactionsResponse::new();
        Ok(response)
    }

    fn health_check(&self, req: &HealthCheckRequest) -> ::grpcio::Result<HealthCheckResponse> {
        let pool = self
            .core_mempool
            .lock()
            .expect("[health_check] acquire mempool lock");
        let mut response = HealthCheckResponse::new();
        response.set_is_healthy(pool.health_check());
        Ok(response)
    }
}
