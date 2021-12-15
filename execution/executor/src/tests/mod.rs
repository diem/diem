// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeMap, iter::once};

use proptest::prelude::*;

use diem_crypto::HashValue;
use diem_state_view::StateViewId;
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::definition::LeafCount,
    transaction::{Transaction, TransactionListWithProof, TransactionStatus, Version},
};
use diemdb::DiemDB;
use executor_types::{BlockExecutorTrait, ChunkExecutorTrait, TransactionReplayer};
use storage_interface::DbReaderWriter;

use crate::{
    block_executor::BlockExecutor,
    chunk_executor::ChunkExecutor,
    components::{apply_chunk_output::IntoLedgerView, chunk_output::ChunkOutput},
    db_bootstrapper::{generate_waypoint, maybe_bootstrap},
    mock_vm::{
        encode_mint_transaction, encode_reconfiguration_transaction, encode_transfer_transaction,
        MockVM, DISCARD_STATUS, KEEP_STATUS,
    },
};

mod chunk_executor_tests;

fn execute_and_commit_block(
    executor: &TestExecutor,
    parent_block_id: HashValue,
    txn_index: u64,
) -> HashValue {
    let txn = encode_mint_transaction(gen_address(txn_index), 100);
    let id = gen_block_id(txn_index + 1);
    let version = (txn_index + 1) * 2;

    let output = executor
        .execute_block(
            (id, vec![txn, Transaction::StateCheckpoint]),
            parent_block_id,
        )
        .unwrap();
    assert_eq!(output.version(), version);

    let ledger_info = gen_ledger_info(version, output.root_hash(), id, txn_index + 1);
    executor.commit_blocks(vec![id], ledger_info).unwrap();
    id
}

struct TestExecutor {
    _path: diem_temppath::TempPath,
    db: DbReaderWriter,
    executor: BlockExecutor<MockVM>,
}

impl TestExecutor {
    fn new() -> TestExecutor {
        let path = diem_temppath::TempPath::new();
        path.create_as_dir().unwrap();
        let db = DbReaderWriter::new(DiemDB::new_for_test(path.path()));
        let genesis = vm_genesis::test_genesis_transaction();
        let waypoint = generate_waypoint::<MockVM>(&db, &genesis).unwrap();
        maybe_bootstrap::<MockVM>(&db, &genesis, waypoint).unwrap();
        let executor = BlockExecutor::new(db.clone());

        TestExecutor {
            _path: path,
            db,
            executor,
        }
    }
}

impl std::ops::Deref for TestExecutor {
    type Target = BlockExecutor<MockVM>;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}

impl std::ops::DerefMut for TestExecutor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.executor
    }
}

fn gen_address(index: u64) -> AccountAddress {
    let bytes = index.to_be_bytes();
    let mut buf = [0; AccountAddress::LENGTH];
    buf[AccountAddress::LENGTH - 8..].copy_from_slice(&bytes);
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
    let executor = TestExecutor::new();
    let parent_block_id = executor.committed_block_id();
    let block_id = gen_block_id(1);

    let txn0 = encode_mint_transaction(gen_address(0), 100);
    let txn1 = encode_mint_transaction(gen_address(1), 100);
    let txn2 = encode_transfer_transaction(gen_address(0), gen_address(1), 500);

    let output = executor
        .execute_block(
            (
                block_id,
                vec![txn0, txn1, txn2, Transaction::StateCheckpoint],
            ),
            parent_block_id,
        )
        .unwrap();

    assert_eq!(
        &vec![
            KEEP_STATUS.clone(),
            KEEP_STATUS.clone(),
            DISCARD_STATUS.clone(),
            KEEP_STATUS.clone(),
        ],
        output.compute_status()
    );
}

#[test]
fn test_executor_one_block() {
    let executor = TestExecutor::new();
    let parent_block_id = executor.committed_block_id();
    let block_id = gen_block_id(1);

    let txns = (0..100)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .chain(once(Transaction::StateCheckpoint))
        .collect::<Vec<_>>();
    let output = executor
        .execute_block((block_id, txns), parent_block_id)
        .unwrap();
    assert_eq!(output.version(), 101);
    let block_root_hash = output.root_hash();

    let ledger_info = gen_ledger_info(101, block_root_hash, block_id, 1);
    executor.commit_blocks(vec![block_id], ledger_info).unwrap();
}

#[test]
fn test_executor_multiple_blocks() {
    let executor = TestExecutor::new();
    let mut parent_block_id = executor.committed_block_id();

    for i in 0..100 {
        parent_block_id = execute_and_commit_block(&executor, parent_block_id, i);
    }
}

