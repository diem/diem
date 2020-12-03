// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_crypto::HashValue;
use diem_proptest_helpers::ValueGenerator;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionListWithProof},
};
use executor::fuzzing::{fuzz_execute_and_commit_blocks, fuzz_execute_and_commit_chunk};
use proptest::{collection::vec, prelude::*};

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
        fuzz_execute_and_commit_chunk(txn_list_with_proof, verified_target_li);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExecuteAndCommitBlocks;

impl FuzzTargetImpl for ExecuteAndCommitBlocks {
    fn description(&self) -> &'static str {
        "LEC > executor::execute_block & executor::commit_blocks"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(execute_and_commit_blocks_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (blocks, li_with_sigs) = fuzz_data_to_value(data, execute_and_commit_blocks_input());
        fuzz_execute_and_commit_blocks(blocks, li_with_sigs);
    }
}

prop_compose! {
    fn execute_and_commit_chunk_input()(
        txn_list_with_proof in any::<TransactionListWithProof>(),
        verified_target_li in any::<LedgerInfoWithSignatures>()
    ) -> (TransactionListWithProof, LedgerInfoWithSignatures) {
        (txn_list_with_proof, verified_target_li)
    }
}

prop_compose! {
    fn execute_and_commit_blocks_input()(
        blocks in vec((any::<HashValue>(), vec(any::<Transaction>(), 0..10)), 0..10),
        li_with_sigs in any::<LedgerInfoWithSignatures>()
    ) -> (Vec<(HashValue, Vec<Transaction>)>, LedgerInfoWithSignatures) {
        (blocks, li_with_sigs)
    }
}
