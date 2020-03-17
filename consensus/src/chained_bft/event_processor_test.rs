// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        event_processor::EventProcessor,
        liveness::{
            pacemaker::{ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker},
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::RotatingProposer,
        },
        network::{IncomingBlockRetrievalRequest, NetworkSender, NetworkTask},
        network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
        network_tests::NetworkPlayground,
        persistent_liveness_storage::RecoveryData,
        test_utils::{
            self, consensus_runtime, MockStateComputer, MockStorage, MockTransactionManager,
            TestPayload, TreeInserter,
        },
    },
    util::time_service::{ClockTimeService, TimeService},
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block::{
        block_test_utils::{
            certificate_for_genesis, gen_test_certificate, placeholder_ledger_info,
        },
        Block,
    },
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalStatus},
    common::Author,
    proposal_msg::ProposalMsg,
    sync_info::SyncInfo,
    timeout::Timeout,
    timeout_certificate::TimeoutCertificate,
    vote::Vote,
    vote_data::VoteData,
    vote_msg::VoteMsg,
};
use futures::{
    channel::{mpsc, oneshot},
    executor::block_on,
};
use libra_crypto::HashValue;
use libra_types::{
    block_info::BlockInfo,
    crypto_proxies::{random_validator_verifier, ValidatorVerifier},
    ledger_info::LedgerInfoWithSignatures,
    validator_signer::ValidatorSigner,
};
use network::peer_manager::{
    conn_status_channel, ConnectionRequestSender, PeerManagerRequestSender,
};
use safety_rules::{ConsensusState, PersistentSafetyStorage as SafetyStorage, SafetyRulesManager};
use std::{num::NonZeroUsize, sync::Arc, time::Duration};
use tokio::runtime::Handle;

/// Auxiliary struct that is setting up node environment for the test.
pub struct NodeSetup {
    block_store: Arc<BlockStore<TestPayload>>,
    event_processor: EventProcessor<TestPayload>,
    storage: Arc<MockStorage<TestPayload>>,
    signer: ValidatorSigner,
    proposer_author: Author,
    validators: Arc<ValidatorVerifier>,
    safety_rules_manager: SafetyRulesManager<TestPayload>,
}

impl NodeSetup {
    fn create_pacemaker(time_service: Arc<dyn TimeService>) -> Pacemaker {
        let base_timeout = Duration::new(60, 0);
        let time_interval = Box::new(ExponentialTimeInterval::fixed(base_timeout));
        let (pacemaker_timeout_sender, _) = channel::new_test(1_024);
        Pacemaker::new(time_interval, time_service, pacemaker_timeout_sender)
    }

    fn create_proposer_election(
        author: Author,
    ) -> Box<dyn ProposerElection<TestPayload> + Send + Sync> {
        Box::new(RotatingProposer::new(vec![author], 1))
    }

    fn create_nodes(
        playground: &mut NetworkPlayground,
        executor: Handle,
        num_nodes: usize,
    ) -> Vec<Self> {
        let (signers, validators) = random_validator_verifier(num_nodes, None, false);
        let proposer_author = signers[0].author();
        let mut nodes = vec![];
        for signer in signers.iter().take(num_nodes) {
            let (initial_data, storage) =
                MockStorage::<TestPayload>::start_for_testing((&validators).into());

            let safety_rules_manager = SafetyRulesManager::new_local(
                signer.author(),
                SafetyStorage::in_memory(signer.private_key().clone()),
            );

            nodes.push(Self::new(
                playground,
                executor.clone(),
                signer.clone(),
                proposer_author,
                storage,
                initial_data,
                safety_rules_manager,
            ));
        }
        nodes
    }

