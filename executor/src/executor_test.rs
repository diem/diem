// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    mock_vm::{
        encode_mint_transaction, encode_transfer_transaction, MockVM, DISCARD_STATUS, KEEP_STATUS,
    },
    Executor, OP_COUNTERS,
};
use config_builder;
use libra_config::config::NodeConfig;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_types::block_info::BlockInfo;
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
    transaction::{Transaction, TransactionListWithProof, Version},
};
use proptest::prelude::*;
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use std::{collections::BTreeMap, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use tokio::runtime::Runtime;

fn create_storage_server(config: &mut NodeConfig) -> Runtime {
    start_storage_service(&config)
}

fn create_executor(config: &NodeConfig) -> (Executor<MockVM>, ExecutedTrees) {
    let mut rt = Runtime::new().unwrap();
    let read_client = Arc::new(StorageReadServiceClient::new(
        "localhost",
        config.storage.port,
    ));
    let write_client = Arc::new(StorageWriteServiceClient::new(
        "localhost",
        config.storage.port,
    ));
    let executor = Executor::new(read_client, write_client, config);
    let read_client = Arc::new(StorageReadServiceClient::new(
        "localhost",
        config.storage.port,
    ));
    let startup_info = rt
        .block_on(read_client.get_startup_info())
        .expect("unable to read ledger info from storage")
        .expect("startup info is None");
    let root_executed_trees = ExecutedTrees::from(startup_info.committed_tree_state);
    (executor, root_executed_trees)
}

fn execute_and_commit_block(
    executor: &TestExecutor,
    committed_trees: &mut ExecutedTrees,
    txn_index: u64,
) {
    let txn = encode_mint_transaction(gen_address(txn_index), 100);
    let parent_block_id = match txn_index {
        0 => *PRE_GENESIS_BLOCK_ID,
        x => gen_block_id(x),
    };
    let id = gen_block_id(txn_index + 1);

    let output = executor
        .execute_block(
            vec![txn.clone()],
            &committed_trees,
            &committed_trees,
            parent_block_id,
            id,
        )
        .unwrap();
    assert_eq!(output.version().unwrap(), txn_index + 1);

    let ledger_info = gen_ledger_info(txn_index + 1, output.accu_root(), id, txn_index + 1);
    let new_trees = output.executed_trees().clone();
    executor
        .commit_blocks(
            vec![(vec![txn], Arc::new(output))],
            ledger_info,
            &committed_trees,
        )
        .unwrap();
    *committed_trees = new_trees;
}

struct TestExecutor {
    // The config is kept around because it owns the temp dir used in the test.
    _config: NodeConfig,
    storage_server: Option<Runtime>,
    executor: Executor<MockVM>,
}

impl TestExecutor {
    fn new() -> (TestExecutor, ExecutedTrees) {
        let (mut config, _) = config_builder::test_config();
        let storage_server = create_storage_server(&mut config);
        let (executor, committed_trees) = create_executor(&config);

        (
            TestExecutor {
                _config: config,
                storage_server: Some(storage_server),
                executor,
            },
            committed_trees,
        )
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
        BlockInfo::new(
            1,
            0,
            commit_block_id,
            root_hash,
            version,
            timestamp_usecs,
            None,
        ),
        HashValue::zero(),
    );
    LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new())
}

#[test]
fn test_executor_status() {
    let (executor, committed_trees) = TestExecutor::new();

    let txn0 = encode_mint_transaction(gen_address(0), 100);
    let txn1 = encode_mint_transaction(gen_address(1), 100);
    let txn2 = encode_transfer_transaction(gen_address(0), gen_address(1), 500);

    let parent_block_id = *PRE_GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);

    let output = executor
        .execute_block(
            vec![txn0, txn1, txn2],
            &committed_trees,
            &committed_trees,
            parent_block_id,
            block_id,
        )
        .unwrap();

    assert_eq!(
        vec![
            KEEP_STATUS.clone(),
            KEEP_STATUS.clone(),
            DISCARD_STATUS.clone()
        ],
        *output.state_compute_result().status()
    );
}

