// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    sparse_merkle::node::{NodeHandle, NodeInner, SubTree},
    test_utils::{naive_smt::NaiveSmt, proof_reader::ProofReader},
    SparseMerkleTree,
};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::account_state_blob::AccountStateBlob;
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    sample::Index,
};
use std::{collections::VecDeque, sync::Arc};

type TxnOutput = Vec<(HashValue, AccountStateBlob)>;
type BlockOutput = Vec<TxnOutput>;

#[derive(Debug)]
pub enum Action {
    Commit,
    Execute(BlockOutput),
}

pub fn arb_smt_correctness_case() -> impl Strategy<Value = Vec<Action>> {
    (
        hash_set(any::<HashValue>(), 1..100), // keys
        vec(
            prop_oneof![
                vec(
                    // txns
                    vec(
                        // txn updates
                        (any::<Index>(), any::<Vec<u8>>()),
                        1..4,
                    ),
                    1..10,
                ),
                Just(vec![]),
            ],
            1..10,
        ),
    )
        .prop_map(|(keys, commit_or_execute)| {
            let keys: Vec<_> = keys.into_iter().collect();
            commit_or_execute
                .into_iter()
                .map(|txns| {
                    if txns.is_empty() {
                        Action::Commit
                    } else {
                        Action::Execute(
                            txns.into_iter()
                                .map(|updates| {
                                    updates
                                        .into_iter()
                                        .map(|(k_idx, v)| (*k_idx.get(&keys), v.to_vec().into()))
                                        .collect()
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                })
                .collect::<Vec<_>>()
        })
}

pub fn test_smt_correctness_impl(input: Vec<Action>) {
    let mut naive_q = VecDeque::new();
    naive_q.push_back(NaiveSmt::new::<AccountStateBlob>(&[]));
    let mut serial_q = VecDeque::new();
    serial_q.push_back(SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH));
    let mut batches_q = VecDeque::new();
    batches_q.push_back(SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH));
    let mut updater_q = VecDeque::new();
    updater_q.push_back(SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH));

    for action in input {
        match action {
            Action::Commit => {
                if naive_q.len() > 1 {
                    naive_q.pop_front();
                    serial_q.pop_front();
                    batches_q.pop_front();
                    updater_q.pop_front();
                }
            }
            Action::Execute(block) => {
                let updates = block
                    .iter()
                    .map(|txn_updates| txn_updates.iter().map(|(k, v)| (*k, v)).collect())
                    .collect::<Vec<_>>();
                let updates_flat_batch = updates.iter().flatten().cloned().collect::<Vec<_>>();

                let committed = naive_q.front_mut().unwrap();
                let proofs = updates_flat_batch
                    .iter()
                    .map(|(k, _)| (*k, committed.get_proof(k)))
                    .collect();
                let proof_reader = ProofReader::new(proofs);

                let mut naive_smt = naive_q.back().unwrap().clone().update(&updates_flat_batch);

                let serial_smt = serial_q
                    .back()
                    .unwrap()
                    .serial_update(updates.clone(), &proof_reader)
                    .unwrap()
                    .1;
                serial_q.back().unwrap().assert_no_external_strong_ref();

                let batches_smt = batches_q
                    .back()
                    .unwrap()
                    .batches_update(updates, &proof_reader)
                    .unwrap()
                    .1;
                batches_q.back().unwrap().assert_no_external_strong_ref();

                let updater_smt = updater_q
                    .back()
                    .unwrap()
                    .batch_update(updates_flat_batch, &proof_reader)
                    .unwrap();
                updater_q.back().unwrap().assert_no_external_strong_ref();

                assert_eq!(serial_smt.root_hash(), naive_smt.get_root_hash());
                assert_eq!(batches_smt.root_hash(), naive_smt.get_root_hash());
                assert_eq!(updater_smt.root_hash(), naive_smt.get_root_hash());

                naive_q.push_back(naive_smt);
                serial_q.push_back(serial_smt);
                batches_q.push_back(batches_smt);
                updater_q.push_back(updater_smt);
            }
        }
    }
}

trait AssertNoExternalStrongRef {
    fn assert_no_external_strong_ref(&self);
}

impl<V> AssertNoExternalStrongRef for SparseMerkleTree<V> {
    fn assert_no_external_strong_ref(&self) {
        assert_subtree_sole_strong_ref(&self.inner.root);
    }
}

fn assert_subtree_sole_strong_ref<V>(subtree: &SubTree<V>) {
    if let SubTree::NonEmpty {
        root: NodeHandle::Shared(arc),
        ..
    } = subtree
    {
        assert_eq!(Arc::strong_count(arc), 1);
        if let NodeInner::Internal(internal_node) = arc.inner() {
            assert_subtree_sole_strong_ref(&internal_node.left);
            assert_subtree_sole_strong_ref(&internal_node.right);
        }
    }
}
