// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use libra_jellyfish_merkle::test_helper::{
    arb_existent_kvs_and_nonexistent_keys, arb_kv_pair_with_distinct_last_nibble,
    arb_tree_with_index, test_get_range_proof, test_get_with_proof,
    test_get_with_proof_with_distinct_last_nibble,
};
use libra_proptest_helpers::ValueGenerator;
use libradb::{
    schema::fuzzing::fuzz_decode, test_helper::arb_blocks_to_commit, test_save_blocks_impl,
};
use proptest::{collection::vec, prelude::*};

#[derive(Clone, Debug, Default)]
pub struct StorageSaveBlocks;

impl FuzzTargetImpl for StorageSaveBlocks {
    fn description(&self) -> &'static str {
        "Storage save blocks"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_blocks_to_commit()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_blocks_to_commit());
        test_save_blocks_impl(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct StorageSchemaDecode;

impl FuzzTargetImpl for StorageSchemaDecode {
    fn description(&self) -> &'static str {
        "Storage schemas do not panic on corrupted bytes."
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen.generate(prop_oneof![
            100 => vec(any::<u8>(), 0..1024),
            1 => vec(any::<u8>(), 1024..1024 * 10),
        ]))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_decode(data)
    }
}

//============== JellyfishMerkleTree =============

#[derive(Clone, Debug, Default)]
pub struct JellyfishGetWithProof;

impl FuzzTargetImpl for JellyfishGetWithProof {
    fn description(&self) -> &'static str {
        "JellyfishMerkleTree get with proof"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_existent_kvs_and_nonexistent_keys(
            1000, 100,
        )))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_existent_kvs_and_nonexistent_keys(1000, 100));
        test_get_with_proof(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct JellyfishGetWithProofWithDistinctLastNibble;

impl FuzzTargetImpl for JellyfishGetWithProofWithDistinctLastNibble {
    fn description(&self) -> &'static str {
        "JellyfishMerkleTree get with proof with distinct last nibble"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_kv_pair_with_distinct_last_nibble()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_kv_pair_with_distinct_last_nibble());
        test_get_with_proof_with_distinct_last_nibble(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct JellyfishGetRangeProof;

impl FuzzTargetImpl for JellyfishGetRangeProof {
    fn description(&self) -> &'static str {
        "JellyfishMerkleTree get range proof"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_tree_with_index(1000)))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_tree_with_index(1000));
        test_get_range_proof(input);
    }
}
