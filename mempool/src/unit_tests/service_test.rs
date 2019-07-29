// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool,
    mempool_service::MempoolService,
    proto::{
        mempool::*,
        mempool_grpc::{self, *},
        shared::mempool_status::*,
    },
};
use config::config::NodeConfigHelpers;
use grpc_helpers::ServerHandle;
use grpcio::{ChannelBuilder, EnvBuilder};
use nextgen_crypto::ed25519::compat::generate_keypair;
use proto_conv::FromProto;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use types::{
    account_address::AccountAddress,
    test_helpers::transaction_test_helpers::get_test_signed_transaction,
    transaction::SignedTransaction,
};

fn setup_mempool() -> (::grpcio::Server, MempoolClient) {
    let node_config = NodeConfigHelpers::get_single_node_test_config(true);

    let env = Arc::new(EnvBuilder::new().build());
    let core_mempool = Arc::new(Mutex::new(CoreMempool::new(&node_config)));
    let handle = MempoolService { core_mempool };
    let service = mempool_grpc::create_mempool(handle);

    let server = ::grpcio::ServerBuilder::new(env.clone())
        .register_service(service)
        .bind("localhost", 0)
        .build()
        .expect("Unable to create grpc server");
    let (_, port) = server.bind_addrs()[0];
    let connection_str = format!("localhost:{}", port);
    let client = MempoolClient::new(ChannelBuilder::new(env).connect(&connection_str));
    (server, client)
}

fn create_add_transaction_request(expiration_time: u64) -> AddTransactionWithValidationRequest {
    let mut req = AddTransactionWithValidationRequest::new();
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
    req.set_signed_txn(transaction.clone());
    req.set_max_gas_cost(10);
    req.set_account_balance(1000);
    req
}

#[test]
fn test_add_transaction() {
    let (server, client) = setup_mempool();
    let _handle = ServerHandle::setup(server);
    // create request
    let mut req = create_add_transaction_request(0);
    req.set_account_balance(100);
    let mut response = client.add_transaction_with_validation(&req).unwrap();
    // check status
    assert_eq!(
        response.take_status().get_code(),
        MempoolAddTransactionStatusCode::Valid
    );
}

#[test]
fn test_get_block() {
    let (server, client) = setup_mempool();
    let _handle = ServerHandle::setup(server);

    // add transaction to mempool
    let mut req = create_add_transaction_request(0);
    req.set_account_balance(100);
    client.add_transaction_with_validation(&req).unwrap();

    // get next block
    let response = client.get_block(&GetBlockRequest::new()).unwrap();
    let block = response.get_block();
    assert_eq!(block.get_transactions().len(), 1);
    assert_eq!(
        block.get_transactions()[0].raw_txn_bytes,
        req.get_signed_txn().raw_txn_bytes
    );
}

#[test]
fn test_consensus_callbacks() {
    let (server, client) = setup_mempool();
    let _handle = ServerHandle::setup(server);

    // add transaction
    let add_req = create_add_transaction_request(0);
    client.add_transaction_with_validation(&add_req).unwrap();

    let mut response = client.get_block(&GetBlockRequest::new()).unwrap();
    assert_eq!(response.get_block().get_transactions().len(), 1);

    // remove: transaction is committed
    let mut transaction = CommittedTransaction::new();
    let signed_txn = SignedTransaction::from_proto(add_req.get_signed_txn().clone()).unwrap();
    let sender = signed_txn.sender().as_ref().to_vec();
    transaction.set_sender(sender);
    transaction.set_sequence_number(0);

    let mut req = CommitTransactionsRequest::new();
    req.set_transactions(::protobuf::RepeatedField::from_vec(vec![transaction]));
    client.commit_transactions(&req).unwrap();
    response = client.get_block(&GetBlockRequest::new()).unwrap();
    assert!(response.get_block().get_transactions().is_empty());
}

#[test]
fn test_gc_by_expiration_time() {
    let (server, client) = setup_mempool();
    let _handle = ServerHandle::setup(server);

    // add transaction with expiration time 1
    let add_req = create_add_transaction_request(1);
    client.add_transaction_with_validation(&add_req).unwrap();

    // commit empty block with block_time 2
    let mut req = CommitTransactionsRequest::new();
    req.set_block_timestamp_usecs(Duration::from_secs(2).as_micros() as u64);
    client.commit_transactions(&req).unwrap();

    // verify that transaction is evicted from Mempool
    let response = client.get_block(&GetBlockRequest::new()).unwrap();
    assert!(response.get_block().get_transactions().is_empty());

    // add transaction with expiration time 3
    let add_req = create_add_transaction_request(3);
    client.add_transaction_with_validation(&add_req).unwrap();
    // commit empty block with block_time 3
    let mut req = CommitTransactionsRequest::new();
    req.set_block_timestamp_usecs(Duration::from_secs(3).as_micros() as u64);
    client.commit_transactions(&req).unwrap();

    // verify that transaction is still in Mempool
    let response = client.get_block(&GetBlockRequest::new()).unwrap();
    assert_eq!(response.get_block().get_transactions().len(), 1);
}
