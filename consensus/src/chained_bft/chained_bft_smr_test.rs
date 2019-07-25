// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockReader,
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRConfig},
        common::Author,
        liveness::proposer_election::ProposalInfo,
        network::ConsensusNetworkImpl,
        network_tests::NetworkPlayground,
        safety::vote_msg::VoteMsg,
        test_utils::{MockStateComputer, MockStorage, MockTransactionManager, TestPayload},
    },
    state_replication::StateMachineReplication,
};
use channel;
use crypto::hash::CryptoHash;
use futures::{channel::mpsc, executor::block_on, prelude::*};
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
#[allow(unused_imports)]
use nextgen_crypto::ed25519::{compat, *};
use proto_conv::FromProto;
use std::sync::Arc;
use types::{validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier};

use crate::chained_bft::{
    persistent_storage::RecoveryData,
    test_utils::{consensus_runtime, with_smr_id},
};
use config::config::ConsensusProposerType::{self, FixedProposer, RotatingProposer};
use std::{collections::HashMap, time::Duration};
use tokio::runtime;
use types::ledger_info::LedgerInfoWithSignatures;

/// Auxiliary struct that is preparing SMR for the test
struct SMRNode {
    author: Author,
    signer: ValidatorSigner<Ed25519PrivateKey>,
    validator: Arc<ValidatorVerifier<Ed25519PublicKey>>,
    peers: Arc<Vec<Author>>,
    proposer: Vec<Author>,
    smr_id: usize,
    smr: ChainedBftSMR<TestPayload, Author>,
    commit_cb_receiver: mpsc::UnboundedReceiver<LedgerInfoWithSignatures>,
    mempool: Arc<MockTransactionManager>,
    mempool_notif_receiver: mpsc::Receiver<usize>,
    storage: Arc<MockStorage<TestPayload>>,
}

impl SMRNode {
    fn start(
        quorum_size: usize,
        playground: &mut NetworkPlayground,
        signer: ValidatorSigner<Ed25519PrivateKey>,
        validator: Arc<ValidatorVerifier<Ed25519PublicKey>>,
        peers: Arc<Vec<Author>>,
        proposer: Vec<Author>,
        smr_id: usize,
        storage: Arc<MockStorage<TestPayload>>,
        initial_data: RecoveryData<TestPayload>,
    ) -> Self {
        let author = signer.author();

        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(8);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);

        playground.add_node(author, consensus_tx, network_reqs_rx);
        let runtime = runtime::Builder::new()
            .after_start(with_smr_id(signer.author().short_str()))
            .build()
            .expect("Failed to create Tokio runtime!");
        let network = ConsensusNetworkImpl::new(
            author,
            network_sender,
            network_events,
            Arc::clone(&peers),
            Arc::clone(&validator),
        );

        let config = ChainedBftSMRConfig {
            max_pruned_blocks_in_mem: 10000,
            pacemaker_initial_timeout: Duration::from_secs(3),
            contiguous_rounds: 2,
            max_block_size: 50,
        };
        let mut smr = ChainedBftSMR::new(
            author,
            quorum_size,
            signer.clone(),
            proposer.clone(),
            network,
            runtime,
            config,
            storage.clone(),
            initial_data,
        );
        let (commit_cb_sender, commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();
        let mut mp = MockTransactionManager::new();
        let commit_receiver = mp.take_commit_receiver();
        let mempool = Arc::new(mp);
        smr.start(
            mempool.clone(),
            Arc::new(MockStateComputer::new(commit_cb_sender.clone())),
        )
        .expect("Failed to start SMR!");
        Self {
            author,
            signer,
            validator,
            peers,
            proposer,
            smr_id,
            smr,
            commit_cb_receiver,
            mempool,
            mempool_notif_receiver: commit_receiver,
            storage,
        }
    }

