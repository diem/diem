// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    mock_vm::{
        encode_mint_transaction, encode_transfer_transaction, MockVM, DISCARD_STATUS, KEEP_STATUS,
    },
    Executor, OP_COUNTERS,
};
use config::config::{NodeConfig, NodeConfigHelpers};
use crypto::{hash::GENESIS_BLOCK_ID, HashValue};
use futures::executor::block_on;
use grpcio::{EnvBuilder, ServerBuilder};
use proptest::prelude::*;
use proto_conv::IntoProtoBytes;
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    sync::{mpsc, Arc},
};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_proto::proto::storage_grpc::create_storage;
use storage_service::StorageService;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
    transaction::{SignedTransaction, TransactionListWithProof, Version},
};
use vm_genesis::{encode_genesis_transaction, GENESIS_KEYPAIR};

fn get_config() -> NodeConfig {
    let config = NodeConfigHelpers::get_single_node_test_config(true);
    // Write out the genesis blob to the correct location.
    // XXX Should this logic live in NodeConfigHelpers?
    let genesis_txn = encode_genesis_transaction(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone());
    let mut file = File::create(&config.execution.genesis_file_location).unwrap();
    file.write_all(&genesis_txn.into_proto_bytes().unwrap())
        .unwrap();

    config
}

fn create_storage_server(config: &mut NodeConfig) -> (grpcio::Server, mpsc::Receiver<()>) {
    let (service, shutdown_receiver) = StorageService::new(&config.storage.get_dir());
    let mut server = ServerBuilder::new(Arc::new(EnvBuilder::new().build()))
        .register_service(create_storage(service))
        .bind("localhost", 0)
        .build()
        .expect("Failed to create storage server.");
    server.start();

    assert_eq!(server.bind_addrs().len(), 1);
    let (_, port) = server.bind_addrs()[0];

    // This is a little messy -- technically the config should also be used to set up the storage
    // server, but the code currently creates the storage server, binds it to a port, then sets up
    // the config.
    // XXX Clean this up a little.
    config.storage.port = port;

    (server, shutdown_receiver)
}

fn create_executor(config: &NodeConfig) -> Executor<MockVM> {
    let client_env = Arc::new(EnvBuilder::new().build());
    let read_client = Arc::new(StorageReadServiceClient::new(
        Arc::clone(&client_env),
        "localhost",
        config.storage.port,
    ));
    let write_client = Arc::new(StorageWriteServiceClient::new(
        Arc::clone(&client_env),
        "localhost",
        config.storage.port,
        None,
    ));
    Executor::new(read_client, write_client, config)
}

fn execute_and_commit_block(executor: &TestExecutor, txn_index: u64) {
    let txn = encode_mint_transaction(gen_address(txn_index), 100);
    let parent_block_id = match txn_index {
        0 => *GENESIS_BLOCK_ID,
        x => gen_block_id(x),
    };
    let id = gen_block_id(txn_index + 1);

    let response = block_on(executor.execute_block(vec![txn], parent_block_id, id))
        .unwrap()
        .unwrap();
    assert_eq!(response.version(), txn_index + 1);

    let ledger_info = gen_ledger_info(txn_index + 1, response.root_hash(), id, txn_index + 1);
    block_on(executor.commit_block(ledger_info))
        .unwrap()
        .unwrap();
}

struct TestExecutor {
    // The config is kept around because it owns the temp dir used in the test.
    _config: NodeConfig,
    storage_server: Option<grpcio::Server>,
    shutdown_receiver: mpsc::Receiver<()>,
    executor: Executor<MockVM>,
}

impl TestExecutor {
    fn new() -> TestExecutor {
        let mut config = get_config();
        let (storage_server, shutdown_receiver) = create_storage_server(&mut config);
        let executor = create_executor(&config);

        TestExecutor {
            _config: config,
            storage_server: Some(storage_server),
            shutdown_receiver,
            executor,
        }
    }
}

impl std::ops::Deref for TestExecutor {
    type Target = Executor<MockVM>;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}

