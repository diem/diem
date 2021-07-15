// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::commit_phase::CommitPhase,
    network::NetworkSender,
    test_utils::{consensus_runtime, MockStorage},
};
use channel::message_queues::QueueStyle;
use consensus_types::executed_block::ExecutedBlock;
use diem_logger::debug;
use diem_types::{
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use network::peer_manager::{ConnectionRequestSender, PeerManagerRequestSender};
use std::sync::{atomic::AtomicU64, Arc};

use crate::{
    metrics_safety_rules::MetricsSafetyRules,
    network_interface::{ConsensusMsg, ConsensusNetworkSender},
};
use channel::{diem_channel, Receiver, Sender};
use diem_infallible::Mutex;
use futures::{SinkExt, StreamExt};

use crate::{
    experimental::ordering_state_computer::OrderingStateComputer, state_replication::StateComputer,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    hash::{HashValue, ACCUMULATOR_PLACEHOLDER_HASH},
    Uniform,
};
use diem_secure_storage::Storage;
use diem_types::{
    account_address::AccountAddress,
    validator_signer::ValidatorSigner,
    validator_verifier::{random_validator_verifier, ValidatorVerifier},
    waypoint::Waypoint,
};
use futures::future::FutureExt;
use network::protocols::network::{Event, NewNetworkSender};
use safety_rules::{PersistentSafetyStorage, SafetyRulesManager};
use std::collections::BTreeMap;

use crate::test_utils::timed_block_on;
use consensus_types::experimental::{commit_decision::CommitDecision, commit_vote::CommitVote};
use diem_types::block_info::BlockInfo;

use crate::experimental::{commit_phase::PendingBlocks, errors::Error};
use executor_types::StateComputeResult;

fn prepare_commit_phase() -> (
    Sender<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    Sender<ConsensusMsg>,
    Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
    Receiver<Event<ConsensusMsg>>,
    Arc<Mutex<MetricsSafetyRules>>,
    Vec<ValidatorSigner>,
    Arc<OrderingStateComputer>,
    ValidatorVerifier,
    CommitPhase,
) {
    let num_nodes = 1;

    // constants
    let channel_size = 30;
    let back_pressure = Arc::new(AtomicU64::new(0));

    // environment setup
    let (signers, validators) = random_validator_verifier(num_nodes, None, false);
    let validator_set = (&validators).into();
    let signer = &signers[0];

    let waypoint =
        Waypoint::new_epoch_boundary(&LedgerInfo::mock_genesis(Some(validator_set))).unwrap();

    let safety_storage = PersistentSafetyStorage::initialize(
        Storage::from(diem_secure_storage::InMemoryStorage::new()),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
        true,
    );
    let safety_rules_manager = SafetyRulesManager::new_local(safety_storage, false, false, true);

    let (_initial_data, storage) = MockStorage::start_for_testing((&validators).into());
    let epoch_state = EpochState {
        epoch: 1,
        verifier: storage.get_validator_set().into(),
    };
    let validators = epoch_state.verifier.clone();
    let (network_reqs_tx, _network_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
    let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, 8, None);

    let network_sender = ConsensusNetworkSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let author = signer.author();

    let (self_loop_tx, self_loop_rx) = channel::new_test(1000);
    let network = NetworkSender::new(author, network_sender, self_loop_tx, validators);

    let (commit_result_tx, commit_result_rx) =
        channel::new_test::<(Vec<Block>, LedgerInfoWithSignatures)>(channel_size);
    let state_computer = Arc::new(OrderingStateComputer::new(commit_result_tx));

    let mut safety_rules = MetricsSafetyRules::new(safety_rules_manager.client(), storage);
    safety_rules.perform_initialize().unwrap();

    let safety_rules_container = Arc::new(Mutex::new(safety_rules));

    // setting up channels
    let (commit_tx, commit_rx) =
        channel::new_test::<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>(channel_size);

    let (msg_tx, msg_rx) = channel::new_test::<ConsensusMsg>(channel_size);

    let commit_phase = CommitPhase::new(
        commit_rx,
        state_computer.clone(),
        msg_rx,
        epoch_state.verifier.clone(),
        safety_rules_container.clone(),
        author,
        back_pressure,
        network,
    );

    (
        commit_tx,        // channel to pass executed blocks into the commit phase
        msg_tx,           // channel to pass commit messages into the commit phase
        commit_result_rx, // channel to receive commit result from the commit phase
        self_loop_rx,     // channel to receive message from the commit phase itself
        safety_rules_container,
        signers,
        state_computer,
        epoch_state.verifier,
        commit_phase,
    )
}

fn prepare_executed_blocks_with_ledger_info(
    signer: &ValidatorSigner,
    executed_hash: HashValue,
    consensus_hash: HashValue,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    let genesis_qc = certificate_for_genesis();
    let block = Block::new_proposal(vec![], 1, 1, genesis_qc, signer);
    let compute_result = StateComputeResult::new(
        executed_hash,
        vec![], // dummy subtree
        0,
        vec![],
        0,
        None,
        vec![],
        vec![],
        vec![],
    );

    let li = LedgerInfo::new(
        block.gen_block_info(
            compute_result.root_hash(),
            compute_result.version(),
            compute_result.epoch_state().clone(),
        ),
        consensus_hash,
    );

    let mut li_sig = LedgerInfoWithSignatures::new(
        li.clone(),
        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
    );

    li_sig.add_signature(signer.author(), signer.sign(&li));

    let executed_block = ExecutedBlock::new(block, compute_result);

    (vec![executed_block], li_sig)
}

fn prepare_executed_blocks_with_executed_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    prepare_executed_blocks_with_ledger_info(
        signer,
        HashValue::random(),
        HashValue::from_u64(0xbeef),
    )
}