    fn restart(mut self, quorum_size: usize, playground: &mut NetworkPlayground) -> Self {
        self.smr.stop();
        let recover_data = self
            .storage
            .get_recovery_data()
            .unwrap_or_else(|e| panic!("fail to restart due to: {}", e));
        Self::start(
            quorum_size,
            playground,
            self.signer,
            self.validator,
            self.peers,
            self.proposer,
            self.smr_id + 10,
            self.storage,
            recover_data,
        )
    }

    fn start_num_nodes(
        num_nodes: usize,
        quorum_size: usize,
        playground: &mut NetworkPlayground,
        proposer_type: ConsensusProposerType,
    ) -> Vec<Self> {
        let mut signers = vec![];
        let mut author_to_public_keys = HashMap::new();
        for smr_id in 0..num_nodes {
            // 0 -> [0000], 1 -> [1000] in the logs
            let random_validator_signer = ValidatorSigner::from_int(smr_id as u8);
            author_to_public_keys.insert(
                random_validator_signer.author(),
                random_validator_signer.public_key(),
            );
            signers.push(random_validator_signer);
        }
        let validator_verifier = Arc::new(
            ValidatorVerifier::new_with_quorum_size(author_to_public_keys, quorum_size)
                .expect("Invalid quorum_size."),
        );
        let peers: Arc<Vec<Author>> =
            Arc::new(signers.iter().map(|signer| signer.author()).collect());
        let proposer = {
            match proposer_type {
                FixedProposer => vec![peers[0]],
                RotatingProposer => validator_verifier.get_ordered_account_addresses(),
            }
        };
        let mut nodes = vec![];
        for smr_id in 0..num_nodes {
            let (storage, initial_data) = MockStorage::start_for_testing();
            nodes.push(Self::start(
                quorum_size,
                playground,
                signers.remove(0),
                Arc::clone(&validator_verifier),
                Arc::clone(&peers),
                proposer.clone(),
                smr_id,
                storage,
                initial_data,
            ));
        }
        nodes
    }
}

fn verify_finality_proof(node: &SMRNode, ledger_info_with_sig: &LedgerInfoWithSignatures) {
    let ledger_info_hash = ledger_info_with_sig.ledger_info().hash();
    for (author, signature) in ledger_info_with_sig.signatures() {
        assert_eq!(
            Ok(()),
            node.validator
                .verify_signature(*author, ledger_info_hash, &(signature.clone().into()))
        );
    }
}

#[test]
/// Should receive a new proposal upon start
fn basic_start_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let nodes = SMRNode::start_num_nodes(2, 2, &mut playground, RotatingProposer);
    let genesis = nodes[0]
        .smr
        .block_store()
        .expect("No valid block store!")
        .root();
    block_on(async move {
        let mut msg = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let first_proposal =
            ProposalInfo::<Vec<u64>, Author>::from_proto(msg[0].1.take_proposal()).unwrap();
        assert_eq!(first_proposal.proposal.height(), 1);
        assert_eq!(first_proposal.proposal.parent_id(), genesis.id());
        assert_eq!(
            first_proposal.proposal.quorum_cert().certified_block_id(),
            genesis.id()
        );
    });
}

#[test]
/// Upon startup, the first proposal is sent, delivered and voted by all the participants.
fn start_with_proposal_test() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let nodes = SMRNode::start_num_nodes(2, 2, &mut playground, RotatingProposer);

    block_on(async move {
        let _proposals = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        // Need to wait for 2 votes for the 2 replicas
        let votes: Vec<VoteMsg> = playground
            .wait_for_messages(2, NetworkPlayground::votes_only)
            .await
            .into_iter()
            .map(|(_, mut msg)| VoteMsg::from_proto(msg.take_vote()).unwrap())
            .collect();
        let proposed_block_id = votes[0].proposed_block_id();

        // Verify that the proposed block id is indeed present in the block store.
        assert!(nodes[0]
            .smr
            .block_store()
            .unwrap()
            .get_block(proposed_block_id)
            .is_some());
        assert!(nodes[1]
            .smr
            .block_store()
            .unwrap()
            .get_block(proposed_block_id)
            .is_some());
    });
}

