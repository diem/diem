// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    mempool_service::MempoolService,
    proto::mempool::*,
    proto::mempool_client::MempoolClientWrapper,
};
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::compat::generate_keypair;
use libra_types::{
    account_address::AccountAddress,
    test_helpers::transaction_test_helpers::get_test_signed_transaction,
    transaction::SignedTransaction,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

async fn setup_mempool() -> (MempoolClientWrapper, Arc<Mutex<CoreMempool>>) {
    let node_config = NodeConfig::random();

    let core_mempool = Arc::new(Mutex::new(CoreMempool::new(&node_config)));
    let mempool_clone = Arc::clone(&core_mempool);
    let handle = MempoolService { core_mempool };
    let service = mempool_server::MempoolServer::new(handle);

    let port = libra_config::utils::get_available_port();
    let mut client = MempoolClientWrapper::new("127.0.0.1", port);

    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(service)
            .serve(([127, 0, 0, 1], port).into()),
    );

    while let Err(_) = client.health_check(HealthCheckRequest::default()).await {}

    (client, mempool_clone)
}

fn add_txn_to_mempool(
    expiration_time: u64,
    account_balance: Option<u64>,
    mempool: Arc<Mutex<CoreMempool>>,
) -> SignedTransaction {
    let sender = AccountAddress::random();
    let (private_key, public_key) = generate_keypair(None);
    let transaction = get_test_signed_transaction(
        sender,
        0,
        private_key,
        public_key,
        None,
        expiration_time,
        1,
        None,
    );
    let balance = match account_balance {
        Some(balance) => balance,
        None => 1000,
    };

    mempool
        .lock()
        .expect("failed to acquire mempool lock")
        .add_txn(transaction.clone(), 10, 0, balance, TimelineState::NotReady);

    transaction
}

#[tokio::test]
async fn test_get_block() {
    let (mut client, mempool) = setup_mempool().await;

    let transaction = add_txn_to_mempool(0, Some(100), mempool);

    // get next block
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    let block = response.block.unwrap();
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(block.transactions[0], transaction.into());
}

#[tokio::test]
async fn test_consensus_callbacks() {
    let (mut client, mempool) = setup_mempool().await;

    // add transaction
    let signed_txn = add_txn_to_mempool(0, None, mempool);

    let mut response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert_eq!(response.block.unwrap().transactions.len(), 1);

    // remove: transaction is committed
    let mut transaction = CommittedTransaction::default();
    let sender = signed_txn.sender().as_ref().to_vec();
    transaction.sender = sender;
    transaction.sequence_number = 0;

    let mut req = CommitTransactionsRequest::default();
    req.transactions = vec![transaction];
    client.commit_transactions(req).await.unwrap();
    response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert!(response.block.unwrap().transactions.is_empty());
}

#[tokio::test]
async fn test_gc_by_expiration_time() {
    let (mut client, mempool) = setup_mempool().await;

    // add transaction with expiration time 1
    add_txn_to_mempool(1, None, Arc::clone(&mempool));

    // commit empty block with block_time 2
    let mut req = CommitTransactionsRequest::default();
    req.block_timestamp_usecs = Duration::from_secs(2).as_micros() as u64;
    client.commit_transactions(req).await.unwrap();

    // verify that transaction is evicted from Mempool
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert!(response.block.unwrap().transactions.is_empty());

    // add transaction with expiration time 3
    add_txn_to_mempool(3, None, mempool);

    // commit empty block with block_time 3
    let mut req = CommitTransactionsRequest::default();
    req.block_timestamp_usecs = Duration::from_secs(3).as_micros() as u64;
    client.commit_transactions(req).await.unwrap();

    // verify that transaction is still in Mempool
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert_eq!(response.block.unwrap().transactions.len(), 1);
}
