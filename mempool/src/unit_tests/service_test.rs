// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool, mempool_service::MempoolService, proto::mempool::*,
    proto::mempool_client::MempoolClientWrapper,
};
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::compat::generate_keypair;
use libra_mempool_shared_proto::proto::mempool_status::*;
use libra_types::{
    account_address::AccountAddress,
    test_helpers::transaction_test_helpers::get_test_signed_transaction,
    transaction::SignedTransaction,
};
use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::Duration,
};

async fn setup_mempool() -> MempoolClientWrapper {
    let node_config = NodeConfig::random();

    let core_mempool = Arc::new(Mutex::new(CoreMempool::new(&node_config)));
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

    client
}

fn create_add_transaction_request(expiration_time: u64) -> AddTransactionWithValidationRequest {
    let mut req = AddTransactionWithValidationRequest::default();
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
    )
    .into();
    req.transaction = Some(transaction);
    req.max_gas_cost = 10;
    req.account_balance = 1000;
    req
}

#[tokio::test]
async fn test_add_transaction() {
    let mut client = setup_mempool().await;
    // create request
    let mut req = create_add_transaction_request(0);
    req.account_balance = 100;
    let response = client.add_transaction_with_validation(req).await.unwrap();
    // check status
    assert_eq!(
        response.status.unwrap().code(),
        MempoolAddTransactionStatusCode::Valid
    );
}

#[tokio::test]
async fn test_get_block() {
    let mut client = setup_mempool().await;

    // add transaction to mempool
    let mut req = create_add_transaction_request(0);
    req.account_balance = 100;
    client
        .add_transaction_with_validation(req.clone())
        .await
        .unwrap();

    // get next block
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    let block = response.block.unwrap();
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(block.transactions[0], req.transaction.unwrap(),);
}

#[tokio::test]
async fn test_consensus_callbacks() {
    let mut client = setup_mempool().await;

    // add transaction
    let add_req = create_add_transaction_request(0);
    client
        .add_transaction_with_validation(add_req.clone())
        .await
        .unwrap();

    let mut response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert_eq!(response.block.unwrap().transactions.len(), 1);

    // remove: transaction is committed
    let mut transaction = CommittedTransaction::default();
    let signed_txn = SignedTransaction::try_from(add_req.transaction.unwrap().clone()).unwrap();
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
    let mut client = setup_mempool().await;

    // add transaction with expiration time 1
    let add_req = create_add_transaction_request(1);
    client
        .add_transaction_with_validation(add_req)
        .await
        .unwrap();

    // commit empty block with block_time 2
    let mut req = CommitTransactionsRequest::default();
    req.block_timestamp_usecs = Duration::from_secs(2).as_micros() as u64;
    client.commit_transactions(req).await.unwrap();

    // verify that transaction is evicted from Mempool
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert!(response.block.unwrap().transactions.is_empty());

    // add transaction with expiration time 3
    let add_req = create_add_transaction_request(3);
    client
        .add_transaction_with_validation(add_req)
        .await
        .unwrap();
    // commit empty block with block_time 3
    let mut req = CommitTransactionsRequest::default();
    req.block_timestamp_usecs = Duration::from_secs(3).as_micros() as u64;
    client.commit_transactions(req).await.unwrap();

    // verify that transaction is still in Mempool
    let response = client.get_block(GetBlockRequest::default()).await.unwrap();
    assert_eq!(response.block.unwrap().transactions.len(), 1);
}
