// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_mempool::fuzzing::{
    mempool_incoming_transactions_strategy, test_mempool_process_incoming_transactions_impl,
};
use diem_proptest_helpers::ValueGenerator;

#[derive(Debug, Default)]
pub struct MempoolIncomingTransactions;

impl FuzzTargetImpl for MempoolIncomingTransactions {
    fn description(&self) -> &'static str {
        "Transactions submitted to mempool"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(
            mempool_incoming_transactions_strategy(),
        ))
    }

    fn fuzz(&self, data: &[u8]) {
        let (txns, timeline_state) =
            fuzz_data_to_value(data, mempool_incoming_transactions_strategy());
        test_mempool_process_incoming_transactions_impl(txns, timeline_state);
    }
}