impl Drop for TestExecutor {
    fn drop(&mut self) {
        self.storage_server
            .take()
            .expect("Storage server should exist.");
        self.shutdown_receiver.recv().unwrap();
    }
}

fn gen_address(index: u64) -> AccountAddress {
    let bytes = index.to_be_bytes();
    let mut buf = [0; ADDRESS_LENGTH];
    buf[ADDRESS_LENGTH - 8..].copy_from_slice(&bytes);
    AccountAddress::new(buf)
}

fn gen_block_id(index: u64) -> HashValue {
    let bytes = index.to_be_bytes();
    let mut buf = [0; HashValue::LENGTH];
    buf[HashValue::LENGTH - 8..].copy_from_slice(&bytes);
    HashValue::new(buf)
}

fn gen_ledger_info(
    version: u64,
    root_hash: HashValue,
    commit_block_id: HashValue,
    timestamp_usecs: u64,
) -> LedgerInfoWithSignatures {
    let ledger_info = LedgerInfo::new(
        version,
        root_hash,
        /* consensus_data_hash = */ HashValue::zero(),
        commit_block_id,
        /* epoch_num = */ 0,
        timestamp_usecs,
        None,
    );
    LedgerInfoWithSignatures::new(ledger_info, /* signatures = */ HashMap::new())
}

#[test]
fn test_executor_status() {
    let executor = TestExecutor::new();

    let txn0 = encode_mint_transaction(gen_address(0), 100);
    let txn1 = encode_mint_transaction(gen_address(1), 100);
    let txn2 = encode_transfer_transaction(gen_address(0), gen_address(1), 500);

    let parent_block_id = *GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);

    let response =
        block_on(executor.execute_block(vec![txn0, txn1, txn2], parent_block_id, block_id))
            .unwrap()
            .unwrap();

    assert_eq!(
        vec![
            KEEP_STATUS.clone(),
            KEEP_STATUS.clone(),
            DISCARD_STATUS.clone()
        ],
        response.status()
    );
}

#[test]
fn test_executor_one_block() {
    let executor = TestExecutor::new();

    let parent_block_id = *GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);
    let version = 100;

    let txns = (0..version)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .collect();
    let execute_block_future = executor.execute_block(txns, parent_block_id, block_id);
    let execute_block_response = block_on(execute_block_future).unwrap().unwrap();
    assert_eq!(execute_block_response.version(), 100);

    let ledger_info = gen_ledger_info(version, execute_block_response.root_hash(), block_id, 1);
    let commit_block_future = executor.commit_block(ledger_info);
    let _commit_block_response = block_on(commit_block_future).unwrap().unwrap();
}

#[test]
fn test_executor_multiple_blocks() {
    let executor = TestExecutor::new();

    for i in 0..100 {
        execute_and_commit_block(&executor, i)
    }
}

#[test]
fn test_executor_execute_same_block_multiple_times() {
    let parent_block_id = *GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);
    let version = 100;

    let txns: Vec<_> = (0..version)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .collect();

    {
        let executor = TestExecutor::new();
        let mut responses = vec![];
        for _i in 0..100 {
            let execute_block_future =
                executor.execute_block(txns.clone(), parent_block_id, block_id);
            let execute_block_response = block_on(execute_block_future).unwrap().unwrap();
            responses.push(execute_block_response);
        }
        responses.dedup();
        assert_eq!(responses.len(), 1);
    }
    {
        let executor = TestExecutor::new();
        let mut futures = vec![];
        for _i in 0..100 {
            let execute_block_future =
                executor.execute_block(txns.clone(), parent_block_id, block_id);
            futures.push(execute_block_future);
        }
        let mut responses: Vec<_> = futures
            .into_iter()
            .map(|fut| block_on(fut).unwrap().unwrap())
            .collect();
        responses.dedup();
        assert_eq!(responses.len(), 1);
    }
}

rusty_fork_test! {
    #[test]
    fn test_num_accounts_created_counter() {
        let executor = TestExecutor::new();
        for i in 0..20 {
            execute_and_commit_block(&executor, i);
            assert_eq!(OP_COUNTERS.counter("num_accounts").get() as u64, i + 1);
        }
    }
}

