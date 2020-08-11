// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use libradb::{
    schema::fuzzing::fuzz_decode, test_helper::arb_blocks_to_commit, test_save_blocks_impl,
};
use proptest::{
    collection::vec,
    prelude::*,
    strategy::{Strategy, ValueTree},
    test_runner::{self, TestRunner},
};

#[derive(Clone, Debug, Default)]
pub struct StorageSaveBlocks;

impl FuzzTargetImpl for StorageSaveBlocks {
    fn description(&self) -> &'static str {
        "Storage save blocks"
    }

    fn fuzz(&self, data: &[u8]) {
        let passthrough_rng =
            test_runner::TestRng::from_seed(test_runner::RngAlgorithm::PassThrough, &data);

        let config = test_runner::Config::default();
        let mut runner = TestRunner::new_with_rng(config, passthrough_rng);

        let strategy = arb_blocks_to_commit();
        let strategy_tree = match strategy.new_tree(&mut runner) {
            Ok(x) => x,
            Err(_) => return,
        };
        let data = strategy_tree.current();

        test_save_blocks_impl(data);
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