#[test]
fn test_executor_one_block() {
    let (executor, committed_trees) = TestExecutor::new();

    let parent_block_id = *PRE_GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);
    let version = 100;

    let txns = (0..version)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .collect::<Vec<_>>();
    let output = executor
        .execute_block(
            txns.clone(),
            &committed_trees,
            &committed_trees,
            parent_block_id,
            block_id,
        )
        .unwrap();
    assert_eq!(output.version().unwrap(), 100);

    let ledger_info = gen_ledger_info(version, output.accu_root(), block_id, 1);
    executor
        .commit_blocks(
            vec![(txns, Arc::new(output))],
            ledger_info,
            &committed_trees,
        )
        .unwrap();
}

#[test]
fn test_executor_multiple_blocks() {
    let (executor, mut committed_trees) = TestExecutor::new();

    for i in 0..100 {
        execute_and_commit_block(&executor, &mut committed_trees, i)
    }
}

#[test]
fn test_executor_two_blocks_with_failed_txns() {
    let (executor, committed_trees) = TestExecutor::new();

    let block1_id = gen_block_id(1);
    let block2_id = gen_block_id(2);
    let block1_txns = (0..50)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .collect::<Vec<_>>();
    let block2_txns = (0..50)
        .map(|i| {
            if i % 2 == 0 {
                encode_mint_transaction(gen_address(i + 50), 100)
            } else {
                encode_transfer_transaction(gen_address(i), gen_address(i + 1), 500)
            }
        })
        .collect::<Vec<_>>();
    let output1 = executor
        .execute_block(
            block1_txns.clone(),
            &committed_trees,
            &committed_trees,
            *PRE_GENESIS_BLOCK_ID,
            block1_id,
        )
        .unwrap();
    let output2 = executor
        .execute_block(
            block2_txns.clone(),
            output1.executed_trees(),
            &committed_trees,
            block1_id,
            block2_id,
        )
        .unwrap();
    let ledger_info = gen_ledger_info(75, output2.accu_root(), block2_id, 1);
    executor
        .commit_blocks(
            vec![
                (block1_txns, Arc::new(output1)),
                (block2_txns, Arc::new(output2)),
            ],
            ledger_info,
            &committed_trees,
        )
        .unwrap();
}

#[test]
fn test_executor_execute_same_block_multiple_times() {
    let parent_block_id = *PRE_GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);
    let version = 100;

    let txns: Vec<_> = (0..version)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .collect();

    let (executor, committed_trees) = TestExecutor::new();
    let mut responses = vec![];
    for _i in 0..100 {
        let output = executor
            .execute_block(
                txns.clone(),
                &committed_trees,
                &committed_trees,
                parent_block_id,
                block_id,
            )
            .unwrap();
        responses.push(output.state_compute_result());
    }
    responses.dedup();
    assert_eq!(responses.len(), 1);
}

rusty_fork_test! {
    #[test]
    fn test_num_accounts_created_counter() {
        let (executor, mut committed_trees) = TestExecutor::new();
        for i in 0..20 {
            execute_and_commit_block(&executor, &mut committed_trees, i);
            assert_eq!(OP_COUNTERS.counter("num_accounts").get() as u64, i + 1);
        }
    }
}

