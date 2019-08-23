// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        common::Author,
        consensus_types::{
            block::Block,
            proposal_msg::ProposalMsg,
            quorum_cert::QuorumCert,
            sync_info::SyncInfo,
            timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate, TimeoutMsg},
        },
        event_processor::EventProcessor,
        liveness::{
            pacemaker::{ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker},
            pacemaker_timeout_manager::HighestTimeoutCertificates,
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::RotatingProposer,
        },
        network::{BlockRetrievalRequest, BlockRetrievalResponse, ConsensusNetworkImpl},
        network_tests::NetworkPlayground,
        persistent_storage::{PersistentStorage, RecoveryData},
        safety::{
            safety_rules::{ConsensusState, SafetyRules},
            vote_msg::VoteMsg,
        },
        test_utils::{
            consensus_runtime, placeholder_certificate_for_block, placeholder_ledger_info,
            MockStateComputer, MockStorage, MockTransactionManager, TestPayload, TreeInserter,
        },
    },
    util::time_service::{ClockTimeService, TimeService},
};
use channel;
use crypto::{ed25519::*, HashValue, VerifyingKey};
use futures::{
    channel::{mpsc, oneshot},
    compat::Future01CompatExt,
    executor::block_on,
};
use network::{
    proto::BlockRetrievalStatus,
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender},
};
use proto_conv::FromProto;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::runtime::TaskExecutor;
use types::{
    account_address::AccountAddress,
    ledger_info::LedgerInfoWithSignatures,
    validator_signer::ValidatorSigner,
    validator_verifier::{ConsensusVerifier, ValidatorVerifier, VerifyError},
};

/// Auxiliary struct that is setting up node environment for the test.
struct NodeSetup {
    author: Author,
    block_store: Arc<BlockStore<TestPayload>>,
    event_processor: EventProcessor<TestPayload>,
    storage: Arc<MockStorage<TestPayload>>,
    signer: ValidatorSigner<Ed25519PrivateKey>,
    proposer_author: Author,
    peers: Arc<Vec<Author>>,
}

impl NodeSetup {
    fn build_empty_store(
        signer: ValidatorSigner<Ed25519PrivateKey>,
        storage: Arc<dyn PersistentStorage<TestPayload>>,
        initial_data: RecoveryData<TestPayload>,
    ) -> Arc<BlockStore<TestPayload>> {
        let (commit_cb_sender, _commit_cb_receiver) =
            mpsc::unbounded::<LedgerInfoWithSignatures<Ed25519Signature>>();

        Arc::new(block_on(BlockStore::new(
            storage,
            initial_data,
            signer,
            Arc::new(MockStateComputer::new(commit_cb_sender)),
            true,
            10, // max pruned blocks in mem
        )))
    }

    fn create_pacemaker(time_service: Arc<dyn TimeService>) -> Pacemaker {
        let base_timeout = Duration::new(60, 0);
        let time_interval = Box::new(ExponentialTimeInterval::fixed(base_timeout));
        let (pacemaker_timeout_sender, _) = channel::new_test(1_024);
        Pacemaker::new(
            MockStorage::<TestPayload>::start_for_testing()
                .0
                .persistent_liveness_storage(),
            time_interval,
            time_service,
            pacemaker_timeout_sender,
            1,
            HighestTimeoutCertificates::default(),
        )
    }

    fn create_proposer_election(
        author: Author,
    ) -> Box<dyn ProposerElection<TestPayload> + Send + Sync> {
        Box::new(RotatingProposer::new(vec![author], 1))
    }