fn prepare_executed_blocks_with_ordered_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    prepare_executed_blocks_with_ledger_info(
        signer,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        *ACCUMULATOR_PLACEHOLDER_HASH,
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
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            mut msg_tx,
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
        ) = prepare_commit_phase();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(prepare_executed_blocks_with_ordered_ledger_info(
                    &signers[0],
                ))
                .await
                .ok();

            match self_loop_rx.next().await {
                Some(Event::Message(_, msg)) => {
                    // send the message into self loop
                    msg_tx.send(msg).await.ok();
                }
                _ => {
                    panic!("We are expecting a commit vote message.");
                }
            };

            // it commits the block
            assert!(matches!(commit_result_rx.next().await, Some((_, _)),));
            // and it sends a commit decision
            assert!(matches!(
                self_loop_rx.next().await,
                Some(Event::Message(_, ConsensusMsg::CommitDecisionMsg(_))),
            ));
        });
    }

    /// happy path 2 - getting a good commit decision
    #[test]
    fn test_happy_path_commit_decision() {
        let mut runtime = consensus_runtime();
        let (
            mut commit_tx,
            mut msg_tx,
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
        ) = prepare_commit_phase();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(prepare_executed_blocks_with_ordered_ledger_info(
                    &signers[0],
                ))
                .await
                .ok();

            // it sends itself a commit vote
            match self_loop_rx.next().await {
                Some(Event::Message(_, ConsensusMsg::CommitVoteMsg(request))) => {
                    let commit_vote = *request;
                    let mut signatures = BTreeMap::<AccountAddress, Ed25519Signature>::new();
                    signatures.insert(commit_vote.author(), commit_vote.signature().clone());
                    // construct a good commit decision from request
                    let commit_decision = ConsensusMsg::CommitDecisionMsg(Box::new(
                        CommitDecision::new(LedgerInfoWithSignatures::new(
                            commit_vote.ledger_info().clone(),
                            signatures,
                        )),
                    ));

                    msg_tx.send(commit_decision).await.ok();
                }
                _ => {
                    panic!("We are expecting a commit vote message.");
                }
            };

            // it commits the block
            assert!(commit_result_rx.next().await.is_some());
            // and it sends a commit decision
            assert!(matches!(
                self_loop_rx.next().await,
                Some(Event::Message(_, ConsensusMsg::CommitDecisionMsg(_))),
            ));
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
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            state_computer,
            _validator,
            commit_phase,
        ) = prepare_commit_phase();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            let genesis_qc = certificate_for_genesis();
            let block = Block::new_proposal(vec![], 1, 1, genesis_qc, signers.first().unwrap());
            let compute_result = state_computer
                .compute(&block, *ACCUMULATOR_PLACEHOLDER_HASH)
                .unwrap();

            // bad blocks
            commit_tx
                .send((
                    vec![ExecutedBlock::new(block.clone(), compute_result)],
                    LedgerInfoWithSignatures::new(
                        LedgerInfo::new(
                            block.gen_block_info(*ACCUMULATOR_PLACEHOLDER_HASH, 0, None),
                            *ACCUMULATOR_PLACEHOLDER_HASH,
                        ),
                        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
                    ),
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
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
        ) = prepare_commit_phase();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            commit_tx
                .send(prepare_executed_blocks_with_ordered_ledger_info(
                    &signers[0],
                ))
                .await
                .ok();

            // it sends itself a commit vote
            let self_msg = self_loop_rx.next().await;

            assert!(matches!(self_msg, Some(Event::Message(_, _),)));

            // send a bad vote

            msg_tx
                .send(ConsensusMsg::CommitVoteMsg(Box::new(
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
            mut commit_result_rx,
            mut self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            commit_phase,
        ) = prepare_commit_phase();

        runtime.spawn(commit_phase.start());

        timed_block_on(&mut runtime, async move {
            // send good commit arguments
            let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);
            commit_tx.send((vecblocks, li_sig.clone())).await.ok();

            // it sends itself a commit vote
            let self_msg = self_loop_rx.next().await;

            assert!(matches!(self_msg, Some(Event::Message(_, _),)));

            // send a bad commit decision without signatures
            msg_tx
                .send(ConsensusMsg::CommitDecisionMsg(Box::new(
                    CommitDecision::new(LedgerInfoWithSignatures::new(
                        li_sig.ledger_info().clone(),
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
    /// negative tests for commit_phase.process_commit_vote
    #[test]
    fn test_commit_phase_process_commit_vote() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
        ) = prepare_commit_phase();

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            let good_ledger_info = LedgerInfo::new(
                vecblocks.last().unwrap().block_info(),
                li_sig.ledger_info().consensus_data_hash(),
            );

            commit_phase.set_blocks(Some(PendingBlocks::new(vecblocks, li_sig)));

            let random_commit_vote = generate_random_commit_vote(signer);

            assert!(matches!(
                commit_phase.process_commit_vote(&random_commit_vote).await,
                Err(Error::InconsistentBlockInfo(_, _))
            ));

            let commit_vote_with_bad_signature = CommitVote::new_with_signature(
                signer.author(),
                good_ledger_info,
                signer.sign(random_commit_vote.ledger_info()),
            );

            assert!(matches!(
                commit_phase
                    .process_commit_vote(&commit_vote_with_bad_signature)
                    .await,
                Err(Error::VerificationError)
            ));
        });
    }

    #[test]
    fn test_commit_phase_process_commit_decision() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
        ) = prepare_commit_phase();

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            let good_ledger_info = LedgerInfo::new(
                vecblocks.last().unwrap().block_info(),
                li_sig.ledger_info().consensus_data_hash(),
            );

            commit_phase.set_blocks(Some(PendingBlocks::new(vecblocks, li_sig)));

            let random_commit_decision = generate_random_commit_decision(signer);

            assert!(matches!(
                commit_phase
                    .process_commit_decision(&random_commit_decision)
                    .await,
                Err(Error::InconsistentBlockInfo(_, _))
            ));

            let commit_decision_with_bad_signature =
                CommitDecision::new(LedgerInfoWithSignatures::new(
                    good_ledger_info,
                    BTreeMap::<AccountAddress, Ed25519Signature>::new(),
                ));

            assert_eq!(
                commit_phase
                    .process_commit_decision(&commit_decision_with_bad_signature)
                    .await,
                Err(Error::VerificationError)
            );
        });
    }

    #[test]
    fn test_commit_phase_check_commit() {
        let mut runtime = consensus_runtime();
        let (
            _commit_tx,
            _msg_tx,
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
        ) = prepare_commit_phase();

        timed_block_on(&mut runtime, async move {
            let signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_executed_ledger_info(signer);

            assert!(commit_phase.blocks().is_none());

            // when blocks is none
            commit_phase.check_commit().await.ok();

            assert!(commit_phase.blocks().is_none());

            commit_phase.set_blocks(Some(PendingBlocks::new(vecblocks.clone(), li_sig.clone())));

            // when blocks is good
            commit_phase.check_commit().await.ok();

            // the block should be consumed
            assert!(commit_phase.blocks().is_none());
            assert_eq!(commit_phase.load_back_pressure(), 1);

            // when block contains bad signatures
            let ledger_info_with_no_sig = LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    vecblocks.last().unwrap().block_info(),
                    li_sig.ledger_info().consensus_data_hash(),
                ),
                BTreeMap::<AccountAddress, Ed25519Signature>::new(), //empty
            );
            commit_phase.set_blocks(Some(PendingBlocks::new(vecblocks, ledger_info_with_no_sig)));
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
            _commit_result_rx,
            _self_loop_rx,
            _safety_rules_container,
            signers,
            _state_computer,
            _validator,
            mut commit_phase,
        ) = prepare_commit_phase();

        timed_block_on(&mut runtime, async move {
            let _signer = &signers[0];

            let (vecblocks, li_sig) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);

            let ledger_info_with_no_sig = LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    vecblocks.last().unwrap().block_info(),
                    li_sig.ledger_info().consensus_data_hash(),
                ),
                BTreeMap::<AccountAddress, Ed25519Signature>::new(), //empty
            );

            // no signatures
            assert!(matches!(
                commit_phase
                    .process_executed_blocks(vecblocks, ledger_info_with_no_sig)
                    .await,
                Err(_),
            ));
        });
    }
}
