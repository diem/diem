// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sparse_merkle::utils::partition;
use bitvec::prelude::*;
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode, SparseMerkleProof};
use std::collections::{BTreeMap, HashMap};

type Cache = HashMap<BitVec<Msb0, u8>, HashValue>;

struct NaiveSubTree<'a> {
    leaves: &'a [(HashValue, HashValue)],
    depth: usize,
}

impl<'a> NaiveSubTree<'a> {
    fn get_proof(
        &'a self,
        key: &HashValue,
        mut cache: &mut Cache,
    ) -> (Option<SparseMerkleLeafNode>, Vec<HashValue>) {
        if self.is_empty() {
            (None, Vec::new())
        } else if self.leaves.len() == 1 {
            let only_leaf = self.leaves[0];
            (
                Some(SparseMerkleLeafNode::new(only_leaf.0, only_leaf.1)),
                Vec::new(),
            )
        } else {
            let (left, right) = self.children();
            if key.bit(self.depth) {
                let (ret_leaf, mut ret_siblings) = right.get_proof(key, &mut cache);
                ret_siblings.push(left.get_hash(&mut cache));
                (ret_leaf, ret_siblings)
            } else {
                let (ret_leaf, mut ret_siblings) = left.get_proof(key, &mut cache);
                ret_siblings.push(right.get_hash(&mut cache));
                (ret_leaf, ret_siblings)
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    fn get_hash(&self, mut cache: &mut Cache) -> HashValue {
        if self.leaves.is_empty() {
            return *SPARSE_MERKLE_PLACEHOLDER_HASH;
        }

        let position = self.leaves[0]
            .0
            .view_bits()
            .split_at(self.depth)
            .0
            .to_bitvec();

        match cache.get(&position) {
            Some(hash) => *hash,
            None => {
                let hash = self.get_hash_uncached(&mut cache);
                cache.insert(position, hash);
                hash
            }
        }
    }

    fn get_hash_uncached(&self, mut cache: &mut Cache) -> HashValue {
        assert!(!self.leaves.is_empty());
        if self.leaves.len() == 1 {
            let only_leaf = self.leaves[0];
            SparseMerkleLeafNode::new(only_leaf.0, only_leaf.1).hash()
        } else {
            let (left, right) = self.children();
            SparseMerkleInternalNode::new(left.get_hash(&mut cache), right.get_hash(&mut cache))
                .hash()
        }
    }

    fn children(&self) -> (Self, Self) {
        let pivot = partition(self.leaves, self.depth);
        let (left, right) = self.leaves.split_at(pivot);
        (
            Self {
                leaves: left,
                depth: self.depth + 1,
            },
            Self {
                leaves: right,
                depth: self.depth + 1,
            },
        )
    }
}

#[derive(Clone, Default)]
pub struct NaiveSmt {
    leaves: Vec<(HashValue, HashValue)>,
    cache: Cache,
}

impl NaiveSmt {
    pub fn new<V: CryptoHash>(leaves: &[(HashValue, &V)]) -> Self {
        Self::default().update(leaves)
    }

    pub fn update<V: CryptoHash>(self, updates: &[(HashValue, &V)]) -> Self {
        let mut leaves = self.leaves.into_iter().collect::<BTreeMap<_, _>>();
        let mut new_leaves = updates
            .iter()
            .map(|(address, value)| (*address, value.hash()))
            .collect::<BTreeMap<_, _>>();
        leaves.append(&mut new_leaves);

        Self {
            leaves: leaves.into_iter().collect::<Vec<_>>(),
            cache: Cache::new(),
        }
    }

    pub fn get_proof<V: CryptoHash>(&mut self, key: &HashValue) -> SparseMerkleProof<V> {
        let root = NaiveSubTree {
            leaves: &self.leaves,
            depth: 0,
        };

        let (leaf, siblings) = root.get_proof(key, &mut self.cache);
        SparseMerkleProof::new(leaf, siblings)
    }

    pub fn get_root_hash(&mut self) -> HashValue {
        let root = NaiveSubTree {
            leaves: &self.leaves,
            depth: 0,
        };

        root.get_hash(&mut self.cache)
    }
}