/// Generates a list of `TransactionListWithProof`s according to the given ranges.
fn create_transaction_chunks(
    chunk_ranges: Vec<std::ops::Range<Version>>,
) -> (Vec<TransactionListWithProof>, LedgerInfoWithSignatures) {
    assert_eq!(chunk_ranges.first().unwrap().start, 1);
    for i in 1..chunk_ranges.len() {
        let previous_range = &chunk_ranges[i - 1];
        let range = &chunk_ranges[i];
        assert!(previous_range.start <= previous_range.end);
        assert!(range.start <= range.end);
        assert!(range.start <= previous_range.end);
        assert!(previous_range.end <= range.end);
    }

    // To obtain the batches of transactions, we first execute and save all these transactions in a
    // separate DB. Then we call get_transactions to retrieve them.
    let mut config = get_config();
    let (storage_server, shutdown_receiver) = create_storage_server(&mut config);
    let executor = create_executor(&config);

    let mut txns = vec![];
    for i in 1..chunk_ranges.last().unwrap().end {
        let txn = encode_mint_transaction(gen_address(i), 100);
        txns.push(txn);
    }
    let id = gen_block_id(1);

    let response = block_on(executor.execute_block(txns.clone(), *GENESIS_BLOCK_ID, id))
        .unwrap()
        .unwrap();
    let ledger_version = txns.len() as u64;
    let ledger_info = gen_ledger_info(ledger_version, response.root_hash(), id, 1);
    block_on(executor.commit_block(ledger_info.clone()))
        .unwrap()
        .unwrap();

    let storage_client = StorageReadServiceClient::new(
        Arc::new(EnvBuilder::new().build()),
        "localhost",
        config.storage.port,
    );

    let batches: Vec<_> = chunk_ranges
        .into_iter()
        .map(|range| {
            storage_client
                .get_transactions(
                    range.start,
                    range.end - range.start,
                    ledger_version,
                    false, /* fetch_events */
                )
                .unwrap()
        })
        .collect();

    drop(storage_server);
    shutdown_receiver.recv().unwrap();

    (batches, ledger_info)
}

#[test]
fn test_executor_execute_chunk() {
    let first_batch_size = 30;
    let second_batch_size = 40;
    let third_batch_size = 20;
    let overlapping_size = 5;

    let (chunks, ledger_info) = {
        let first_batch_start = 1;
        let second_batch_start = first_batch_start + first_batch_size;
        let third_batch_start = second_batch_start + second_batch_size - overlapping_size;
        create_transaction_chunks(vec![
            first_batch_start..first_batch_start + first_batch_size,
            second_batch_start..second_batch_start + second_batch_size,
            third_batch_start..third_batch_start + third_batch_size,
        ])
    };

    // Now we execute these two chunks of transactions.
    let mut config = get_config();
    let (storage_server, shutdown_receiver) = create_storage_server(&mut config);
    let executor = create_executor(&config);
    let storage_client = StorageReadServiceClient::new(
        Arc::new(EnvBuilder::new().build()),
        "localhost",
        config.storage.port,
    );

    // Execute the first chunk. After that we should still get the genesis ledger info from DB.
    block_on(executor.execute_chunk(chunks[0].clone(), ledger_info.clone()))
        .unwrap()
        .unwrap();
    let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *GENESIS_BLOCK_ID);

    // Execute the second chunk. After that we should still get the genesis ledger info from DB.
    block_on(executor.execute_chunk(chunks[1].clone(), ledger_info.clone()))
        .unwrap()
        .unwrap();
    let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *GENESIS_BLOCK_ID);

    // Execute an empty chunk. After that we should still get the genesis ledger info from DB.
    block_on(executor.execute_chunk(TransactionListWithProof::new_empty(), ledger_info.clone()))
        .unwrap()
        .unwrap();
    let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *GENESIS_BLOCK_ID);

    // Execute the second chunk again. After that we should still get the same thing.
    block_on(executor.execute_chunk(chunks[1].clone(), ledger_info.clone()))
        .unwrap()
        .unwrap();
    let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *GENESIS_BLOCK_ID);

    // Execute the third chunk. After that we should get the new ledger info.
    block_on(executor.execute_chunk(chunks[2].clone(), ledger_info.clone()))
        .unwrap()
        .unwrap();
    let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
    assert_eq!(li, ledger_info);

    drop(storage_server);
    shutdown_receiver.recv().unwrap();
}

