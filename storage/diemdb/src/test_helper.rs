// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

///! This module provides reusable helpers in tests.
use super::*;
use diem_crypto::hash::{CryptoHash, EventAccumulatorHasher, TransactionAccumulatorHasher};
use diem_types::{
    account_address::HashAccountAddress,
    ledger_info::LedgerInfoWithSignatures,
    proof::accumulator::InMemoryAccumulator,
    proptest_types::{AccountInfoUniverse, BlockGen},
};
use executor_types::ProofReader;
use proptest::{collection::vec, prelude::*};
use scratchpad::SparseMerkleTree;

prop_compose! {
    /// This returns a [`proptest`](https://altsysrq.github.io/proptest-book/intro.html)
    /// [`Strategy`](https://docs.rs/proptest/0/proptest/strategy/trait.Strategy.html) that yields an
    /// arbitrary number of arbitrary batches of transactions to commit.
    ///
    /// It is used in tests for both transaction block committing during normal running and
    /// transaction syncing during start up.
    fn arb_blocks_to_commit_impl(
        num_accounts: usize,
        max_user_txns_per_block: usize,
        max_blocks: usize,
    )(
        mut universe in any_with::<AccountInfoUniverse>(num_accounts).no_shrink(),
        block_gens in vec(any_with::<BlockGen>(max_user_txns_per_block), 1..=max_blocks),
    ) -> Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)> {
        type EventAccumulator = InMemoryAccumulator<EventAccumulatorHasher>;
        type TxnAccumulator = InMemoryAccumulator<TransactionAccumulatorHasher>;

        let mut smt = SparseMerkleTree::<AccountStateBlob>::default().freeze();
        let mut txn_accumulator = TxnAccumulator::new_empty();
        let mut result = Vec::new();

        for block_gen in block_gens {
            let (mut txns_to_commit, mut ledger_info) = block_gen.materialize(&mut universe);

            // make real txn_info's
            for txn in txns_to_commit.iter_mut() {
                let placeholder_txn_info = txn.transaction_info();

                // calculate event root hash
                let event_hashes: Vec<_> = txn.events().iter().map(CryptoHash::hash).collect();
                let event_root_hash = EventAccumulator::from_leaves(&event_hashes).root_hash();

                // calcualte state checkpoint hash
                let state_checkpoint_hash = if txn.account_states().is_empty() {
                    None
                } else {
                    let updates: Vec<_> = txn.account_states().iter().map(|(addr, blob)| {
                            ( HashAccountAddress::hash(addr), blob )
                    }).collect();

                    smt = smt.batch_update(updates, &ProofReader::new_empty()).unwrap();

                    Some(smt.root_hash())
                };

                let txn_info = TransactionInfo::new(
                    txn.transaction().hash(),
                    placeholder_txn_info.state_change_hash(),
                    event_root_hash,
                    state_checkpoint_hash,
                    placeholder_txn_info.gas_used(),
                    placeholder_txn_info.status().clone(),
                );
                txn_accumulator = txn_accumulator.append(&[txn_info.hash()]);
                txn.set_transaction_info(txn_info);
            }

            // updated ledger info with real root hash and sign
            ledger_info.set_executed_state_id(txn_accumulator.root_hash());
            let validator_set = universe.get_validator_set(ledger_info.epoch());
            let signatures = validator_set
                .iter()
                .map(|signer| (signer.author(), signer.sign(&ledger_info)))
                .collect();
            let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, signatures);

            result.push((txns_to_commit, ledger_info_with_sigs))
        }
        result
    }
}

pub fn arb_blocks_to_commit(
) -> impl Strategy<Value = Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>> {
    arb_blocks_to_commit_impl(
        5,  /* num_accounts */
        2,  /* max_user_txn_per_block */
        10, /* max_blocks */
    )
}