/// Generates a list of `TransactionListWithProof`s according to the given ranges.
fn create_transaction_chunks(
    chunk_ranges: Vec<std::ops::Range<Version>>,
) -> (Vec<TransactionListWithProof>, LedgerInfoWithSignatures) {
    let mut rt = Runtime::new().unwrap();
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
    let (mut config, _) = config_builder::test_config();
    let storage_server = create_storage_server(&mut config);
    let (executor, root_trees) = create_executor(&config);

    let mut txns = vec![];
    for i in 1..chunk_ranges.last().unwrap().end {
        let txn = encode_mint_transaction(gen_address(i), 100);
        txns.push(txn);
    }
    let id = gen_block_id(1);

    let output = executor
        .execute_block(
            txns.clone(),
            &root_trees,
            &root_trees,
            *GENESIS_BLOCK_ID,
            id,
        )
        .unwrap();
    let ledger_version = txns.len() as u64;
    let ledger_info = gen_ledger_info(ledger_version, output.accu_root(), id, 1);
    executor
        .commit_blocks(
            vec![(txns, Arc::new(output))],
            ledger_info.clone(),
            &root_trees,
        )
        .unwrap();

    let storage_client = StorageReadServiceClient::new("localhost", config.storage.port);

    let batches: Vec<_> = chunk_ranges
        .into_iter()
        .map(|range| {
            rt.block_on(storage_client.get_transactions(
                range.start,
                range.end - range.start,
                ledger_version,
                false, /* fetch_events */
            ))
            .unwrap()
        })
        .collect();

    drop(storage_server);

    (batches, ledger_info)
}

#[test]
fn test_executor_execute_and_commit_chunk() {
    let mut rt = Runtime::new().unwrap();
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
    let (mut config, _) = config_builder::test_config();
    let storage_server = create_storage_server(&mut config);
    let (executor, mut committed_trees) = create_executor(&config);
    let storage_client = StorageReadServiceClient::new("localhost", config.storage.port);

    // Execute the first chunk. After that we should still get the genesis ledger info from DB.
    executor
        .execute_and_commit_chunk(
            chunks[0].clone(),
            ledger_info.clone(),
            None,
            &mut committed_trees,
        )
        .unwrap();
    let (_, li, _, _) = rt
        .block_on(storage_client.update_to_latest_ledger(0, vec![]))
        .unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *PRE_GENESIS_BLOCK_ID);

    // Execute the second chunk. After that we should still get the genesis ledger info from DB.
    executor
        .execute_and_commit_chunk(
            chunks[1].clone(),
            ledger_info.clone(),
            None,
            &mut committed_trees,
        )
        .unwrap();
    let (_, li, _, _) = rt
        .block_on(storage_client.update_to_latest_ledger(0, vec![]))
        .unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *PRE_GENESIS_BLOCK_ID);

    // Execute an empty chunk. After that we should still get the genesis ledger info from DB.
    executor
        .execute_and_commit_chunk(
            TransactionListWithProof::new_empty(),
            ledger_info.clone(),
            None,
            &mut committed_trees,
        )
        .unwrap();
    let (_, li, _, _) = rt
        .block_on(storage_client.update_to_latest_ledger(0, vec![]))
        .unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *PRE_GENESIS_BLOCK_ID);

    // Execute the second chunk again. After that we should still get the same thing.
    executor
        .execute_and_commit_chunk(
            chunks[1].clone(),
            ledger_info.clone(),
            None,
            &mut committed_trees,
        )
        .unwrap();
    let (_, li, _, _) = rt
        .block_on(storage_client.update_to_latest_ledger(0, vec![]))
        .unwrap();
    assert_eq!(li.ledger_info().version(), 0);
    assert_eq!(li.ledger_info().consensus_block_id(), *PRE_GENESIS_BLOCK_ID);

    // Execute the third chunk. After that we should get the new ledger info.
    executor
        .execute_and_commit_chunk(
            chunks[2].clone(),
            ledger_info.clone(),
            None,
            &mut committed_trees,
        )
        .unwrap();
    let (_, li, _, _) = rt
        .block_on(storage_client.update_to_latest_ledger(0, vec![]))
        .unwrap();
    assert_eq!(li, ledger_info);

    drop(storage_server);
}

