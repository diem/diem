// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides reusable helpers in tests.

use super::*;
use crate::mock_genesis::{db_with_mock_genesis, GENESIS_INFO};
use crypto::{ed25519::*, hash::CryptoHash};
use proptest::{collection::vec, prelude::*};
use tools::tempdir::TempPath;
use types::{
    ledger_info::LedgerInfo,
    proptest_types::{AccountInfoUniverse, TransactionToCommitGen},
};

fn to_blocks_to_commit(
    partial_blocks: Vec<(
        Vec<TransactionToCommit>,
        LedgerInfoWithSignatures<Ed25519Signature>,
    )>,
) -> Result<
    Vec<(
        Vec<TransactionToCommit>,
        LedgerInfoWithSignatures<Ed25519Signature>,
    )>,
> {
    // Use temporary LibraDB and STORE LEVEL APIs to calculate hashes on a per transaction basis.
    // Result is used to test the batch PUBLIC API for saving everything, i.e. `save_transactions()`
    let tmp_dir = TempPath::new();
    let db = db_with_mock_genesis(&tmp_dir.path())?;

    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_ledger_info = genesis_ledger_info_with_sigs.ledger_info();
    let mut cur_ver = 0;
    let mut cur_txn_accu_hash = genesis_ledger_info.transaction_accumulator_hash();
    let blocks_to_commit = partial_blocks
        .into_iter()
        .map(|(txns_to_commit, partial_ledger_info_with_sigs)| {
            for txn_to_commit in txns_to_commit.iter() {
                cur_ver += 1;
                let mut cs = ChangeSet::new();

                let txn_hash = txn_to_commit.signed_txn().hash();
                let state_root_hash = db.state_store.put_account_state_sets(
                    vec![txn_to_commit.account_states().clone()],
                    cur_ver,
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
                partial_ledger_info_with_sigs
                    .ledger_info()
                    .next_validator_set()
                    .cloned(),
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

prop_compose! {
    /// This returns a [`proptest`](https://altsysrq.github.io/proptest-book/intro.html)
    /// [`Strategy`](https://docs.rs/proptest/0/proptest/strategy/trait.Strategy.html) that yields an
    /// arbitrary number of arbitrary batches of transactions to commit.
    ///
    /// It is used in tests for both transaction block committing during normal running and
    /// transaction syncing during start up.
    pub fn arb_blocks_to_commit()(
        mut universe in any_with::<AccountInfoUniverse>(5).no_shrink(),
        batches in vec(
            (
                vec(any::<TransactionToCommitGen>(), 0..=2),
                any::<LedgerInfoWithSignatures<Ed25519Signature>>()
            ),
            1..10,
        ),
    ) ->
        Vec<(
            Vec<TransactionToCommit>,
            LedgerInfoWithSignatures<Ed25519Signature>,
        )>
    {
        let partial_blocks = batches
            .into_iter()
            .map(|(txn_gens, partial_ledger_info)| {
                (
                    txn_gens
                        .into_iter()
                        .map(|gen| gen.materialize(&mut universe))
                        .collect(),
                    partial_ledger_info,
                )
            })
            .collect();

        to_blocks_to_commit(partial_blocks).unwrap()
    }
}
