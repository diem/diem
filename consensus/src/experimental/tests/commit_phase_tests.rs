// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{experimental::commit_phase::CommitPhase, test_utils::consensus_runtime};

use consensus_types::executed_block::ExecutedBlock;
use diem_logger::debug;
use diem_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};

use std::sync::Arc;

use crate::{metrics_safety_rules::MetricsSafetyRules, network_interface::ConsensusMsg};
use channel::{Receiver, Sender};
use diem_infallible::Mutex;
use futures::{SinkExt, StreamExt};

use crate::{
    experimental::ordering_state_computer::OrderingStateComputer, state_replication::StateComputer,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_crypto::{ed25519::Ed25519Signature, hash::ACCUMULATOR_PLACEHOLDER_HASH};

use diem_types::{
    account_address::AccountAddress, validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};
use futures::future::FutureExt;
use network::protocols::network::Event;

use std::collections::BTreeMap;

use crate::test_utils::{timed_block_on, EmptyStateComputer};
use consensus_types::experimental::{commit_decision::CommitDecision, commit_vote::CommitVote};
use diem_types::block_info::BlockInfo;

use crate::{
    block_storage::BlockStore,
    experimental::{
        commit_phase::{CommitChannelType, PendingBlocks},
        errors::Error,
        execution_phase::ExecutionRequest,
    },
    round_manager::VerifiedEvent,
    state_replication::empty_state_computer_call_back,
};

use crate::experimental::{
    buffer_manager::SyncAck,
    tests::test_utils::{
        prepare_commit_phase_with_block_store_state_computer,
        prepare_executed_blocks_with_executed_ledger_info,
        prepare_executed_blocks_with_ordered_ledger_info,
    },
};
use futures::channel::oneshot;
use tokio::runtime::Runtime;

const TEST_CHANNEL_SIZE: usize = 30;

pub fn prepare_commit_phase(
    runtime: &Runtime,
) -> (
    Sender<CommitChannelType>,
    Sender<VerifiedEvent>,
    Sender<oneshot::Sender<SyncAck>>,
    Receiver<ExecutionRequest>,
    Receiver<Event<ConsensusMsg>>,
    Arc<Mutex<MetricsSafetyRules>>,
    Vec<ValidatorSigner>,
    Arc<OrderingStateComputer>,
    ValidatorVerifier,
    CommitPhase,
    Arc<BlockStore>,
) {
    prepare_commit_phase_with_block_store_state_computer(
        runtime,
        Arc::new(EmptyStateComputer),
        TEST_CHANNEL_SIZE,
    )
}

fn generate_random_commit_vote(signer: &ValidatorSigner) -> CommitVote {
    let dummy_ledger_info = LedgerInfo::new(BlockInfo::random(0), *ACCUMULATOR_PLACEHOLDER_HASH);

    CommitVote::new(signer.author(), dummy_ledger_info, signer)
}

fn generate_random_commit_decision(signer: &ValidatorSigner) -> CommitDecision {
    let dummy_ledger_info = LedgerInfo::new(BlockInfo::random(0), *ACCUMULATOR_PLACEHOLDER_HASH);

    let mut dummy_ledger_info_with_sig = LedgerInfoWithSignatures::new(
        dummy_ledger_info.clone(),
        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
    );

    dummy_ledger_info_with_sig.add_signature(signer.author(), signer.sign(&dummy_ledger_info));

    CommitDecision::new(dummy_ledger_info_with_sig)
}

mod commit_phase_e2e_tests {
    use super::*;

    /// happy path test
    #[test]
    fn test_happy_path() {
        // TODO
    }

    /// reset test
    #[test]
    fn test_reset() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            _msg_tx,
            mut commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // fill in the commit channel with (TEST_CHANNEL_SIZE + 1) good commit blocks
            for _ in 0..=TEST_CHANNEL_SIZE {
                commit_tx
                    .send(CommitChannelType(
                        vecblocks.clone(),
                        li_sig.clone(),
                        empty_state_computer_call_back(),
                    ))
                    .await
                    .ok();
            }

            // reset
            let (tx, rx) = oneshot::channel::<SyncAck>();
            commit_phase_reset_tx.send(tx).await.ok();
            rx.await.ok();

            // now commit_tx should be exhausted. We can send more without blocking.
            commit_tx
                .send(CommitChannelType(
                    vecblocks.clone(),
                    li_sig.clone(),
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();
        });
    }

    /// commit message retry test
    #[test]
    fn test_retry() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(CommitChannelType(
                    vecblocks,
                    li_sig,
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();

            // check the next two messages from the self loop channel
            let commit_vote_msg = self_loop_rx.next().await.unwrap();
            if let Event::Message(_, ConsensusMsg::CommitVoteMsg(request)) = commit_vote_msg {
                let second_commit_vote_msg = self_loop_rx.next().await.unwrap();
                if let Event::Message(_, ConsensusMsg::CommitVoteMsg(second_request)) =
                    second_commit_vote_msg
                {
                    assert_eq!(request, second_request);
                    return;
                }
            }
            panic!("We expect only commit vote messages from the self loop channel in this test.");
        });
    }

    // [ Attention ]
    // These e2e tests below are end-to-end negative tests.
    // They might yield false negative results if now_or_never() is called
    // earlier than the commit phase committed any blocks.

    /// Send bad commit blocks
    #[test]
    fn test_bad_commit_blocks() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            state_computer,
            _validator,
            commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        let genesis_qc = certificate_for_genesis();
        let block = Block::new_proposal(vec![], 1, 1, genesis_qc, signers.first().unwrap());
        let compute_result = state_computer
            .compute(&block, *ACCUMULATOR_PLACEHOLDER_HASH)
            .unwrap();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // bad blocks
            commit_tx
                .send(CommitChannelType(
                    vec![ExecutedBlock::new(block.clone(), compute_result)],
                    LedgerInfoWithSignatures::new(
                        LedgerInfo::new(
                            block.gen_block_info(*ACCUMULATOR_PLACEHOLDER_HASH, 0, None),
                            *ACCUMULATOR_PLACEHOLDER_HASH,
                        ),
                        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
                    ),
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();

            // the commit phase should not send message to itself
            assert!(self_loop_rx.next().now_or_never().is_none());

            debug!("Let's see if we can reach here.");
            // it does not commit blocks either
            assert!(commit_result_rx.next().now_or_never().is_none());
        });
    }

    /// Send bad commit vote
    #[test]
    fn test_bad_commit_vote() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            mut msg_tx,
            _commit_phase_reset_tx,
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(CommitChannelType(
                    vecblocks,
                    li_sig,
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();

            // it sends itself a commit vote
            let self_msg = self_loop_rx.next().await;

            assert!(matches!(self_msg, Some(Event::Message(_, _),)));

            // send a bad vote

            msg_tx
                .send(VerifiedEvent::CommitVote(Box::new(
                    generate_random_commit_vote(&signers[0]),
                )))
                .await
                .ok();

            // it does not commit blocks either
            assert!(commit_result_rx.next().now_or_never().is_none());
        });
    }

    /// Send bad commit decision
    #[test]
    fn test_bad_commit_decision() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            mut msg_tx,
            _commit_phase_reset_tx,
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(CommitChannelType(
                    vecblocks,
                    li_sig,
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();

            let (_, li_sig_prime) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

            // it sends itself a commit vote
            let self_msg = self_loop_rx.next().await;

            assert!(matches!(self_msg, Some(Event::Message(_, _),)));

            // send a bad commit decision with inconsistent block info
            msg_tx
                .send(VerifiedEvent::CommitDecision(Box::new(
                    CommitDecision::new(LedgerInfoWithSignatures::new(
                        li_sig_prime.ledger_info().clone(),
                        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
                    )),
                )))
                .await
                .ok();

            // it does not commit blocks either
            assert!(commit_result_rx.next().now_or_never().is_none());
        });
    }
}