    fn create_nodes(
        playground: &mut NetworkPlayground,
        executor: TaskExecutor,
        num_nodes: usize,
    ) -> Vec<NodeSetup> {
        let mut signers = vec![];
        let mut peers = vec![];
        for i in 0..num_nodes {
            let signer = ValidatorSigner::random([i as u8; 32]);
            peers.push(signer.author());
            signers.push(signer);
        }
        let proposer_author = peers[0];
        let peers_ref = Arc::new(peers);
        let mut nodes = vec![];
        for signer in signers.iter().take(num_nodes) {
            let (storage, initial_data) = MockStorage::<TestPayload>::start_for_testing();
            nodes.push(Self::new(
                playground,
                executor.clone(),
                signer.clone(),
                proposer_author,
                Arc::clone(&peers_ref),
                storage,
                initial_data,
            ));
        }
        nodes
    }

    fn new(
        playground: &mut NetworkPlayground,
        executor: TaskExecutor,
        signer: ValidatorSigner<Ed25519PrivateKey>,
        proposer_author: Author,
        peers: Arc<Vec<Author>>,
        storage: Arc<MockStorage<TestPayload>>,
        initial_data: RecoveryData<TestPayload>,
    ) -> Self {
        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(8);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);
        let author = signer.author();

        playground.add_node(author, consensus_tx, network_reqs_rx);
        let validator = ValidatorVerifier::new_single(signer.author(), signer.public_key());

        let network = ConsensusNetworkImpl::new(
            signer.author(),
            network_sender,
            network_events,
            Arc::clone(&peers),
            Arc::new(validator),
        );
        let consensus_state = initial_data.state();

        let block_store = Self::build_empty_store(signer.clone(), storage.clone(), initial_data);
        let time_service = Arc::new(ClockTimeService::new(executor.clone()));
        let proposal_generator = ProposalGenerator::new(
            block_store.clone(),
            Arc::new(MockTransactionManager::new()),
            time_service.clone(),
            1,
            true,
        );
        let safety_rules = SafetyRules::new(consensus_state);

        let pacemaker = Self::create_pacemaker(time_service.clone());

        let proposer_election = Self::create_proposer_election(proposer_author);
        let (commit_cb_sender, _commit_cb_receiver) =
            mpsc::unbounded::<LedgerInfoWithSignatures<Ed25519Signature>>();
        let mut event_processor = EventProcessor::new(
            author,
            Arc::clone(&block_store),
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules,
            Arc::new(MockStateComputer::new(commit_cb_sender)),
            Arc::new(MockTransactionManager::new()),
            network,
            storage.clone(),
            time_service,
            true,
        );
        block_on(event_processor.start());
        Self {
            author,
            block_store,
            event_processor,
            storage,
            signer,
            proposer_author,
            peers,
        }
    }

    pub fn restart(self, playground: &mut NetworkPlayground, executor: TaskExecutor) -> Self {
        let recover_data = self
            .storage
            .get_recovery_data()
            .unwrap_or_else(|e| panic!("fail to restart due to: {}", e));
        Self::new(
            playground,
            executor,
            self.signer,
            self.proposer_author,
            self.peers,
            self.storage,
            recover_data,
        )
    }
}

