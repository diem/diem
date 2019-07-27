// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides reusable helpers in tests.

use super::*;
use crate::mock_genesis::{db_with_mock_genesis, GENESIS_INFO};
use crypto::hash::CryptoHash;
use itertools::zip_eq;
use proptest::{collection::vec, prelude::*};
use types::{ledger_info::LedgerInfo, proptest_types::arb_txn_to_commit_batch};

fn to_blocks_to_commit(
    txns_to_commit_vec: Vec<Vec<TransactionToCommit>>,
    partial_ledger_info_with_sigs_vec: Vec<LedgerInfoWithSignatures>,
) -> Result<Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>> {
    // Use temporary LibraDB and STORE LEVEL APIs to calculate hashes on a per transaction basis.
    // Result is used to test the batch PUBLIC API for saving everything, i.e. `save_transactions()`
    let tmp_dir = tempfile::tempdir()?;
    let db = db_with_mock_genesis(&tmp_dir)?;

    let genesis_txn_info = GENESIS_INFO.0.clone();
    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_ledger_info = genesis_ledger_info_with_sigs.ledger_info();
    let mut cur_state_root_hash = genesis_txn_info.state_root_hash();
    let mut cur_ver = 0;
    let mut cur_txn_accu_hash = genesis_ledger_info.transaction_accumulator_hash();
    let blocks_to_commit = zip_eq(txns_to_commit_vec, partial_ledger_info_with_sigs_vec)
        .map(|(txns_to_commit, partial_ledger_info_with_sigs)| {
            for txn_to_commit in txns_to_commit.iter() {
                cur_ver += 1;
                let mut cs = ChangeSet::new();

                let txn_hash = txn_to_commit.signed_txn().hash();
                let state_root_hash = db.state_store.put_account_state_sets(
                    vec![txn_to_commit.account_states().clone()],
                    cur_ver,
                    cur_state_root_hash,
                    &mut cs,
                )?[0];
                let event_root_hash =
                    db.event_store
                        .put_events(cur_ver, txn_to_commit.events(), &mut cs)?;

                let txn_info = TransactionInfo::new(
                    txn_hash,
                    state_root_hash,
                    event_root_hash,
                    txn_to_commit.gas_used(),
                );
                let txn_accu_hash =
                    db.ledger_store
                        .put_transaction_infos(cur_ver, &[txn_info], &mut cs)?;
                db.db.write_schemas(cs.batch)?;

                cur_state_root_hash = state_root_hash;
                cur_txn_accu_hash = txn_accu_hash;
            }

            let ledger_info = LedgerInfo::new(
                cur_ver,
                cur_txn_accu_hash,
                partial_ledger_info_with_sigs
                    .ledger_info()
                    .consensus_data_hash(),
                partial_ledger_info_with_sigs
                    .ledger_info()
                    .consensus_block_id(),
                partial_ledger_info_with_sigs.ledger_info().epoch_num(),
                partial_ledger_info_with_sigs
                    .ledger_info()
                    .timestamp_usecs(),
            );
            let ledger_info_with_sigs = LedgerInfoWithSignatures::new(
                ledger_info,
                partial_ledger_info_with_sigs.signatures().clone(),
            );
            Ok((txns_to_commit, ledger_info_with_sigs))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(blocks_to_commit)
}

/// This returns a [`proptest`](https://altsysrq.github.io/proptest-book/intro.html)
/// [`Strategy`](https://docs.rs/proptest/0/proptest/strategy/trait.Strategy.html) that yields an
/// arbitrary number of arbitrary batches of transactions to commit.
///
/// It is used in tests for both transaction block committing during normal running and
/// transaction syncing during start up.
pub fn arb_blocks_to_commit(
) -> impl Strategy<Value = Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>> {
    vec(0..3usize, 1..10usize)
        .prop_flat_map(|batch_sizes| {
            let total_txns = batch_sizes.iter().sum();
            let total_batches = batch_sizes.len();
            (
                Just(batch_sizes),
                arb_txn_to_commit_batch(3, total_txns),
                vec(
                    any_with::<LedgerInfoWithSignatures>((1..3).into()),
                    total_batches,
                ),
            )
        })
        .prop_map(
            |(batch_sizes, all_txns_to_commit, partial_ledger_info_with_sigs_vec)| {
                // split txns_to_commit to batches
                let txns_to_commit_batches = batch_sizes
                    .iter()
                    .scan(0, |end, batch_size| {
                        *end += batch_size;
                        Some(all_txns_to_commit[*end - batch_size..*end].to_vec())
                    })
                    .collect::<Vec<_>>();
                to_blocks_to_commit(txns_to_commit_batches, partial_ledger_info_with_sigs_vec)
                    .unwrap()
            },
        )
}
