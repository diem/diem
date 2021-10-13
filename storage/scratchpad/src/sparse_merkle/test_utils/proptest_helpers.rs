// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{sparse_merkle::node::{Node, NodeHandle, SubTree}, test_utils::{naive_smt::NaiveSmt, proof_reader::ProofReader}, SparseMerkleTree, AccountStatus};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::account_state_blob::AccountStateBlob;
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    sample::Index,
};
use std::{borrow::Borrow, collections::VecDeque, sync::Arc};

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


fn state_view_proof(
    keys: &[HashValue],
    smt: &SparseMerkleTree<AccountStateBlob>,
    db: &mut NaiveSmt,
) -> ProofReader<AccountStateBlob> {
    ProofReader::new(
        keys.iter()
            .filter_map(|h| match smt.get(*h) {
                AccountStatus::ExistsInScratchPad(_) | AccountStatus::DoesNotExist => None,
                AccountStatus::ExistsInDB | AccountStatus::Unknown => Some((*h, db.get_proof(h))),
            })
            .collect(),
    )
}

pub fn test_smt_correctness_impl(input: Vec<Action>) {
    let mut commit_q = VecDeque::new();
    commit_q.push_back((
        NaiveSmt::new::<AccountStateBlob>(&[]),
        SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH),
        SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH),
        SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH),
    ));
    let mut prev_commits = Vec::new();

    for action in input {
        match action {
            Action::Commit => {
                if commit_q.len() > 1 {
                    prev_commits.push(commit_q.pop_front())
                }
            }
            Action::Execute(block) => {
                let updates = block
                    .iter()
                    .map(|txn_updates| txn_updates.iter().map(|(k, v)| (*k, v)).collect())
                    .collect::<Vec<_>>();
                let updates_flat_batch = updates.iter().flatten().cloned().collect::<Vec<_>>();
                let keys: Vec<_> = updates_flat_batch.iter().map(|(k, _)| *k).collect();

                let (mut committed_n, committed_s, committed_b, committed_u) =
                    commit_q.front().unwrap().clone();

                let (parent_n, parent_s, parent_b, parent_u) =
                    commit_q.back().unwrap().clone();

                prev_commits.pop();
                let proofs_s = state_view_proof(&keys, &committed_s, &mut committed_n);
                let proofs_b = state_view_proof(&keys, &committed_b, &mut committed_n);
                let proofs_u = state_view_proof(&keys, &committed_u, &mut committed_n);
                // emulate edge case where in-mem nodes got dropped after proofs are read
                prev_commits.pop();

                let mut naive_smt = parent_n.update(&updates_flat_batch);
                let serial_smt = parent_s
                    .serial_update(updates.clone(), &proofs_s)
                    .unwrap()
                    .1;
                parent_s.assert_no_external_strong_ref();

                let batches_smt = parent_b
                    .serial_update(updates.clone(), &proofs_b)
                    .unwrap()
                    .1;
                parent_b.assert_no_external_strong_ref();

                let updater_smt = parent_u
                    .serial_update(updates.clone(), &proofs_u)
                    .unwrap()
                    .1;
                parent_u.assert_no_external_strong_ref();

                assert_eq!(serial_smt.root_hash(), naive_smt.get_root_hash());
                assert_eq!(batches_smt.root_hash(), naive_smt.get_root_hash());
                assert_eq!(updater_smt.root_hash(), naive_smt.get_root_hash());

                commit_q.push_back((naive_smt, serial_smt, batches_smt, updater_smt));
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
        if let Node::Internal(internal_node) = arc.borrow() {
            assert_subtree_sole_strong_ref(&internal_node.left);
            assert_subtree_sole_strong_ref(&internal_node.right);
        }
    }
}
