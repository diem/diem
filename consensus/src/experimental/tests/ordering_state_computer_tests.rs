// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::ordering_state_computer::OrderingStateComputer, state_replication::StateComputer,
};
use channel::Receiver;
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use diem_types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::random_validator_verifier,
};
use executor_types::StateComputeResult;
use std::sync::Arc;

use crate::{
    experimental::execution_phase::{ExecutionChannelType, ResetAck},
    state_replication::empty_state_computer_call_back,
    test_utils::{consensus_runtime, timed_block_on, EmptyStateComputer},
};
use consensus_types::{executed_block::ExecutedBlock, quorum_cert::QuorumCert};
use diem_crypto::ed25519::Ed25519Signature;
use diem_types::{account_address::AccountAddress, validator_signer::ValidatorSigner};
use futures::{channel::oneshot, StreamExt};
use rand::Rng;
use std::collections::BTreeMap;

pub fn prepare_ordering_state_computer(
    channel_size: usize,
) -> (
    Arc<OrderingStateComputer>,
    Receiver<ExecutionChannelType>,
    Receiver<oneshot::Sender<ResetAck>>,
) {
    let (commit_result_tx, commit_result_rx) =
        channel::new_test::<ExecutionChannelType>(channel_size);
    let (execution_phase_reset_tx, execution_phase_reset_rx) =
        channel::new_test::<oneshot::Sender<ResetAck>>(1);
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
    let num_nodes = 1;
    let channel_size = 30;
    let mut runtime = consensus_runtime();

    let (state_computer, mut commit_result_rx, _) = prepare_ordering_state_computer(channel_size);

    let (signers, _) = random_validator_verifier(num_nodes, None, false);
    let signer = &signers[0];
    let genesis_qc = certificate_for_genesis();
    let block = random_empty_block(signer, genesis_qc);

    // test compute
    let dummy_state_compute_result = state_computer
        .compute(&block, *ACCUMULATOR_PLACEHOLDER_HASH)
        .unwrap();
    assert_eq!(dummy_state_compute_result, StateComputeResult::new_dummy());

    // test commit
    let li = LedgerInfo::new(
        block.gen_block_info(
            dummy_state_compute_result.root_hash(),
            dummy_state_compute_result.version(),
            dummy_state_compute_result.epoch_state().clone(),
        ),
        *ACCUMULATOR_PLACEHOLDER_HASH,
    );

    let blocks = vec![Arc::new(ExecutedBlock::new(
        block.clone(),
        dummy_state_compute_result,
    ))];

    let li_sig =
        LedgerInfoWithSignatures::new(li, BTreeMap::<AccountAddress, Ed25519Signature>::new());

    // ordering_state_computer should send the same block and finality proof to the channel
    timed_block_on(&mut runtime, async move {
        state_computer
            .commit(&blocks, li_sig.clone(), empty_state_computer_call_back())
            .await
            .ok();

        let ExecutionChannelType(ordered_block, finality_proof, _) =
            commit_result_rx.next().await.unwrap();
        assert_eq!(ordered_block.len(), 1);
        assert_eq!(ordered_block[0], block);
        assert_eq!(finality_proof, li_sig);
    });
}
