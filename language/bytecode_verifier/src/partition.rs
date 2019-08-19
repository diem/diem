// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the partition data structure used for tracking equality among nonces.
use crate::nonce::Nonce;
use mirai_annotations::checked_verify;
use std::{
    collections::{BTreeMap, BTreeSet},
    usize::MAX,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Partition {
    nonce_to_id: BTreeMap<Nonce, usize>,
    id_to_nonce_set: BTreeMap<usize, BTreeSet<Nonce>>,
}

impl Partition {
    // adds a nonce to the partition; new_nonce must be a fresh nonce
    pub fn add_nonce(&mut self, new_nonce: Nonce) {
        let nonce_const = new_nonce.inner();
        self.nonce_to_id.insert(new_nonce.clone(), nonce_const);
        let mut singleton_set = BTreeSet::new();
        singleton_set.insert(new_nonce);
        self.id_to_nonce_set.insert(nonce_const, singleton_set);
    }

    // removes a nonce that already exists in the partition
    pub fn remove_nonce(&mut self, nonce: Nonce) {
        let id = self.nonce_to_id.remove(&nonce).unwrap();
        self.id_to_nonce_set.entry(id).and_modify(|nonce_set| {
            nonce_set.remove(&nonce);
        });
        if self.id_to_nonce_set[&id].is_empty() {
            self.id_to_nonce_set.remove(&id).unwrap();
        }
    }

    // adds an equality between nonce1 and nonce2
    pub fn add_equality(&mut self, nonce1: Nonce, nonce2: Nonce) {
        let id1 = self.nonce_to_id[&nonce1];
        let id2 = self.nonce_to_id[&nonce2];
        if id1 == id2 {
            return;
        }
        let mut nonce_set2 = self.id_to_nonce_set.remove(&id2).unwrap();
        for nonce in &nonce_set2 {
            self.nonce_to_id
                .entry(nonce.clone())
                .and_modify(|id| *id = id1);
        }
        self.id_to_nonce_set.entry(id1).and_modify(|nonce_set| {
            nonce_set.append(&mut nonce_set2);
        });
    }

    // checks if nonce1 and nonce2 are known to be equal
    pub fn is_equal(&self, nonce1: Nonce, nonce2: Nonce) -> bool {
        self.nonce_to_id[&nonce1] == self.nonce_to_id[&nonce2]
    }

    // returns a canonical version of self in which an id of a set is determined
    // to be the least element of the set.
    // the choice of returned id is arbitrary but it must be a function on nonce sets.
    pub fn construct_canonical_partition(&self, nonce_map: &BTreeMap<Nonce, Nonce>) -> Self {
        let mut id_to_nonce_set = BTreeMap::new();
        for nonce_set in self.id_to_nonce_set.values() {
            let canonical_nonce_set: BTreeSet<Nonce> = nonce_set
                .iter()
                .map(|nonce| nonce_map[nonce].clone())
                .collect();
            let canonical_id = Self::canonical_id(&canonical_nonce_set);
            id_to_nonce_set.insert(canonical_id, canonical_nonce_set);
        }
        let nonce_to_id = Self::compute_nonce_to_id(&id_to_nonce_set);
        Self {
            nonce_to_id,
            id_to_nonce_set,
        }
    }

    pub fn nonces(&self) -> BTreeSet<Nonce> {
        self.nonce_to_id.keys().cloned().collect()
    }

    // both self and partition must be canonical and over the same set of nonces
    pub fn join(&self, partition: &Partition) -> Self {
        checked_verify!(self.nonces() == partition.nonces());
        // The join algorithm exploits the property that both self and partition are partitions over
        // the same set of nonces. The algorithm does partition refinement by constructing
        // for each nonce the intersection of the two sets containing it in self and partition.
        // In the resulting partition, the nonce is mapped to this intersection set.
        let mut nonce_to_id_pair = BTreeMap::new();
        let mut id_pair_to_nonce_set = BTreeMap::new();
        for (nonce, id) in self.nonce_to_id.iter() {
            let id_pair = (id, partition.nonce_to_id[nonce]);
            nonce_to_id_pair.insert(nonce.clone(), id_pair);
            id_pair_to_nonce_set.entry(id_pair).or_insert({
                let nonce_set_for_id_pair: BTreeSet<Nonce> = self.id_to_nonce_set[&id_pair.0]
                    .intersection(&partition.id_to_nonce_set[&id_pair.1])
                    .cloned()
                    .collect();
                nonce_set_for_id_pair
            });
        }
        let id_to_nonce_set: BTreeMap<usize, BTreeSet<Nonce>> = id_pair_to_nonce_set
            .into_iter()
            .map(|(_, nonce_set)| (Self::canonical_id(&nonce_set), nonce_set))
            .collect();
        let nonce_to_id = Self::compute_nonce_to_id(&id_to_nonce_set);
        Self {
            nonce_to_id,
            id_to_nonce_set,
        }
    }

    fn canonical_id(nonce_set: &BTreeSet<Nonce>) -> usize {
        let mut minimum_id = MAX;
        for nonce in nonce_set {
            let id = nonce.inner();
            if minimum_id > id {
                minimum_id = id;
            }
        }
        minimum_id
    }

    fn compute_nonce_to_id(
        id_to_nonce_set: &BTreeMap<usize, BTreeSet<Nonce>>,
    ) -> BTreeMap<Nonce, usize> {
        let mut nonce_to_id = BTreeMap::new();
        for (id, nonce_set) in id_to_nonce_set.iter() {
            for nonce in nonce_set {
                nonce_to_id.insert(nonce.clone(), id.clone());
            }
        }
        nonce_to_id
    }
}