#[test]
fn basic_new_rank_event_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let node = &nodes[0];
    let genesis = node.block_store.root();
    let mut inserter = TreeInserter::new(node.block_store.clone());
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
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
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let pending_proposals = pending_messages
            .into_iter()
            .filter(|m| m.1.has_proposal())
            .map(|mut m| ProposalMsg::<TestPayload>::from_proto(m.1.take_proposal()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pending_proposals.len(), 1);
        assert_eq!(pending_proposals[0].proposal.round(), new_round,);
        assert_eq!(
            pending_proposals[0]
                .proposal
                .quorum_cert()
                .certified_block_id(),
            genesis.id()
        );
        assert_eq!(pending_proposals[0].proposer(), node.author);

        // Simulate a case with a1 receiving enough votes for a QC: a new proposal
        // should be a child of a1 and carry its QC.
        let vote_msg = VoteMsg::new(
            a1.id(),
            node.block_store.get_state_for_block(a1.id()).unwrap(),
            a1.round(),
            a1.quorum_cert().certified_parent_block_id(),
            a1.quorum_cert().certified_parent_block_round(),
            a1.quorum_cert().certified_grandparent_block_id(),
            a1.quorum_cert().certified_grandparent_block_round(),
            node.block_store.signer().author(),
            placeholder_ledger_info(),
            node.block_store.signer(),
        );
        node.block_store.insert_vote_and_qc(vote_msg, 0);
        node.event_processor
            .process_new_round_event(NewRoundEvent {
                round: 2,
                reason: NewRoundReason::QCReady,
                timeout: Duration::new(5, 0),
            })
            .await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let pending_proposals = pending_messages
            .into_iter()
            .filter(|m| m.1.has_proposal())
            .map(|mut m| ProposalMsg::<TestPayload>::from_proto(m.1.take_proposal()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pending_proposals.len(), 1);
        assert_eq!(pending_proposals[0].proposal.round(), 2);
        assert_eq!(pending_proposals[0].proposal.parent_id(), a1.id());
        assert_eq!(pending_proposals[0].proposal.height(), 2);
        assert_eq!(
            pending_proposals[0]
                .proposal
                .quorum_cert()
                .certified_block_id(),
            a1.id()
        );
    });
}

#[test]
/// If the proposal is valid, a vote should be sent
fn process_successful_proposal_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let node = &mut nodes[1];

    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    block_on(async move {
        let proposal = Block::make_block(
            genesis.as_ref(),
            vec![1],
            1,
            1,
            genesis_qc.clone(),
            node.block_store.signer(),
        );
        let proposal_id = proposal.id();
        node.event_processor.process_proposed_block(proposal).await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        let pending_for_proposer = pending_messages
            .into_iter()
            .filter(|m| m.1.has_vote() && m.0 == node.author)
            .map(|mut m| VoteMsg::from_proto(m.1.take_vote()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pending_for_proposer.len(), 1);
        assert_eq!(pending_for_proposer[0].author(), node.author);
        assert_eq!(pending_for_proposer[0].proposed_block_id(), proposal_id);
        assert_eq!(
            *node.storage.shared_storage.state.lock().unwrap(),
            ConsensusState::new(1, 0),
        );
    });
}