mod commit_phase_function_tests {
    use super::*;
    use crate::experimental::tests::test_utils::new_executed_ledger_info_with_empty_signature;

    /// negative tests for commit_phase.process_commit_vote
    #[test]
    fn test_commit_phase_process_commit_vote() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            commit_phase.set_blocks(Some(PendingBlocks::new(
                vecblocks,
                li_sig,
                empty_state_computer_call_back(),
            )));

            let random_commit_vote = generate_random_commit_vote(signer);

            assert!(matches!(
                commit_phase.process_commit_vote(&random_commit_vote).await,
                Err(Error::InconsistentBlockInfo(_, _))
            ));
        });
    }

    #[test]
    fn test_commit_phase_process_commit_decision() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            commit_phase.set_blocks(Some(PendingBlocks::new(
                vecblocks,
                li_sig,
                empty_state_computer_call_back(),
            )));

            let random_commit_decision = generate_random_commit_decision(signer);

            assert!(matches!(
                commit_phase
                    .process_commit_decision(&random_commit_decision)
                    .await,
                Err(Error::InconsistentBlockInfo(_, _))
            ));
        });
    }

    #[test]
    fn test_commit_phase_process_reset() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            assert!(commit_phase.blocks().is_none());

            // fill in the commit channel with TEST_CHANNEL_SIZE good commit blocks
            for _ in 0..TEST_CHANNEL_SIZE {
                commit_tx
                    .send(CommitChannelType(
                        vecblocks.clone(),
                        li_sig.clone(),
                        empty_state_computer_call_back(),
                    ))
                    .await
                    .ok();
            }

            // set the blocks to be some good blocks
            commit_phase.set_blocks(Some(PendingBlocks::new(
                vecblocks.clone(),
                li_sig.clone(),
                empty_state_computer_call_back(),
            )));

            // reset
            let (tx, rx) = oneshot::channel::<SyncAck>();
            commit_phase.process_reset_event(tx).await.ok();
            rx.await.ok();

            // the block should be dropped
            assert!(commit_phase.blocks().is_none());
            // .. and we should be able to send blocks to commit_tx
            commit_tx
                .send(CommitChannelType(
                    vecblocks.clone(),
                    li_sig.clone(),
                    empty_state_computer_call_back(),
                ))
                .await
                .ok();
        });
    }

    #[test]
    fn test_commit_phase_check_commit() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            assert!(commit_phase.blocks().is_none());

            // when blocks is none
            commit_phase.check_commit().await.ok();

            assert!(commit_phase.blocks().is_none());

            commit_phase.set_blocks(Some(PendingBlocks::new(
                vecblocks.clone(),
                li_sig.clone(),
                empty_state_computer_call_back(),
            )));

            // when blocks is good
            commit_phase.check_commit().await.ok();

            // the block should be consumed
            assert!(commit_phase.blocks().is_none());
            assert_eq!(commit_phase.load_back_pressure(), 1);

            // when block contains bad signatures
            let ledger_info_with_no_sig = new_executed_ledger_info_with_empty_signature(
                vecblocks.last().unwrap().block_info(),
                li_sig.ledger_info(),
            );

            commit_phase.set_blocks(Some(PendingBlocks::new(
                vecblocks,
                ledger_info_with_no_sig,
                empty_state_computer_call_back(),
            )));
            commit_phase.check_commit().await.ok();

            // the block should be there
            assert!(commit_phase.blocks().is_some());
        });
    }

    #[test]
    fn test_commit_phase_process_executed_blocks() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_phase_reset_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
            _block_store,
        ) = prepare_commit_phase(&runtime);

        timed_block_on(&mut runtime, async move {
            let _signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

            let ledger_info_with_no_sig = new_executed_ledger_info_with_empty_signature(
                vecblocks.last().unwrap().block_info(),
                li_sig.ledger_info(),
            );

            // no signatures
            assert!(matches!(
                commit_phase
                    .process_executed_blocks(
                        vecblocks,
                        ledger_info_with_no_sig,
                        empty_state_computer_call_back()
                    )
                    .await,
                Err(_),
            ));
        });
    }
}