#[test]
fn test_executor_execute_chunk_restart() {
    let first_batch_size = 30;
    let second_batch_size = 40;

    let (chunks, ledger_info) = {
        let first_batch_start = 1;
        let second_batch_start = first_batch_start + first_batch_size;
        create_transaction_chunks(vec![
            first_batch_start..first_batch_start + first_batch_size,
            second_batch_start..second_batch_start + second_batch_size,
        ])
    };

    let mut config = get_config();
    let (storage_server, shutdown_receiver) = create_storage_server(&mut config);

    // First we simulate syncing the first chunk of transactions.
    {
        let executor = create_executor(&config);
        let storage_client = StorageReadServiceClient::new(
            Arc::new(EnvBuilder::new().build()),
            "localhost",
            config.storage.port,
        );

        block_on(executor.execute_chunk(chunks[0].clone(), ledger_info.clone()))
            .unwrap()
            .unwrap();
        let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
        assert_eq!(li.ledger_info().version(), 0);
        assert_eq!(li.ledger_info().consensus_block_id(), *GENESIS_BLOCK_ID);
    }

    // Then we restart executor and resume to the next chunk.
    {
        let executor = create_executor(&config);
        let storage_client = StorageReadServiceClient::new(
            Arc::new(EnvBuilder::new().build()),
            "localhost",
            config.storage.port,
        );

        block_on(executor.execute_chunk(chunks[1].clone(), ledger_info.clone()))
            .unwrap()
            .unwrap();
        let (_, li, _) = storage_client.update_to_latest_ledger(0, vec![]).unwrap();
        assert_eq!(li, ledger_info);
    }

    drop(storage_server);
    shutdown_receiver.recv().unwrap();
}

struct TestBlock {
    txns: Vec<SignedTransaction>,
    parent_id: HashValue,
    id: HashValue,
}

impl TestBlock {
    fn new(
        addr_index: std::ops::Range<u64>,
        amount: u32,
        parent_id: HashValue,
        id: HashValue,
    ) -> Self {
        TestBlock {
            txns: addr_index
                .map(|index| encode_mint_transaction(gen_address(index), u64::from(amount)))
                .collect(),
            parent_id,
            id,
        }
    }
}