    fn new(
        playground: &mut NetworkPlayground,
        executor: Handle,
        signer: ValidatorSigner,
        proposer_author: Author,
        storage: Arc<MockStorage<TestPayload>>,
        initial_data: RecoveryData<TestPayload>,
        safety_rules_manager: SafetyRulesManager<TestPayload>,
    ) -> Self {
        let epoch_info = initial_data.epoch_info();
        let validators = epoch_info.verifier.clone();
        let (network_reqs_tx, network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (connection_reqs_tx, _) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
        let (_, conn_status_rx) = conn_status_channel::new();
        let network_sender = ConsensusNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
            conn_mgr_reqs_tx,
        );
        let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_status_rx);
        let author = signer.author();

        playground.add_node(author, consensus_tx, network_reqs_rx, conn_mgr_reqs_rx);

        let (self_sender, self_receiver) = channel::new_test(8);
        let network = NetworkSender::new(author, network_sender, self_sender, validators.clone());

        let (task, _receiver) = NetworkTask::<TestPayload>::new(network_events, self_receiver);

        executor.spawn(task.start());
        let last_vote_sent = initial_data.last_vote();
        let (commit_cb_sender, _commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();
        let (state_sync_client, _state_sync) = mpsc::unbounded();
        let state_computer = Arc::new(MockStateComputer::new(
            state_sync_client,
            commit_cb_sender,
            Arc::clone(&storage),
            None,
        ));

        let block_store = Arc::new(BlockStore::new(
            storage.clone(),
            initial_data,
            state_computer,
            10, // max pruned blocks in mem
        ));

        let time_service = Arc::new(ClockTimeService::new(executor));

        let proposal_generator = ProposalGenerator::new(
            author,
            block_store.clone(),
            Box::new(MockTransactionManager::new(None)),
            time_service.clone(),
            1,
        );

        let pacemaker = Self::create_pacemaker(time_service.clone());

        let proposer_election = Self::create_proposer_election(proposer_author);

        let mut event_processor = EventProcessor::new(
            epoch_info,
            Arc::clone(&block_store),
            last_vote_sent,
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules_manager.client(),
            network,
            Box::new(MockTransactionManager::new(None)),
            storage.clone(),
            time_service,
        );
        block_on(event_processor.start());
        Self {
            block_store,
            event_processor,
            storage,
            signer,
            proposer_author,
            validators,
            safety_rules_manager,
        }
    }

    pub fn restart(self, playground: &mut NetworkPlayground, executor: Handle) -> Self {
        let recover_data = self
            .storage
            .try_start()
            .unwrap_or_else(|e| panic!("fail to restart due to: {}", e));
        Self::new(
            playground,
            executor,
            self.signer,
            self.proposer_author,
            self.storage,
            recover_data,
            self.safety_rules_manager,
        )
    }
}