#[test]
/// If the proposal does not pass voting rules,
/// No votes are sent, but the block is still added to the block tree.
fn process_old_proposal_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let node = &mut nodes[1];
    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let new_block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let new_block_id = new_block.id();
    let old_block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        2,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let old_block_id = old_block.id();
    block_on(async move {
        node.event_processor.process_proposed_block(new_block).await;
        node.event_processor.process_proposed_block(old_block).await;
        let pending_messages = playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        let pending_for_me = pending_messages
            .into_iter()
            .filter(|m| m.1.has_vote() && m.0 == node.author)
            .map(|mut m| VoteMsg::from_proto(m.1.take_vote()).unwrap())
            .collect::<Vec<_>>();
        // just the new one
        assert_eq!(pending_for_me.len(), 1);
        assert_eq!(pending_for_me[0].proposed_block_id(), new_block_id);
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
    let mut playground = NetworkPlayground::new(runtime.executor());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.executor(), 1)
        .pop()
        .unwrap();
    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let correct_block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let block_skip_round = Block::make_block(
        genesis.as_ref(),
        vec![1],
        2,
        2,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    block_on(async move {
        let bad_proposal = ProposalMsg::<TestPayload> {
            proposal: block_skip_round,
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        };
        assert_eq!(
            node.event_processor
                .pre_process_proposal(bad_proposal)
                .await,
            None
        );
        let good_proposal = ProposalMsg::<TestPayload> {
            proposal: correct_block.clone(),
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        };
        assert_eq!(
            node.event_processor
                .pre_process_proposal(good_proposal.clone())
                .await,
            Some(good_proposal.proposal)
        );
    });
}

#[test]
/// Ensure that after new round messages are sent that the receivers have the latest
/// quorum certificate
fn process_new_round_msg_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let non_proposer = nodes.pop().unwrap();
    let mut static_proposer = nodes.pop().unwrap();

    let genesis = non_proposer.block_store.root();
    let block_0 = non_proposer
        .block_store
        .create_block(genesis, vec![1], 1, 1);
    let block_0_id = block_0.id();
    block_on(
        non_proposer
            .block_store
            .execute_and_insert_block(block_0.clone()),
    )
    .unwrap();
    block_on(
        static_proposer
            .block_store
            .execute_and_insert_block(block_0.clone()),
    )
    .unwrap();

    // Populate block_0 and a quorum certificate for block_0 on non_proposer
    let block_0_quorum_cert = placeholder_certificate_for_block(
        vec![&static_proposer.signer, &non_proposer.signer],
        block_0_id,
        1,
        block_0.quorum_cert().certified_block_id(),
        block_0.quorum_cert().certified_block_round(),
        block_0.quorum_cert().certified_parent_block_id(),
        block_0.quorum_cert().certified_parent_block_round(),
    );
    non_proposer
        .block_store
        .insert_single_quorum_cert(block_0_quorum_cert.clone())
        .unwrap();
    assert_eq!(
        static_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block_round(),
        0
    );
    assert_eq!(
        non_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block_round(),
        1
    );

    // As the static proposer processes the new round message it should learn about
    // block_0_quorum_cert at round 1.
    block_on(static_proposer.event_processor.process_remote_timeout_msg(
        TimeoutMsg::new(
            SyncInfo::new(
                block_0_quorum_cert,
                QuorumCert::certificate_for_genesis(),
                None,
            ),
            PacemakerTimeout::new(2, &non_proposer.signer, None),
            &non_proposer.signer,
        ),
        2,
    ));
    assert_eq!(
        static_proposer
            .block_store
            .highest_quorum_cert()
            .certified_block_round(),
        1
    );
}

#[test]
/// We don't vote for proposals that comes from proposers that are not valid proposers for round
fn process_proposer_mismatch_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let incorrect_proposer = nodes.pop().unwrap();
    let mut node = nodes.pop().unwrap();
    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let correct_block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let block_incorrect_proposer = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        incorrect_proposer.block_store.signer(),
    );
    block_on(async move {
        let bad_proposal = ProposalMsg::<TestPayload> {
            proposal: block_incorrect_proposer,
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        };
        assert_eq!(
            node.event_processor
                .pre_process_proposal(bad_proposal)
                .await,
            None
        );
        let good_proposal = ProposalMsg::<TestPayload> {
            proposal: correct_block.clone(),
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        };

        assert_eq!(
            node.event_processor
                .pre_process_proposal(good_proposal.clone())
                .await,
            Some(good_proposal.proposal)
        );
    });
}

#[test]
/// We allow to 'skips' round if proposal carries timeout certificate for next round
fn process_timeout_certificate_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // In order to observe the votes we're going to check proposal processing on the non-proposer
    // node (which will send the votes to the proposer).
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.executor(), 1)
        .pop()
        .unwrap();
    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let correct_block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let block_skip_round = Block::make_block(
        genesis.as_ref(),
        vec![1],
        2,
        2,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let tc =
        PacemakerTimeoutCertificate::new(1, vec![PacemakerTimeout::new(1, &node.signer, None)]);
    block_on(async move {
        let skip_round_proposal = ProposalMsg::<TestPayload> {
            proposal: block_skip_round,
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), Some(tc)),
        };
        assert_eq!(
            node.event_processor
                .pre_process_proposal(skip_round_proposal.clone())
                .await,
            Some(skip_round_proposal.proposal)
        );
        let old_good_proposal = ProposalMsg::<TestPayload> {
            proposal: correct_block.clone(),
            sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
        };
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
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.executor(), 1)
        .pop()
        .unwrap();
    let genesis = node.block_store.root();
    let mut inserter = TreeInserter::new(node.block_store.clone());
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    let vote_msg = VoteMsg::new(
        a1.id(),
        node.block_store.get_state_for_block(a1.id()).unwrap(),
        a1.round(),
        a1.quorum_cert().certified_parent_block_id(),
        a1.quorum_cert().certified_parent_block_round(),
        a1.quorum_cert().certified_grandparent_block_id(),
        a1.quorum_cert().certified_parent_block_round(),
        node.block_store.signer().author(),
        placeholder_ledger_info(),
        node.block_store.signer(),
    );
    block_on(async move {
        node.event_processor.process_vote(vote_msg, 1).await;
        // The new QC is aggregated
        assert_eq!(
            node.block_store.highest_quorum_cert().certified_block_id(),
            a1.id()
        );
    });
    block_on(runtime.shutdown_now().compat()).unwrap();
}

