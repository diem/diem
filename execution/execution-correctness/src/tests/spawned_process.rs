// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::start_storage_service;
use crate::{process_client_wrapper::ProcessClientWrapper, tests::suite};
use executor_types::BlockExecutor;

#[test]
fn test() {
    suite::run_test_suite(block_executor);
}

fn block_executor() -> Box<dyn BlockExecutor> {
    let (config, _handle) = start_storage_service();
    let block_executor = ProcessClientWrapper::new(config.storage.address);
    Box::new(block_executor)
}