#[test]
fn basic_new_rank_event_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let node1 = nodes.pop().unwrap();
    let node = &mut nodes[0];
    let genesis = node.block_store.root();
    let mut inserter = TreeInserter::new_with_store(node.signer.clone(), node.block_store.clone());
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    block_on(async move {
        let new_round = 1;
        node.event_processor
            .process_new_round_event(NewRoundEvent {
                round: new_round,
                reason: NewRoundReason::QCReady,
                timeout: Duration::new(5, 0),
            })
            .await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        let pending_proposals: Vec<ProposalMsg<TestPayload>> = pending_messages
            .into_iter()
            .filter_map(|m| match m.1 {
                ConsensusMsg::ProposalMsg(proposal) => Some(*proposal),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(pending_proposals.len(), 1);
        assert_eq!(pending_proposals[0].proposal().round(), new_round);
        assert_eq!(
            pending_proposals[0]
                .proposal()
                .quorum_cert()
                .certified_block()
                .id(),
            genesis.id()
        );
        assert_eq!(pending_proposals[0].proposer(), node.signer.author());

        // Simulate a case with a1 receiving enough votes for a QC: a new proposal
        // should be a child of a1 and carry its QC.
        let vote = Vote::new(
            VoteData::new(
                a1.block().gen_block_info(
                    a1.compute_result().state_id(),
                    a1.compute_result().version(),
                    a1.compute_result().validators().clone(),
                ),
                a1.quorum_cert().certified_block().clone(),
            ),
            node.signer.author(),
            placeholder_ledger_info(),
            &node.signer,
        );
        let vote1 = Vote::new(
            VoteData::new(
                a1.block().gen_block_info(
                    a1.compute_result().state_id(),
                    a1.compute_result().version(),
                    a1.compute_result().validators().clone(),
                ),
                a1.quorum_cert().certified_block().clone(),
            ),
            node1.signer.author(),
            placeholder_ledger_info(),
            &node1.signer,
        );
        // Adding votes to form a QC
        node.event_processor.add_vote(&vote).await.unwrap();
        node.event_processor.add_vote(&vote1).await.unwrap();

        node.event_processor
            .process_new_round_event(NewRoundEvent {
                round: 2,
                reason: NewRoundReason::QCReady,
                timeout: Duration::new(5, 0),
            })
            .await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        let pending_proposals: Vec<ProposalMsg<TestPayload>> = pending_messages
            .into_iter()
            .filter_map(|m| match m.1 {
                ConsensusMsg::ProposalMsg(proposal) => Some(*proposal),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(pending_proposals.len(), 1);
        assert_eq!(pending_proposals[0].proposal().round(), 2);
        assert_eq!(pending_proposals[0].proposal().parent_id(), a1.id());
        assert_eq!(
            pending_proposals[0]
                .proposal()
                .quorum_cert()
                .certified_block()
                .id(),
            a1.id()
        );
    });
}

#[test]
/// If the proposal is valid, a vote should be sent
fn process_successful_proposal_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let node = &mut nodes[1];

    let genesis_qc = certificate_for_genesis();
    block_on(async move {
        let proposal = Block::new_proposal(vec![1], 1, 1, genesis_qc.clone(), &node.signer);
        let proposal_id = proposal.id();
        node.event_processor.process_proposed_block(proposal).await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        let pending_for_proposer = pending_messages
            .into_iter()
            .filter_map(|m| {
                if m.0 != node.signer.author() {
                    return None;
                }

                match m.1 {
                    ConsensusMsg::VoteMsg(vote_msg) => Some(*vote_msg),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(pending_for_proposer.len(), 1);
        assert_eq!(
            pending_for_proposer[0].vote().author(),
            node.signer.author()
        );
        assert_eq!(
            pending_for_proposer[0].vote().vote_data().proposed().id(),
            proposal_id
        );
        assert_eq!(
            node.event_processor.safety_rules.consensus_state().unwrap(),
            ConsensusState::new(1, 1, 0),
        );
    });
}

#[test]
/// If the proposal does not pass voting rules,
/// No votes are sent, but the block is still added to the block tree.
fn process_old_proposal_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let node = &mut nodes[1];
    let genesis_qc = certificate_for_genesis();
    let new_block = Block::new_proposal(vec![1], 1, 1, genesis_qc.clone(), &node.signer);
    let new_block_id = new_block.id();
    let old_block = Block::new_proposal(vec![1], 1, 2, genesis_qc, &node.signer);
    let old_block_id = old_block.id();
    block_on(async move {
        node.event_processor.process_proposed_block(new_block).await;
        node.event_processor.process_proposed_block(old_block).await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        let pending_for_me = pending_messages
            .into_iter()
            .filter_map(|m| {
                if m.0 != node.signer.author() {
                    return None;
                }

                match m.1 {
                    ConsensusMsg::VoteMsg(vote_msg) => Some(*vote_msg),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        // just the new one
        assert_eq!(pending_for_me.len(), 1);
        assert_eq!(
            pending_for_me[0].vote().vote_data().proposed().id(),
            new_block_id
        );
        assert!(node.block_store.get_block(old_block_id).is_some());
    });
}

#[test]
/// We don't vote for proposals that 'skips' rounds
/// After that when we then receive proposal for correct round, we vote for it
/// Basically it checks that adversary can not send proposal and skip rounds violating pacemaker
/// rules
fn process_round_mismatch_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 1)
        .pop()
        .unwrap();
    let genesis_qc = certificate_for_genesis();
    let correct_block = Block::new_proposal(vec![1], 1, 1, genesis_qc.clone(), &node.signer);
    let block_skip_round = Block::new_proposal(vec![1], 2, 2, genesis_qc.clone(), &node.signer);
    block_on(async move {
        let bad_proposal = ProposalMsg::<TestPayload>::new(
            block_skip_round,
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        );
        assert_eq!(
            node.event_processor
                .pre_process_proposal(bad_proposal)
                .await,
            None
        );
        let good_proposal = ProposalMsg::<TestPayload>::new(
            correct_block.clone(),
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        );
        assert_eq!(
            node.event_processor
                .pre_process_proposal(good_proposal.clone())
                .await,
            Some(good_proposal.take_proposal())
        );
    });
}

#[test]
/// Ensure that after the vote messages are broadcasted upon timeout, the receivers
/// have the highest quorum certificate (carried by the SyncInfo of the vote message)
fn process_vote_timeout_msg_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let non_proposer = nodes.pop().unwrap();
    let mut static_proposer = nodes.pop().unwrap();

    let qc = non_proposer.block_store.highest_quorum_cert();
    let block_0 = Block::new_proposal(vec![1], 1, 1, qc.as_ref().clone(), &non_proposer.signer);
    non_proposer
        .block_store
        .execute_and_insert_block(block_0.clone())
        .unwrap();
    static_proposer
        .block_store
        .execute_and_insert_block(block_0.clone())
        .unwrap();

    let parent_block_info = block_0.quorum_cert().certified_block();
    // Populate block_0 and a quorum certificate for block_0 on non_proposer
    let block_0_quorum_cert = gen_test_certificate(
        vec![&static_proposer.signer, &non_proposer.signer],
        block_0.gen_block_info(
            parent_block_info.executed_state_id(),
            parent_block_info.version(),
            parent_block_info.next_validator_set().cloned(),
        ),
        parent_block_info.clone(),
        None,
    );
    non_proposer
        .block_store
        .insert_single_quorum_cert(block_0_quorum_cert.clone())
        .unwrap();
    assert_eq!(
        static_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block()
            .round(),
        0
    );
    assert_eq!(
        non_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block()
            .round(),
        1
    );

    // As the static proposer processes the the vote message it should learn about the
    // block_0_quorum_cert at round 1.
    let dummy_vote_data = VoteData::new(BlockInfo::random(1), BlockInfo::random(0));

    let mut vote_on_timeout = Vote::new(
        dummy_vote_data,
        non_proposer.signer.author(),
        placeholder_ledger_info(),
        &non_proposer.signer,
    );
    let signature = vote_on_timeout.timeout().sign(&non_proposer.signer);
    vote_on_timeout.add_timeout_signature(signature);

    let vote_msg_on_timeout = VoteMsg::new(
        vote_on_timeout,
        SyncInfo::new(block_0_quorum_cert, certificate_for_genesis(), None),
    );
    block_on(
        static_proposer
            .event_processor
            .process_vote(vote_msg_on_timeout),
    );

    assert_eq!(
        static_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block()
            .round(),
        1
    );
}

#[test]
/// We don't vote for proposals that comes from proposers that are not valid proposers for round
fn process_proposer_mismatch_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let incorrect_proposer = nodes.pop().unwrap();
    let mut node = nodes.pop().unwrap();
    let genesis_qc = certificate_for_genesis();
    let correct_block = Block::new_proposal(vec![1], 1, 1, genesis_qc.clone(), &node.signer);
    let block_incorrect_proposer = Block::new_proposal(
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        &incorrect_proposer.signer,
    );
    block_on(async move {
        let bad_proposal = ProposalMsg::<TestPayload>::new(
            block_incorrect_proposer,
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        );
        assert_eq!(
            node.event_processor
                .pre_process_proposal(bad_proposal)
                .await,
            None
        );
        let good_proposal = ProposalMsg::<TestPayload>::new(
            correct_block.clone(),
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        );

        assert_eq!(
            node.event_processor
                .pre_process_proposal(good_proposal.clone())
                .await,
            Some(good_proposal.take_proposal())
        );
    });
}

#[test]
/// We allow to 'skip' round if proposal carries timeout certificate for next round
fn process_timeout_certificate_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 1)
        .pop()
        .unwrap();
    let genesis_qc = certificate_for_genesis();
    let correct_block = Block::new_proposal(vec![1], 1, 1, genesis_qc.clone(), &node.signer);
    let block_skip_round = Block::new_proposal(vec![1], 2, 2, genesis_qc.clone(), &node.signer);
    let timeout = Timeout::new(1, 1);
    let timeout_signature = timeout.sign(&node.signer);

    let mut tc = TimeoutCertificate::new(timeout);
    tc.add_signature(node.signer.author(), timeout_signature);

    block_on(async move {
        let skip_round_proposal = ProposalMsg::<TestPayload>::new(
            block_skip_round,
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), Some(tc)),
        );
        assert_eq!(
            node.event_processor
                .pre_process_proposal(skip_round_proposal.clone())
                .await,
            Some(skip_round_proposal.take_proposal())
        );
        let old_good_proposal = ProposalMsg::<TestPayload>::new(
            correct_block.clone(),
            SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        );
        assert_eq!(
            node.event_processor
                .pre_process_proposal(old_good_proposal.clone())
                .await,
            None
        );
    });
}