// Executes a list of transactions by executing and immediately commtting one at a time. Returns
// the root hash after all transactions are committed.
fn run_transactions_naive(transactions: Vec<SignedTransaction>) -> HashValue {
    let executor = TestExecutor::new();
    let mut iter = transactions.into_iter();
    let first_txn = iter.next();
    let response = block_on(executor.execute_block(
        match first_txn {
            None => vec![],
            Some(txn) => vec![txn],
        },
        *GENESIS_BLOCK_ID,
        gen_block_id(1),
    ))
    .unwrap()
    .unwrap();
    let mut root_hash = response.root_hash();

    for (i, txn) in iter.enumerate() {
        let parent_block_id = gen_block_id(i as u64 + 1);
        // when i = 0, id should be 2.
        let id = gen_block_id(i as u64 + 2);
        let response = block_on(executor.execute_block(vec![txn], parent_block_id, id))
            .unwrap()
            .unwrap();

        root_hash = response.root_hash();
        let ledger_info = gen_ledger_info(i as u64 + 2, root_hash, id, i as u64 + 1);
        block_on(executor.commit_block(ledger_info))
            .unwrap()
            .unwrap();
    }
    root_hash
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_executor_two_branches(
        a_size in 0..30u64,
        b_size in 0..30u64,
        c_size in 0..30u64,
        amount in any::<u32>(),
    ) {
        // Genesis -> A -> B
        //            |
        //            └--> C
        let block_a = TestBlock::new(0..a_size, amount, *GENESIS_BLOCK_ID, gen_block_id(1));
        let block_b = TestBlock::new(0..b_size, amount, gen_block_id(1), gen_block_id(2));
        let block_c = TestBlock::new(0..c_size, amount, gen_block_id(1), gen_block_id(3));

        // Execute block A, B and C. Hold all results in memory.
        let executor = TestExecutor::new();

        let response_a = block_on(executor.execute_block(
            block_a.txns.clone(), block_a.parent_id, block_a.id,
        )).unwrap().unwrap();
        prop_assert_eq!(response_a.version(), a_size);
        let response_b = block_on(executor.execute_block(
            block_b.txns.clone(), block_b.parent_id, block_b.id,
        )).unwrap().unwrap();
        prop_assert_eq!(response_b.version(), a_size + b_size);
        let response_c = block_on(executor.execute_block(
            block_c.txns.clone(), block_c.parent_id, block_c.id,
        )).unwrap().unwrap();
        prop_assert_eq!(response_c.version(), a_size + c_size);

        let root_hash_a = response_a.root_hash();
        let root_hash_b = response_b.root_hash();
        let root_hash_c = response_c.root_hash();

        // Execute block A and B. Execute and commit one transaction at a time.
        let expected_root_hash_a = run_transactions_naive(block_a.txns.clone());
        prop_assert_eq!(root_hash_a, expected_root_hash_a);

        let expected_root_hash_b = run_transactions_naive({
            let mut txns = vec![];
            txns.extend(block_a.txns.iter().cloned());
            txns.extend(block_b.txns.iter().cloned());
            txns
        });
        prop_assert_eq!(root_hash_b, expected_root_hash_b);

        let expected_root_hash_c = run_transactions_naive({
            let mut txns = vec![];
            txns.extend(block_a.txns.iter().cloned());
            txns.extend(block_c.txns.iter().cloned());
            txns
        });
        prop_assert_eq!(root_hash_c, expected_root_hash_c);
    }

    #[test]
    fn test_executor_restart(a_size in 0..30u64, b_size in 0..30u64, amount in any::<u32>()) {
        let block_a = TestBlock::new(0..a_size, amount, *GENESIS_BLOCK_ID, gen_block_id(1));
        let block_b = TestBlock::new(0..b_size, amount, gen_block_id(1), gen_block_id(2));

        let mut config = get_config();
        let (storage_server, shutdown_receiver) = create_storage_server(&mut config);

        // First execute and commit one block, then destroy executor.
        {
            let executor = create_executor(&config);
            let response_a = block_on(executor.execute_block(
                block_a.txns.clone(), block_a.parent_id, block_a.id,
            )).unwrap().unwrap();
            let root_hash = response_a.root_hash();
            let ledger_info = gen_ledger_info(block_a.txns.len() as u64, root_hash, block_a.id, 1);
            block_on(executor.commit_block(ledger_info)).unwrap().unwrap();
        }

        // Now we construct a new executor and run one more block.
        let root_hash = {
            let executor = create_executor(&config);
            let response_b = block_on(executor.execute_block(
                block_b.txns.clone(), block_b.parent_id, block_b.id,
            )).unwrap().unwrap();
            let root_hash = response_b.root_hash();
            let ledger_info = gen_ledger_info(
                (block_a.txns.len() + block_b.txns.len()) as u64,
                root_hash,
                block_b.id,
                2,
            );
            block_on(executor.commit_block(ledger_info)).unwrap().unwrap();
            root_hash
        };

        let expected_root_hash = run_transactions_naive({
            let mut txns = vec![];
            txns.extend(block_a.txns.iter().cloned());
            txns.extend(block_b.txns.iter().cloned());
            txns
        });
        prop_assert_eq!(root_hash, expected_root_hash);

        drop(storage_server);
        shutdown_receiver.recv().unwrap();
    }
}
