// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    executed_block::ExecutedBlock,
    quorum_cert::QuorumCert,
};
use diem_crypto::HashValue;
use diem_types::{ledger_info::LedgerInfo, validator_verifier::random_validator_verifier};
use executor_types::{Error, StateComputeResult};

use crate::{
    experimental::{
        buffer_manager::create_channel,
        execution_phase::{ExecutionPhase, ExecutionRequest, ExecutionResponse},
        pipeline_phase::PipelinePhase,
        tests::phase_tester::PhaseTester,
    },
    test_utils::{consensus_runtime, RandomComputeResultStateComputer},
};

pub fn prepare_execution_phase() -> (HashValue, ExecutionPhase) {
    let execution_proxy = Arc::new(RandomComputeResultStateComputer::new());
    let random_hash_value = execution_proxy.get_root_hash();
    let execution_phase = ExecutionPhase::new(execution_proxy);
    (random_hash_value, execution_phase)
}

fn add_execution_phase_test_cases(
    phase_tester: &mut PhaseTester<ExecutionPhase>,
    random_hash_value: HashValue,
) {
    let genesis_qc = certificate_for_genesis();
    let (signers, _validators) = random_validator_verifier(1, None, false);
    let block = Block::new_proposal(vec![], 1, 1, genesis_qc, &signers[0]);

    // happy path
    phase_tester.add_test_case(
        ExecutionRequest {
            ordered_blocks: vec![ExecutedBlock::new(block, StateComputeResult::new_dummy())],
        },
        Box::new(move |resp| {
            assert_eq!(
                resp.inner.unwrap()[0].compute_result().root_hash(),
                random_hash_value
            );
        }),
    );

    // empty block
    phase_tester.add_test_case(
        ExecutionRequest {
            ordered_blocks: vec![],
        },
        Box::new(move |resp| assert!(matches!(resp.inner, Err(Error::EmptyBlocks)))),
    );

    // bad parent id
    let bad_qc = QuorumCert::certificate_for_genesis_from_ledger_info(
        &LedgerInfo::mock_genesis(None),
        random_hash_value,
    );
    let bad_block = Block::new_proposal(vec![], 1, 1, bad_qc, &signers[0]);
    phase_tester.add_test_case(
        ExecutionRequest {
            ordered_blocks: vec![ExecutedBlock::new(
                bad_block,
                StateComputeResult::new_dummy(),
            )],
        },
        Box::new(move |resp| assert!(matches!(resp.inner, Err(Error::BlockNotFound(_))))),
    );
}

#[test]
fn execution_phase_tests() {
    let runtime = consensus_runtime();

    // unit tests
    let (random_hash_value, execution_phase) = prepare_execution_phase();
    let mut unit_phase_tester = PhaseTester::<ExecutionPhase>::new();
    add_execution_phase_test_cases(&mut unit_phase_tester, random_hash_value);
    unit_phase_tester.unit_test(&execution_phase);

    // e2e tests
    let (in_channel_tx, in_channel_rx) = create_channel::<ExecutionRequest>();
    let (out_channel_tx, out_channel_rx) = create_channel::<ExecutionResponse>();

    let execution_phase_pipeline = PipelinePhase::new(
        in_channel_rx,
        Some(out_channel_tx),
        Box::new(execution_phase),
    );

    runtime.spawn(execution_phase_pipeline.start());

    let mut e2e_phase_tester = PhaseTester::<ExecutionPhase>::new();
    add_execution_phase_test_cases(&mut e2e_phase_tester, random_hash_value);
    e2e_phase_tester.e2e_test(in_channel_tx, out_channel_rx);
}