#[test]
fn test_executor_execute_and_commit_chunk_restart() {
    let mut rt = Runtime::new().unwrap();
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

    let (mut config, _) = config_builder::test_config();
    let storage_server = create_storage_server(&mut config);
    let mut synced_trees;

    // First we simulate syncing the first chunk of transactions.
    {
        let (executor, mut committed_trees) = create_executor(&config);
        let storage_client = StorageReadServiceClient::new("localhost", config.storage.port);

        executor
            .execute_and_commit_chunk(
                chunks[0].clone(),
                ledger_info.clone(),
                None,
                &mut committed_trees,
            )
            .unwrap();
        synced_trees = committed_trees;
        let (_, li, _, _) = rt
            .block_on(storage_client.update_to_latest_ledger(0, vec![]))
            .unwrap();
        assert_eq!(li.ledger_info().version(), 0);
        assert_eq!(li.ledger_info().consensus_block_id(), *PRE_GENESIS_BLOCK_ID);
    }

    // Then we restart executor and resume to the next chunk.
    {
        let (executor, _) = create_executor(&config);
        let storage_client = StorageReadServiceClient::new("localhost", config.storage.port);

        executor
            .execute_and_commit_chunk(
                chunks[1].clone(),
                ledger_info.clone(),
                None,
                &mut synced_trees,
            )
            .unwrap();
        let (_, li, _, _) = rt
            .block_on(storage_client.update_to_latest_ledger(0, vec![]))
            .unwrap();
        assert_eq!(li, ledger_info);
    }

    drop(storage_server);
}