#[test]
/// Happy path for vote processing:
/// 1) if a new QC is formed and a block is present send a PM event
fn process_votes_basic_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 1)
        .pop()
        .unwrap();
    let genesis = node.block_store.root();
    let mut inserter = TreeInserter::new_with_store(node.signer.clone(), node.block_store.clone());
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);

    let vote_data = VoteData::new(
        BlockInfo::new(
            a1.quorum_cert().certified_block().epoch(),
            a1.round(),
            a1.id(),
            a1.compute_result().state_id(),
            a1.compute_result().version(),
            a1.timestamp_usecs(),
            a1.compute_result().validators().clone(),
        ),
        a1.quorum_cert().certified_block().clone(),
    );

    let vote_msg = VoteMsg::new(
        Vote::new(
            vote_data,
            node.signer.author(),
            placeholder_ledger_info(),
            &node.signer,
        ),
        test_utils::placeholder_sync_info(),
    );

    block_on(async move {
        node.event_processor.process_vote(vote_msg).await;
        // The new QC is aggregated
        assert_eq!(
            node.block_store
                .highest_quorum_cert()
                .certified_block()
                .id(),
            a1.id()
        );
    });
}

#[test]
fn process_block_retrieval() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 1)
        .pop()
        .unwrap();

    let genesis_qc = certificate_for_genesis();
    let block = Block::new_proposal(vec![1], 1, 1, genesis_qc, &node.signer);
    let block_id = block.id();

    block_on(async move {
        node.event_processor
            .process_certificates(block.quorum_cert(), None)
            .await
            .expect("Failed to process certificates");
        node.event_processor.process_proposed_block(block).await;

        // first verify that we can retrieve the block if it's in the tree
        let (tx1, rx1) = oneshot::channel();
        let single_block_request = IncomingBlockRetrievalRequest {
            req: BlockRetrievalRequest::new(block_id, 1),
            response_sender: tx1,
        };
        node.event_processor
            .process_block_retrieval(single_block_request)
            .await;
        match rx1.await {
            Ok(Ok(bytes)) => {
                let response = match lcs::from_bytes(&bytes) {
                    Ok(ConsensusMsg::<TestPayload>::BlockRetrievalResponse(resp)) => *resp,
                    _ => panic!("block retrieval failure"),
                };
                assert_eq!(response.status(), BlockRetrievalStatus::Succeeded);
                assert_eq!(response.blocks().get(0).unwrap().id(), block_id);
            }
            _ => panic!("block retrieval failure"),
        }

        // verify that if a block is not there, return ID_NOT_FOUND
        let (tx2, rx2) = oneshot::channel();
        let missing_block_request = IncomingBlockRetrievalRequest {
            req: BlockRetrievalRequest::new(HashValue::random(), 1),
            response_sender: tx2,
        };

        node.event_processor
            .process_block_retrieval(missing_block_request)
            .await;
        match rx2.await {
            Ok(Ok(bytes)) => {
                let response = match lcs::from_bytes(&bytes) {
                    Ok(ConsensusMsg::<TestPayload>::BlockRetrievalResponse(resp)) => *resp,
                    _ => panic!("block retrieval failure"),
                };
                assert_eq!(response.status(), BlockRetrievalStatus::IdNotFound);
                assert!(response.blocks().is_empty());
            }
            _ => panic!("block retrieval failure"),
        }

        // if asked for many blocks, return NOT_ENOUGH_BLOCKS
        let (tx3, rx3) = oneshot::channel();
        let many_block_request = IncomingBlockRetrievalRequest {
            req: BlockRetrievalRequest::new(block_id, 3),
            response_sender: tx3,
        };
        node.event_processor
            .process_block_retrieval(many_block_request)
            .await;
        match rx3.await {
            Ok(Ok(bytes)) => {
                let response = match lcs::from_bytes(&bytes) {
                    Ok(ConsensusMsg::<TestPayload>::BlockRetrievalResponse(resp)) => *resp,
                    _ => panic!("block retrieval failure"),
                };
                assert_eq!(response.status(), BlockRetrievalStatus::NotEnoughBlocks);
                assert_eq!(block_id, response.blocks().get(0).unwrap().id());
                assert_eq!(
                    node.block_store.root().id(),
                    response.blocks().get(1).unwrap().id()
                );
            }
            _ => panic!("block retrieval failure"),
        }
    });
}