#[test]
fn process_block_retrieval() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.executor(), 1)
        .pop()
        .unwrap();

    let genesis = node.block_store.root();
    let genesis_qc = QuorumCert::certificate_for_genesis();

    let block = Block::make_block(
        genesis.as_ref(),
        vec![1],
        1,
        1,
        genesis_qc.clone(),
        node.block_store.signer(),
    );
    let block_id = block.id();

    block_on(async move {
        node.event_processor
            .process_certificates(block.quorum_cert(), None)
            .await;
        node.event_processor.process_proposed_block(block).await;

        // first verify that we can retrieve the block if it's in the tree
        let (tx1, rx1) = oneshot::channel();
        let single_block_request = BlockRetrievalRequest {
            block_id,
            num_blocks: 1,
            response_sender: tx1,
        };
        node.event_processor
            .process_block_retrieval(single_block_request)
            .await;
        match rx1.await {
            Ok(BlockRetrievalResponse { status, blocks }) => {
                assert_eq!(status, BlockRetrievalStatus::SUCCEEDED);
                assert_eq!(block_id, blocks.get(0).unwrap().id());
            }
            _ => panic!("block retrieval failure"),
        }

        // verify that if a block is not there, return ID_NOT_FOUND
        let (tx2, rx2) = oneshot::channel();
        let missing_block_request = BlockRetrievalRequest {
            block_id: HashValue::random(),
            num_blocks: 1,
            response_sender: tx2,
        };
        node.event_processor
            .process_block_retrieval(missing_block_request)
            .await;
        match rx2.await {
            Ok(BlockRetrievalResponse { status, blocks }) => {
                assert_eq!(status, BlockRetrievalStatus::ID_NOT_FOUND);
                assert!(blocks.is_empty());
            }
            _ => panic!("block retrieval failure"),
        }

        // if asked for many blocks, return NOT_ENOUGH_BLOCKS
        let (tx3, rx3) = oneshot::channel();
        let many_block_request = BlockRetrievalRequest {
            block_id,
            num_blocks: 3,
            response_sender: tx3,
        };
        node.event_processor
            .process_block_retrieval(many_block_request)
            .await;
        match rx3.await {
            Ok(BlockRetrievalResponse { status, blocks }) => {
                assert_eq!(status, BlockRetrievalStatus::NOT_ENOUGH_BLOCKS);
                assert_eq!(block_id, blocks.get(0).unwrap().id());
                assert_eq!(node.block_store.root().id(), blocks.get(1).unwrap().id());
            }
            _ => panic!("block retrieval failure"),
        }
    });
}

