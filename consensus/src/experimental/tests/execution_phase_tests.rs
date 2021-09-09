// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::{
        execution_phase::{ExecutionPhase, ExecutionRequest, ExecutionResponse},
        pipeline_phase::{PipelinePhase, ResponseWithInstruction, StatelessPipeline},
    },
    test_utils::{consensus_runtime, timed_block_on, RandomComputeResultStateComputer},
};
use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    executed_block::ExecutedBlock,
};
use diem_crypto::HashValue;
use diem_types::validator_verifier::random_validator_verifier;
use executor_types::StateComputeResult;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use std::sync::Arc;

pub fn prepare_execution_phase() -> (HashValue, ExecutionPhase) {
    let execution_proxy = Arc::new(RandomComputeResultStateComputer::new());
    let random_hash_value = execution_proxy.get_root_hash();
    let execution_phase = ExecutionPhase::new(execution_proxy);
    (random_hash_value, execution_phase)
}

pub fn prepare_execution_pipeline() -> (
    UnboundedSender<ExecutionRequest>,
    UnboundedReceiver<ExecutionResponse>,
    HashValue,
    PipelinePhase<ExecutionPhase>,
) {
    let (in_channel_tx, in_channel_rx) = unbounded::<ExecutionRequest>();
    let (out_channel_tx, out_channel_rx) = unbounded::<ExecutionResponse>();

    let (hash_val, execution_phase) = prepare_execution_phase();

    let execution_phase_pipeline =
        PipelinePhase::new(in_channel_rx, out_channel_tx, Box::new(execution_phase));

    (
        in_channel_tx,
        out_channel_rx,
        hash_val,
        execution_phase_pipeline,
    )
}

// unit tests
#[test]
fn test_execution_phase_process() {
    let mut runtime = consensus_runtime();

    let (random_hash_value, execution_phase) = prepare_execution_phase();

    let genesis_qc = certificate_for_genesis();
    let (signers, _validators) = random_validator_verifier(1, None, false);
    let block = Block::new_proposal(vec![], 1, 1, genesis_qc, &signers[0]);

    timed_block_on(&mut runtime, async move {
        let ResponseWithInstruction {
            resp,
            instruction: _,
        } = execution_phase
            .process(ExecutionRequest {
                ordered_blocks: vec![ExecutedBlock::new(block, StateComputeResult::new_dummy())],
            })
            .await;

        assert_eq!(
            resp.inner.unwrap()[0].compute_result().root_hash(),
            random_hash_value
        );
    });
}

#[test]
fn test_execution_phase_happy_path() {
    let mut runtime = consensus_runtime();

    let (mut in_channel_tx, mut out_channel_rx, random_hash_value, execution_phase_pipeline) =
        prepare_execution_pipeline();

    runtime.spawn(execution_phase_pipeline.start());

    let genesis_qc = certificate_for_genesis();
    let (signers, _validators) = random_validator_verifier(1, None, false);
    let block = Block::new_proposal(vec![], 1, 1, genesis_qc, &signers[0]);

    timed_block_on(&mut runtime, async move {
        in_channel_tx
            .send(ExecutionRequest {
                ordered_blocks: vec![ExecutedBlock::new(block, StateComputeResult::new_dummy())],
            })
            .await
            .ok();

        let out_item = out_channel_rx.next().await.unwrap();

        assert_eq!(
            out_item.inner.unwrap()[0].compute_result().root_hash(),
            random_hash_value
        );
    });
}

// TODO: unhappy paths
