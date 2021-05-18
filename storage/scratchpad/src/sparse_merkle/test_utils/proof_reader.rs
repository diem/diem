// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ProofRead;
use diem_crypto::HashValue;
use diem_types::proof::SparseMerkleProof;
use std::collections::HashMap;

pub struct ProofReader<V>(HashMap<HashValue, SparseMerkleProof<V>>);

impl<V: Sync> ProofReader<V> {
    pub fn new(key_with_proof: Vec<(HashValue, SparseMerkleProof<V>)>) -> Self {
        ProofReader(key_with_proof.into_iter().collect())
    }
}

impl<V: Sync> Default for ProofReader<V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<V: Sync> ProofRead<V> for ProofReader<V> {
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof<V>> {
        self.0.get(&key)
    }
}