#[test]
/// rebuild a node from previous storage without violating safety guarantees.
fn basic_restart_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut node = NodeSetup::create_nodes(&mut playground, runtime.executor(), 1)
        .pop()
        .unwrap();
    let mut inserter = TreeInserter::new(node.block_store.clone());
    let node_mut = &mut node;

    let genesis = node_mut.block_store.root();
    let mut proposals = Vec::new();
    let num_proposals = 100;
    // insert a few successful proposals
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    proposals.push(a1);
    for i in 2..=num_proposals {
        let parent = proposals.last().unwrap();
        let proposal = inserter.insert_block(parent, i);
        proposals.push(proposal);
    }
    for proposal in &proposals {
        block_on(
            node_mut
                .event_processor
                .process_certificates(proposal.quorum_cert(), None),
        );
        block_on(
            node_mut
                .event_processor
                .process_proposed_block(Block::clone(proposal)),
        );
    }
    // verify after restart we recover the data
    node = node.restart(&mut playground, runtime.executor());
    assert_eq!(
        node.event_processor.consensus_state(),
        ConsensusState::new(num_proposals, num_proposals - 2),
    );
    for block in proposals {
        assert_eq!(node.block_store.block_exists(block.id()), true);
    }
}

#[test]
/// Generate a NIL vote extending HQC upon timeout if no votes have been sent in the round.
fn nil_vote_on_timeout() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // It needs 2 nodes to test network message.
    let mut nodes = NodeSetup::create_nodes(&mut playground, runtime.executor(), 2);
    let node = &mut nodes[0];
    let genesis_id = node.block_store.root().id();
    block_on(async move {
        // Process the outgoing timeout and verify that the TimeoutMsg contains a NIL vote that
        // extends genesis
        node.event_processor.process_local_timeout(1).await;
        let timeout_msg = TimeoutMsg::from_proto(
            playground
                .wait_for_messages(1, NetworkPlayground::timeout_msg_only)
                .await[0]
                .1
                .take_timeout_msg(),
        )
        .unwrap();
        assert_eq!(timeout_msg.pacemaker_timeout().round(), 1);
        let vote_msg = timeout_msg.pacemaker_timeout().vote_msg().unwrap().clone();
        assert_eq!(vote_msg.round(), 1);
        assert_eq!(vote_msg.parent_block_id(), genesis_id);
    });
}

//
// Fuzzing
// =======
// These are fuzzing related functions
//

#[derive(Clone)]
pub struct FakeVerifier<PublicKey> {
    author_to_public_keys: HashMap<AccountAddress, PublicKey>,
    quorum_size: usize,
}

impl<PublicKey> FakeVerifier<PublicKey> {
    pub fn new_with_quorum_size(
        author_to_public_keys: HashMap<AccountAddress, PublicKey>,
        quorum_size: usize,
    ) -> Self {
        Self {
            author_to_public_keys,
            quorum_size,
        }
    }
}

impl<PublicKey: VerifyingKey> ConsensusVerifier<PublicKey> for FakeVerifier<PublicKey> {
    fn verify_signature(
        &self,
        _author: AccountAddress,
        _hash: HashValue,
        _signature: &PublicKey::SignatureMaterial,
    ) -> std::result::Result<(), VerifyError> {
        Ok(())
    }

    fn verify_aggregated_signature<T>(
        &self,
        _hash: HashValue,
        _aggregated_signature: &HashMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        Ok(())
    }

    fn batch_verify_aggregated_signature<T>(
        &self,
        _hash: HashValue,
        _aggregated_signature: &HashMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        Ok(())
    }

    fn check_num_of_signatures<T>(
        &self,
        _aggregated_signature: &HashMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        Ok(())
    }

    fn check_keys<T>(
        &self,
        _aggregated_signature: &HashMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        Ok(())
    }

    fn get_public_key(&self, author: AccountAddress) -> Option<PublicKey> {
        self.author_to_public_keys.get(&author).cloned()
    }

    fn get_ordered_account_addresses(&self) -> Vec<AccountAddress> {
        let mut account_addresses: Vec<AccountAddress> =
            self.author_to_public_keys.keys().cloned().collect();
        account_addresses.sort();
        account_addresses
    }

    fn len(&self) -> usize {
        self.author_to_public_keys.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn quorum_size(&self) -> usize {
        self.quorum_size
    }
}
