// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::{self, TestRunner},
};
use state_synchronizer::fuzzing::{state_sync_msg_strategy, test_state_sync_msg_fuzzer_impl};

#[derive(Debug, Default)]
pub struct StateSyncMsg;

impl FuzzTargetImpl for StateSyncMsg {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "State sync network message"
    }

    fn fuzz(&self, data: &[u8]) {
        let passthrough_rng =
            test_runner::TestRng::from_seed(test_runner::RngAlgorithm::PassThrough, &data);
        let config = test_runner::Config::default();
        let mut runner = TestRunner::new_with_rng(config, passthrough_rng);

        let strategy = state_sync_msg_strategy();
        let strategy_tree = match strategy.new_tree(&mut runner) {
            Ok(x) => x,
            Err(_) => return,
        };
        let data = strategy_tree.current();

        test_state_sync_msg_fuzzer_impl(data);
    }
}
