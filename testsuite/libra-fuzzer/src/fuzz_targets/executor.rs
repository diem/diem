// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use executor::fuzzing::fuzz;
use libra_proptest_helpers::ValueGenerator;
use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::TransactionListWithProof};
use proptest::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ExecuteAndCommitChunk;

impl FuzzTargetImpl for ExecuteAndCommitChunk {
    fn description(&self) -> &'static str {
        "state-sync > executor::execute_and_commit_chunk"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(execute_and_commit_chunk_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (txn_list_with_proof, verified_target_li) =
            fuzz_data_to_value(data, execute_and_commit_chunk_input());
        fuzz(txn_list_with_proof, verified_target_li);
    }
}

prop_compose! {
    fn execute_and_commit_chunk_input()(txn_list_with_proof in any::<TransactionListWithProof>(), verified_target_li in any::<LedgerInfoWithSignatures>()) -> (TransactionListWithProof, LedgerInfoWithSignatures) {
        (txn_list_with_proof, verified_target_li)
    }
}