#[test]
/// Upon startup, the first proposal is sent, voted by all the participants, QC is formed and
/// then the next proposal is sent.
fn basic_full_round() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let _nodes = SMRNode::start_num_nodes(2, 2, &mut playground, FixedProposer);

    block_on(async move {
        let _broadcast_proposals_1 = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let _votes_1 = playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        let mut broadcast_proposals_2 = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let next_proposal = ProposalInfo::<Vec<u64>, Author>::from_proto(
            broadcast_proposals_2[0].1.take_proposal(),
        )
        .unwrap();
        assert_eq!(next_proposal.proposal.round(), 2);
        assert_eq!(next_proposal.proposal.height(), 2);
    });
}

/// Verify the basic e2e flow: blocks are committed, txn manager is notified, block tree is
/// pruned, restart the node and we can still continue.
#[test]
fn basic_commit_and_restart() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut nodes = SMRNode::start_num_nodes(2, 2, &mut playground, RotatingProposer);
    let mut block_ids = vec![];

    block_on(async {
        let num_rounds = 10;

        for round in 0..num_rounds {
            let _proposals = playground
                .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg)
                .await;

            // A proposal is carrying a QC that commits a block of round - 3.
            if round >= 3 {
                let block_id_to_commit = block_ids[round - 3];
                let commit_v1 = nodes[0].commit_cb_receiver.next().await.unwrap();
                let commit_v2 = nodes[1].commit_cb_receiver.next().await.unwrap();
                assert_eq!(
                    commit_v1.ledger_info().consensus_block_id(),
                    block_id_to_commit
                );
                verify_finality_proof(&nodes[0], &commit_v1);
                assert_eq!(
                    commit_v2.ledger_info().consensus_block_id(),
                    block_id_to_commit
                );
                verify_finality_proof(&nodes[1], &commit_v2);
            }

            // v1 and v2 send votes
            let mut votes = playground
                .wait_for_messages(1, NetworkPlayground::votes_only)
                .await;
            let vote_msg = VoteMsg::from_proto(votes[0].1.take_vote()).unwrap();
            block_ids.push(vote_msg.proposed_block_id());
        }
        assert!(
            nodes[0].smr.block_store().unwrap().root().height() >= 6,
            "height of node 0 is {}",
            nodes[0].smr.block_store().unwrap().root().height()
        );
        assert!(
            nodes[1].smr.block_store().unwrap().root().height() >= 6,
            "height of node 1 is {}",
            nodes[1].smr.block_store().unwrap().root().height()
        );
        // This message is for proposal with round 11 to delivery the QC, but not gather the QC
        // so after restart, proposer will propose round 11 again.
        playground
            .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg)
            .await;
    });
    // create a new playground to avoid polling potential vote messages in previous one.
    playground = NetworkPlayground::new(runtime.executor());
    nodes = nodes
        .into_iter()
        .map(|node| node.restart(2, &mut playground))
        .collect();

    block_on(async {
        let mut round = 0;

        while round < 10 {
            // The loop is to ensure that we collect a network vote(enough for QC with 2 nodes) then
            // move the round forward because there's a race that node1 may or may not
            // reject round 11 depends on whether it voted for before restart.
            loop {
                let msg = playground
                    .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg)
                    .await;
                if msg[0].1.has_vote() {
                    round += 1;
                    break;
                }
            }
        }
        // Because of the race, we can't assert the commit reliably, instead we assert
        // both nodes commit to at least round 17.
        // We cannot reliable wait for the event of "commit & prune": the only thing that we know is
        // that after receiving the vote for round 20, the root should be at least height 16.
        assert!(
            nodes[0].smr.block_store().unwrap().root().height() >= 16,
            "height of node 0 is {}",
            nodes[0].smr.block_store().unwrap().root().height()
        );
        assert!(
            nodes[1].smr.block_store().unwrap().root().height() >= 16,
            "height of node 1 is {}",
            nodes[1].smr.block_store().unwrap().root().height()
        );
    });
}

