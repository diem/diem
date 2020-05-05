// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::start_storage_service;
use crate::{tests::suite, ExecutionCorrectnessManager};
use executor_types::BlockExecutor;

#[test]
fn test() {
    suite::run_test_suite(block_executor);
}

fn block_executor() -> Box<dyn BlockExecutor> {
    let (config, _handle) = start_storage_service();
    let execution_correctness_manager =
        ExecutionCorrectnessManager::new_thread(config.storage.simple_address);
    let block_executor = execution_correctness_manager.client();
    block_executor
}