#[test]
/// rebuild a node from previous storage without violating safety guarantees.
fn basic_restart_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 1)
        .pop()
        .unwrap();
    let mut inserter = TreeInserter::new_with_store(node.signer.clone(), node.block_store.clone());
    let node_mut = &mut node;

    let genesis = node_mut.block_store.root();
    let mut proposals = Vec::new();
    let num_proposals = 100;
    // insert a few successful proposals
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    proposals.push(a1);
    for i in 2..=num_proposals {
        let parent = proposals.last().unwrap();
        let proposal = inserter.insert_block(&parent, i, None);
        proposals.push(proposal);
    }
    for proposal in &proposals {
        block_on(
            node_mut
                .event_processor
                .process_certificates(proposal.quorum_cert(), None),
        )
        .expect("Failed to process certificates");
        block_on(
            node_mut
                .event_processor
                .process_proposed_block(proposal.block().clone()),
        );
    }
    // verify after restart we recover the data
    node = node.restart(&mut playground, runtime.handle().clone());
    assert_eq!(
        node.event_processor.consensus_state(),
        ConsensusState::new(1, num_proposals, num_proposals - 2),
    );
    for block in proposals {
        assert_eq!(node.block_store.block_exists(block.id()), true);
    }
}

#[test]
/// Generate a NIL vote extending HQC upon timeout if no votes have been sent in the round.
fn nil_vote_on_timeout() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // It needs 2 nodes to test network message.
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.handle().clone(), 2);
    let node = &mut nodes[0];
    block_on(async move {
        // Process the outgoing vote message and verify that it contains a round signature
        // and that the vote extends genesis.
        node.event_processor.process_local_timeout(1).await;
        let msg = &playground
            .wait_for_messages(1, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await[0];
        let vote_msg = match &msg.1 {
            ConsensusMsg::VoteMsg(vote_msg) => &*vote_msg,
            _ => panic!("Failed to retrieve VoteMsg"),
        };

        let vote = vote_msg.vote();

        assert!(vote.is_timeout());
        assert_eq!(vote.vote_data().proposed().round(), 1);
        assert_eq!(vote.vote_data().parent().id(), node.block_store.root().id());
    });
}