#[test]
fn basic_block_retrieval() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(3, 2, &mut playground, FixedProposer);
    block_on(async move {
        let mut first_proposals = vec![];
        // First three proposals are delivered just to nodes[0[ and nodes[1].
        playground.drop_message_for(&nodes[0].author, nodes[2].author);
        for _ in 0..2 {
            playground
                .wait_for_messages(1, NetworkPlayground::proposals_only)
                .await;
            let mut votes = playground
                .wait_for_messages(1, NetworkPlayground::votes_only)
                .await;
            let vote_msg = VoteMsg::from_proto(votes[0].1.take_vote()).unwrap();
            let proposal_id = vote_msg.proposed_block_id();
            first_proposals.push(proposal_id);
        }
        // The next proposal is delivered to all: as a result nodes[2] should retrieve the missing
        // blocks from nodes[0] and vote for the 3th proposal.
        playground.stop_drop_message_for(&nodes[0].author, &nodes[2].author);

        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await;
        // Wait until nodes[2] sent out a vote, drop the vote from nodes[1] so that nodes[0]
        // won't move too far and prune the requested block
        playground.drop_message_for(&nodes[1].author, nodes[0].author);
        playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        playground.stop_drop_message_for(&nodes[1].author, &nodes[0].author);
        // the first two proposals should be present at nodes[2]
        for block_id in &first_proposals {
            assert!(nodes[2]
                .smr
                .block_store()
                .unwrap()
                .get_block(*block_id)
                .is_some());
        }

        // Both nodes[1] and nodes[2] are going to vote for 4th proposal and commit the 1th one.

        // Verify that nodes[2] commits the first proposal.
        playground
            .wait_for_messages(2, NetworkPlayground::votes_only)
            .await;
        if let Some(commit_v3) = nodes[2].commit_cb_receiver.next().await {
            assert_eq!(
                commit_v3.ledger_info().consensus_block_id(),
                first_proposals[0],
            );
        }
    });
}

#[test]
fn block_retrieval_with_timeout() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let nodes = SMRNode::start_num_nodes(3, 2, &mut playground, FixedProposer);
    block_on(async move {
        let mut first_proposals = vec![];
        // First three proposals are delivered just to nodes[0] and nodes[1].
        playground.drop_message_for(&nodes[0].author, nodes[2].author);
        for _ in 0..2 {
            playground
                .wait_for_messages(1, NetworkPlayground::proposals_only)
                .await;
            let mut votes = playground
                .wait_for_messages(1, NetworkPlayground::votes_only)
                .await;
            let vote_msg = VoteMsg::from_proto(votes[0].1.take_vote()).unwrap();
            let proposal_id = vote_msg.proposed_block_id();
            first_proposals.push(proposal_id);
        }
        // The next proposal is delivered to all: as a result nodes[2] should retrieve the missing
        // blocks from v1 and vote for the 4th proposal.
        playground.stop_drop_message_for(&nodes[0].author, &nodes[2].author);

        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await;
        playground.drop_message_for(&nodes[1].author, nodes[0].author);
        // Block RPC and wait until timeout for current round
        playground.drop_message_for(&nodes[2].author, nodes[0].author);
        playground
            .wait_for_messages(1, NetworkPlayground::new_round_only)
            .await;
        // Unblock RPC
        playground.stop_drop_message_for(&nodes[2].author, &nodes[0].author);
        // Wait until v3 sent out a vote, drop the vote from v2 so that v1 won't move too far
        // and prune the requested block
        playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        playground.stop_drop_message_for(&nodes[1].author, &nodes[0].author);
        // the first two proposals should be present at v3
        for block_id in &first_proposals {
            assert!(nodes[2]
                .smr
                .block_store()
                .unwrap()
                .get_block(*block_id)
                .is_some());
        }
    });
}

