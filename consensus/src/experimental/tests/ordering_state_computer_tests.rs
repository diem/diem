// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::{
        buffer_manager::SyncAck, execution_phase::ExecutionRequest,
        ordering_state_computer::OrderingStateComputer,
    },
    test_utils::EmptyStateComputer,
};
use channel::Receiver;
use consensus_types::{block::Block, quorum_cert::QuorumCert};
use diem_types::validator_signer::ValidatorSigner;
use futures::channel::oneshot;
use rand::Rng;
use std::sync::Arc;

pub fn prepare_ordering_state_computer(
    channel_size: usize,
) -> (
    Arc<OrderingStateComputer>,
    Receiver<ExecutionRequest>,
    Receiver<oneshot::Sender<SyncAck>>,
) {
    let (commit_result_tx, commit_result_rx) = channel::new_test::<ExecutionRequest>(channel_size);
    let (execution_phase_reset_tx, execution_phase_reset_rx) =
        channel::new_test::<oneshot::Sender<SyncAck>>(1);
    let state_computer = Arc::new(OrderingStateComputer::new(
        commit_result_tx,
        Arc::new(EmptyStateComputer {}),
        execution_phase_reset_tx,
    ));

    (state_computer, commit_result_rx, execution_phase_reset_rx)
}

pub fn random_empty_block(signer: &ValidatorSigner, qc: QuorumCert) -> Block {
    let mut rng = rand::thread_rng();
    Block::new_proposal(vec![], rng.gen::<u64>(), rng.gen::<u64>(), qc, signer)
}

#[test]
fn test_ordering_state_computer() {
    // TODO: after changing the ordering state computer
}