struct TestBlock {
    txns: Vec<Transaction>,
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
fn run_transactions_naive(transactions: Vec<Transaction>) -> HashValue {
    let (executor, mut committed_trees) = TestExecutor::new();
    let mut iter = transactions.into_iter();
    let first_txn = iter.next().map_or(vec![], |txn| vec![txn]);
    let first_id = gen_block_id(1);
    let mut output = executor
        .execute_block(
            first_txn.clone(),
            &committed_trees,
            &committed_trees,
            *GENESIS_BLOCK_ID,
            gen_block_id(1),
        )
        .unwrap();
    let mut root_hash = output.accu_root();
    let ledger_info = gen_ledger_info(first_txn.len() as u64, root_hash, first_id, 0);
    let new_trees = output.executed_trees().clone();
    executor
        .commit_blocks(
            vec![(first_txn, Arc::new(output))],
            ledger_info,
            &committed_trees,
        )
        .unwrap();
    committed_trees = new_trees;

    for (i, txn) in iter.enumerate() {
        let parent_block_id = gen_block_id(i as u64 + 1);
        // when i = 0, id should be 2.
        let id = gen_block_id(i as u64 + 2);
        output = executor
            .execute_block(
                vec![txn.clone()],
                &committed_trees,
                &committed_trees,
                parent_block_id,
                id,
            )
            .unwrap();

        root_hash = output.accu_root();
        let ledger_info = gen_ledger_info(i as u64 + 2, root_hash, id, i as u64 + 1);
        let new_trees = output.executed_trees().clone();
        executor
            .commit_blocks(
                vec![(vec![txn], Arc::new(output))],
                ledger_info,
                &committed_trees,
            )
            .unwrap();
        committed_trees = new_trees;
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
        let block_a = TestBlock::new(0..a_size, amount, *PRE_GENESIS_BLOCK_ID, gen_block_id(1));
        let block_b = TestBlock::new(0..b_size, amount, gen_block_id(1), gen_block_id(2));
        let block_c = TestBlock::new(0..c_size, amount, gen_block_id(1), gen_block_id(3));
        // Execute block A, B and C. Hold all results in memory.
        let (executor, committed_trees) = TestExecutor::new();

        let output_a = executor.execute_block(
            block_a.txns.clone(), &committed_trees, &committed_trees,block_a.parent_id, block_a.id,
        ).unwrap();
        prop_assert_eq!(output_a.version().unwrap(), a_size);
        let output_b = executor.execute_block(
            block_b.txns.clone(), output_a.executed_trees(), &committed_trees, block_b.parent_id, block_b.id,
        ).unwrap();
        prop_assert_eq!(output_b.version().unwrap(), a_size + b_size);
        let output_c = executor.execute_block(
            block_c.txns.clone(), output_a.executed_trees(), &committed_trees, block_c.parent_id, block_c.id,
        ).unwrap();
        prop_assert_eq!(output_c.version().unwrap(), a_size + c_size);

        let root_hash_a = output_a.accu_root();
        let root_hash_b = output_b.accu_root();
        let root_hash_c = output_c.accu_root();

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
        let block_a = TestBlock::new(0..a_size, amount, *PRE_GENESIS_BLOCK_ID, gen_block_id(1));
        let block_b = TestBlock::new(0..b_size, amount, gen_block_id(1), gen_block_id(2));

        let (mut config, _) = config_builder::test_config();
        let storage_server = create_storage_server(&mut config);

        // First execute and commit one block, then destroy executor.
        {
            let (executor, committed_trees) = create_executor(&config);
            let output_a = executor.execute_block(
                block_a.txns.clone(), &committed_trees, &committed_trees, block_a.parent_id, block_a.id,
            ).unwrap();
            let root_hash = output_a.accu_root();
            let ledger_info = gen_ledger_info(block_a.txns.len() as u64, root_hash, block_a.id, 1);
            executor.commit_blocks(vec![(block_a.txns.clone(), Arc::new(output_a))],
                                   ledger_info,
                                   &committed_trees)
                .unwrap();
        }

        // Now we construct a new executor and run one more block.
        let root_hash = {
            let (executor, committed_trees) = create_executor(&config);
            let output_b = executor.execute_block(
                block_b.txns.clone(), &committed_trees, &committed_trees, block_b.parent_id, block_b.id,
            ).unwrap();
            let root_hash = output_b.accu_root();
            let ledger_info = gen_ledger_info(
                (block_a.txns.len() + block_b.txns.len()) as u64,
                root_hash,
                block_b.id,
                2,
            );
            executor.commit_blocks(vec![(block_b.txns.clone(), Arc::new(output_b))],
                                   ledger_info,
                                   &committed_trees)
                .unwrap();
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
    }

    #[test]
    fn test_idempotent_commits(chunk_size in 1..30u64, overlap_size in 1..30u64, num_new_txns in 1..30u64) {
        let (chunk_start, chunk_end) = (1, chunk_size + 1);
        let (overlap_start, overlap_end) = (chunk_size + 1, chunk_size + overlap_size + 1);
        let (mut chunks, ledger_info) =
            create_transaction_chunks(vec![
                chunk_start..chunk_end,
                overlap_start..overlap_end
            ]);

        let (mut config, _) = config_builder::test_config();
        let storage_server = create_storage_server(&mut config);
        let (executor, committed_trees) = create_executor(&config);

        let overlap_txn_list_with_proof = chunks.pop().unwrap();
        let txn_list_with_proof_to_commit = chunks.pop().unwrap();
        let mut first_block_txns = txn_list_with_proof_to_commit.transactions.clone();

        let mut synced_trees = committed_trees.clone();
        // Commit the first chunk without committing the ledger info.
        executor.execute_and_commit_chunk(txn_list_with_proof_to_commit, ledger_info, None, &mut synced_trees)
            .unwrap();

        let parent_block_id = HashValue::zero();
        let first_block_id = gen_block_id(1);
        first_block_txns.extend(overlap_txn_list_with_proof.transactions);
        let second_block_txns = ((chunk_size + overlap_size + 1..=chunk_size + overlap_size + num_new_txns)
                             .map(|i| encode_mint_transaction(gen_address(i), 100))).collect::<Vec<_>>();

        let output1 = executor.execute_block(
            first_block_txns.clone(),
            &committed_trees,
            &committed_trees,
            parent_block_id,
            first_block_id,
        ).unwrap();

        let second_block_id = gen_block_id(2);
        let output2 = executor.execute_block(
            second_block_txns.clone(),
            output1.executed_trees(),
            &committed_trees,
            first_block_id,
            second_block_id,
        ).unwrap();

        let version = chunk_size + overlap_size + num_new_txns;
        prop_assert_eq!(output2.version().unwrap(), version);

        let ledger_info = gen_ledger_info(version, output2.accu_root(), second_block_id, 1);
        executor.commit_blocks(
            vec![(first_block_txns, Arc::new(output1)),
                 (second_block_txns, Arc::new(output2))],
            ledger_info,
            &synced_trees,
        ).unwrap();

        drop(storage_server);
    }
}