#[test]
/// Verify that a node that is lagging behind can catch up by state sync some blocks
/// have been pruned by the others.
fn basic_state_sync() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(3, 2, &mut playground, FixedProposer);
    block_on(async move {
        let mut proposals = vec![];
        // The first ten proposals are delivered just to nodes[0] and nodes[1], which should commit
        // the first seven blocks.
        playground.drop_message_for(&nodes[0].author, nodes[2].author);
        for _ in 0..10 {
            playground
                .wait_for_messages(1, NetworkPlayground::proposals_only)
                .await;
            let mut votes = playground
                .wait_for_messages(1, NetworkPlayground::votes_only)
                .await;
            let vote_msg = VoteMsg::from_proto(votes[0].1.take_vote()).unwrap();
            let proposal_id = vote_msg.proposed_block_id();
            proposals.push(proposal_id);
        }

        let mut node0_commits = vec![];
        for i in 0..7 {
            node0_commits.push(
                nodes[0]
                    .commit_cb_receiver
                    .next()
                    .await
                    .unwrap()
                    .ledger_info()
                    .consensus_block_id(),
            );
            assert_eq!(node0_commits[i], proposals[i]);
        }

        // Next proposal is delivered to all: as a result nodes[2] should be able to retrieve the
        // missing blocks from nodes[0] and commit the first eight proposals as well.
        playground.stop_drop_message_for(&nodes[0].author, &nodes[2].author);
        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await;
        let mut node2_commits = vec![];
        // The only notification we will receive is for the last (8th) proposal.
        node2_commits.push(
            nodes[2]
                .commit_cb_receiver
                .next()
                .await
                .unwrap()
                .ledger_info()
                .consensus_block_id(),
        );
        assert_eq!(node2_commits[0], proposals[7]);

        // wait for the vote from node2
        playground
            .wait_for_messages(1, NetworkPlayground::votes_only)
            .await;
        for (_, proposal) in playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await
        {
            assert_eq!(proposal.has_proposal(), true);
        }
        // Verify that node 2 has notified its mempool about the committed txn of next block.
        nodes[2]
            .mempool_notif_receiver
            .next()
            .await
            .expect("Fail to be notified by a mempool committed txns");
        assert_eq!(nodes[2].mempool.get_committed_txns().len(), 50);
    });
}

#[test]
/// Verify that a node syncs up when receiving a timeout message with a relevant ledger info
fn state_sync_on_timeout() {
    let runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.executor());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(3, 2, &mut playground, FixedProposer);
    block_on(async move {
        let mut proposals = vec![];
        // The first ten proposals are delivered just to nodes[0] and nodes[1], which should commit
        // the first seven blocks.
        playground.drop_message_for(&nodes[0].author, nodes[2].author);
        for _ in 0..10 {
            playground
                .wait_for_messages(1, NetworkPlayground::proposals_only)
                .await;
            let mut votes = playground
                .wait_for_messages(1, NetworkPlayground::votes_only)
                .await;
            let vote_msg = VoteMsg::from_proto(votes[0].1.take_vote()).unwrap();
            let proposal_id = vote_msg.proposed_block_id();
            proposals.push(proposal_id);
        }

        // Start dropping messages from 0 to 1 as well: node 0 is now disconnected and we can
        // expect timeouts from both 0 and 1.
        playground.drop_message_for(&nodes[0].author, nodes[1].author);

        // Wait for a timeout message from 2 to {0, 1} and from 1 to {0, 2}
        // (node 0 cannot send to anyone).  Note that there are 6 messages waited on
        // since 2 can timeout 2x while waiting for 1 to timeout.
        playground
            .wait_for_messages(6, NetworkPlayground::new_round_only)
            .await;

        let mut node2_commits = vec![];
        // The only notification we will receive is for the last commit known to nodes[1]: 7th
        // proposal.
        node2_commits.push(
            nodes[2]
                .commit_cb_receiver
                .next()
                .await
                .unwrap()
                .ledger_info()
                .consensus_block_id(),
        );
        assert_eq!(node2_commits[0], proposals[6]);
    });
}
