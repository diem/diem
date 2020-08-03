// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use executor::fuzzing::fuzz;
use libra_proptest_helpers::ValueGenerator;
use rand::RngCore;

#[derive(Clone, Debug, Default)]
pub struct ExecuteAndCommitChunk;

impl FuzzTargetImpl for ExecuteAndCommitChunk {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "state-sync > executor::execute_and_commit_chunk"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let mut output = vec![0u8; 4096];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut output);
        Some(output)
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz(data);
    }
}