#[test]
fn test_executor_two_blocks_with_failed_txns() {
    let executor = TestExecutor::new();
    let parent_block_id = executor.committed_block_id();

    let block1_id = gen_block_id(1);
    let block2_id = gen_block_id(2);

    let block1_txns = (0..50)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .chain(once(Transaction::StateCheckpoint))
        .collect::<Vec<_>>();
    let block2_txns = (0..50)
        .map(|i| {
            if i % 2 == 0 {
                encode_mint_transaction(gen_address(i + 50), 100)
            } else {
                encode_transfer_transaction(gen_address(i), gen_address(i + 1), 500)
            }
        })
        .chain(once(Transaction::StateCheckpoint))
        .collect::<Vec<_>>();
    let _output1 = executor
        .execute_block((block1_id, block1_txns), parent_block_id)
        .unwrap();
    let output2 = executor
        .execute_block((block2_id, block2_txns), block1_id)
        .unwrap();
    let ledger_info = gen_ledger_info(77, output2.root_hash(), block2_id, 1);
    executor
        .commit_blocks(vec![block1_id, block2_id], ledger_info)
        .unwrap();
}

#[test]
fn test_executor_commit_twice() {
    let executor = TestExecutor::new();
    let parent_block_id = executor.committed_block_id();
    let block1_txns = (0..5)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .chain(once(Transaction::StateCheckpoint))
        .collect::<Vec<_>>();
    let block1_id = gen_block_id(1);
    let output1 = executor
        .execute_block((block1_id, block1_txns), parent_block_id)
        .unwrap();
    let ledger_info = gen_ledger_info(6, output1.root_hash(), block1_id, 1);
    executor
        .commit_blocks(vec![block1_id], ledger_info.clone())
        .unwrap();
    // commit with the same ledger info again.
    executor
        .commit_blocks(vec![block1_id], ledger_info)
        .unwrap();
}

#[test]
fn test_executor_execute_same_block_multiple_times() {
    let executor = TestExecutor::new();
    let parent_block_id = executor.committed_block_id();
    let block_id = gen_block_id(1);

    let txns: Vec<_> = (0..100)
        .map(|i| encode_mint_transaction(gen_address(i), 100))
        .chain(once(Transaction::StateCheckpoint))
        .collect();

    let mut responses = vec![];
    for _i in 0..100 {
        let output = executor
            .execute_block((block_id, txns.clone()), parent_block_id)
            .unwrap();
        responses.push(output);
    }
    responses.dedup();
    assert_eq!(responses.len(), 1);
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
    let TestExecutor { executor, .. } = TestExecutor::new();

    let mut txns = vec![];
    for i in 1..(chunk_ranges.last().unwrap().end - 1) {
        let txn = encode_mint_transaction(gen_address(i), 100);
        txns.push(txn);
    }
    txns.push(Transaction::StateCheckpoint);
    let id = gen_block_id(1);

    let output = executor
        .execute_block((id, txns.clone()), executor.committed_block_id())
        .unwrap();
    let ledger_version = txns.len() as u64;
    let ledger_info = gen_ledger_info(ledger_version, output.root_hash(), id, 1);
    executor
        .commit_blocks(vec![id], ledger_info.clone())
        .unwrap();

    let batches: Vec<_> = chunk_ranges
        .into_iter()
        .map(|range| {
            executor
                .db
                .reader
                .get_transactions(
                    range.start,
                    range.end - range.start,
                    ledger_version,
                    false, /* fetch_events */
                )
                .unwrap()
        })
        .collect();

    (batches, ledger_info)
}

#[test]
fn test_noop_block_after_reconfiguration() {
    let executor = TestExecutor::new();
    let mut parent_block_id = executor.committed_block_id();
    let first_txn = encode_reconfiguration_transaction(gen_address(1));
    let first_block_id = gen_block_id(1);
    let output1 = executor
        .execute_block((first_block_id, vec![first_txn]), parent_block_id)
        .unwrap();
    parent_block_id = first_block_id;
    let second_block = TestBlock::new(10, 10, gen_block_id(2));
    let output2 = executor
        .execute_block((second_block.id, second_block.txns), parent_block_id)
        .unwrap();
    assert_eq!(output1.root_hash(), output2.root_hash());
}

struct TestBlock {
    txns: Vec<Transaction>,
    id: HashValue,
}

impl TestBlock {
    fn new(num_user_txns: u64, amount: u32, id: HashValue) -> Self {
        TestBlock {
            txns: (0..num_user_txns)
                .map(|index| encode_mint_transaction(gen_address(index), u64::from(amount)))
                .chain(once(Transaction::StateCheckpoint))
                .collect(),
            id,
        }
    }

    fn len(&self) -> u64 {
        self.txns.len() as u64
    }
}

// Executes a list of transactions by executing and immediately committing one at a time. Returns
// the root hash after all transactions are committed.
fn run_transactions_naive(transactions: Vec<Transaction>) -> HashValue {
    let executor = TestExecutor::new();
    let db = &executor.db;
    let mut ledger_view = db
        .reader
        .get_latest_tree_state()
        .unwrap()
        .into_ledger_view(&db.reader)
        .unwrap();

    for txn in transactions {
        let out = ChunkOutput::by_transaction_execution::<MockVM>(
            vec![txn],
            ledger_view.state_view(&ledger_view, StateViewId::Miscellaneous, db.reader.clone()),
        )
        .unwrap();
        let (executed, _, _) = out.apply_to_ledger(&ledger_view).unwrap();
        db.writer
            .save_transactions(
                &executed.transactions_to_commit().unwrap(),
                ledger_view.txn_accumulator().num_leaves(),
                None,
            )
            .unwrap();
        ledger_view = executed.result_view;
    }
    ledger_view.txn_accumulator().root_hash()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_executor_two_branches(
        a_size in 2..30u64,
        b_size in 2..30u64,
        c_size in 2..30u64,
        amount in any::<u32>(),
    ) {
        // Genesis -> A -> B
        //            |
        //            └--> C
        let block_a = TestBlock::new(a_size, amount, gen_block_id(1));
        let block_b = TestBlock::new(b_size, amount, gen_block_id(2));
        let block_c = TestBlock::new(c_size, amount, gen_block_id(3));
        // Execute block A, B and C. Hold all results in memory.
        let executor = TestExecutor::new();
        let parent_block_id = executor.committed_block_id();

        let output_a = executor.execute_block(
            (block_a.id, block_a.txns.clone()), parent_block_id
        ).unwrap();
        let root_hash_a = output_a.root_hash();
        prop_assert_eq!(output_a.version(), block_a.len());
        let output_b = executor.execute_block((block_b.id, block_b.txns.clone()), block_a.id).unwrap();
        prop_assert_eq!(output_b.version(), block_a.len() + block_b.len());
        let output_c = executor.execute_block((block_c.id, block_c.txns.clone()), block_a.id).unwrap();
        prop_assert_eq!(output_c.version(), block_a.len() + block_c.len());

        let root_hash_b = output_b.root_hash();
        let root_hash_c = output_c.root_hash();

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
    fn test_reconfiguration_with_retry_transaction_status(
        (num_user_txns, reconfig_txn_index) in (10..100u64).prop_flat_map(|num_user_txns| {
            (
                Just(num_user_txns),
                0..num_user_txns-1 // avoid state checkpoint right after reconfig
            )
        })) {
            let block_id = gen_block_id(1);
            let mut block = TestBlock::new(num_user_txns, 10, block_id);
            let num_txns = block.txns.len() as LeafCount;
            block.txns[reconfig_txn_index as usize] = encode_reconfiguration_transaction(gen_address(reconfig_txn_index));
            let executor = TestExecutor::new();

            let parent_block_id = executor.committed_block_id();
            let output = executor.execute_block(
                (block_id, block.txns.clone()), parent_block_id
            ).unwrap();

            // assert: txns after the reconfiguration are with status "Retry"
            let retry_iter = output.compute_status().iter()
            .skip_while(|status| matches!(*status, TransactionStatus::Keep(_)));
            prop_assert_eq!(
                retry_iter.take_while(|status| matches!(*status,TransactionStatus::Retry)).count() as u64,
                num_txns - reconfig_txn_index - 1
            );

            // commit
            let ledger_info = gen_ledger_info(reconfig_txn_index + 1 /* version */, output.root_hash(), block_id, 1 /* timestamp */);
            executor.commit_blocks(vec![block_id], ledger_info).unwrap();
            let parent_block_id = executor.committed_block_id();

            // retry txns after reconfiguration
            let retry_block_id = gen_block_id(2);
            let retry_output = executor.execute_block(
                (retry_block_id, block.txns.iter().skip(reconfig_txn_index as usize + 1).cloned().collect()), parent_block_id
            ).unwrap();
            prop_assert!(retry_output.compute_status().iter().all(|s| matches!(*s, TransactionStatus::Keep(_))));

            // commit
            let ledger_info = gen_ledger_info(num_txns as Version, retry_output.root_hash(), retry_block_id, 12345 /* timestamp */);
            executor.commit_blocks(vec![retry_block_id], ledger_info).unwrap();

            // get txn_infos from db
            let db = executor.db.reader.clone();
            prop_assert_eq!(db.get_latest_version().unwrap(), num_txns as Version);
            let txn_list = db.get_transactions(1 /* start version */, num_txns as u64, num_txns as Version /* ledger version */, false /* fetch events */).unwrap();
            prop_assert_eq!(&block.txns, &txn_list.transactions);
            let txn_infos = txn_list.proof.transaction_infos;

            // replay txns in one batch across epoch boundary,
            // and the replayer should deal with `Retry`s automatically
            let replayer = chunk_executor_tests::TestExecutor::new();
            replayer.executor.replay(block.txns, txn_infos).unwrap();
            replayer.executor.commit().unwrap();
            let replayed_db = replayer.db.reader.clone();
            prop_assert_eq!(
                replayed_db.get_accumulator_root_hash(num_txns).unwrap(),
                db.get_accumulator_root_hash(num_txns).unwrap()
            );
        }

    #[test]
    fn test_executor_restart(a_size in 1..30u64, b_size in 1..30u64, amount in any::<u32>()) {
        let block_a = TestBlock::new(a_size, amount, gen_block_id(1));
        let block_b = TestBlock::new(b_size, amount, gen_block_id(2));

        let TestExecutor { _path, db, executor } = TestExecutor::new();
        let mut parent_block_id;
        let mut root_hash;

        // First execute and commit one block, then destroy executor.
        {
            parent_block_id = executor.committed_block_id();
            let output_a = executor.execute_block(
                (block_a.id, block_a.txns.clone()), parent_block_id
            ).unwrap();
            root_hash = output_a.root_hash();
            let ledger_info = gen_ledger_info(block_a.txns.len() as u64, root_hash, block_a.id, 1);
            executor.commit_blocks(vec![block_a.id], ledger_info).unwrap();
            parent_block_id = block_a.id;
        }

        // Now we construct a new executor and run one more block.
        {
            let executor = BlockExecutor::<MockVM>::new(db);
            let output_b = executor.execute_block((block_b.id, block_b.txns.clone()), parent_block_id).unwrap();
            root_hash = output_b.root_hash();
            let ledger_info = gen_ledger_info(
                (block_a.txns.len() + block_b.txns.len()) as u64,
                root_hash,
                block_b.id,
                2,
            );
            executor.commit_blocks(vec![block_b.id], ledger_info).unwrap();
        };

        let expected_root_hash = run_transactions_naive({
            let mut txns = vec![];
            txns.extend(block_a.txns.iter().cloned());
            txns.extend(block_b.txns.iter().cloned());
            txns
        });
        prop_assert_eq!(root_hash, expected_root_hash);
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

        let overlap_txn_list_with_proof = chunks.pop().unwrap();
        let txn_list_with_proof_to_commit = chunks.pop().unwrap();
        let mut first_block_txns = txn_list_with_proof_to_commit.transactions.clone();

        // Commit the first chunk without committing the ledger info.
        let TestExecutor { _path, db, executor } = TestExecutor::new();
        ChunkExecutor::<MockVM>::new(db).unwrap()
            .execute_and_commit_chunk(txn_list_with_proof_to_commit, &ledger_info, None).unwrap();

        first_block_txns.extend(overlap_txn_list_with_proof.transactions);
        let second_block_txns = (chunk_size + overlap_size + 1..=chunk_size + overlap_size + num_new_txns)
                                 .map(|i| encode_mint_transaction(gen_address(i), 100))
                                .chain(once(Transaction::StateCheckpoint))
                                .collect::<Vec<_>>();

        executor.reset().unwrap();
        let parent_block_id = executor.committed_block_id();
        let first_block_id = gen_block_id(1);
        let _output1 = executor.execute_block(
            (first_block_id, first_block_txns),
            parent_block_id
        ).unwrap();

        let second_block_id = gen_block_id(2);
        let output2 = executor.execute_block(
            (second_block_id, second_block_txns),
            first_block_id,
        ).unwrap();

        let version = chunk_size + overlap_size + num_new_txns + 1;
        prop_assert_eq!(output2.version(), version);

        let ledger_info = gen_ledger_info(version, output2.root_hash(), second_block_id, 1);
        executor.commit_blocks(
            vec![first_block_id, second_block_id],
            ledger_info,
        ).unwrap();
    }
}
