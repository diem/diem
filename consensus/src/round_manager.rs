// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::{BlockReader, BlockRetriever, BlockStore, VoteReceptionResult},
    counters,
    liveness::{
        proposal_generator::ProposalGenerator,
        proposer_election::ProposerElection,
        round_state::{NewRoundEvent, NewRoundReason, RoundState},
    },
    metrics_safety_rules::MetricsSafetyRules,
    network::{IncomingBlockRetrievalRequest, NetworkSender},
    network_interface::ConsensusMsg,
    persistent_liveness_storage::{PersistentLivenessStorage, RecoveryData},
    state_replication::{StateComputer, TxnManager},
    util::time_service::duration_since_epoch,
};
use anyhow::{ensure, Context, Result};
use consensus_types::{
    block::Block,
    block_retrieval::{BlockRetrievalResponse, BlockRetrievalStatus},
    common::{Author, Round},
    proposal_msg::ProposalMsg,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
    timeout_certificate::TimeoutCertificate,
    vote::Vote,
    vote_msg::VoteMsg,
    vote_proposal::VoteProposal,
};
use debug_interface::prelude::*;
use libra_crypto::hash::TransactionAccumulatorHasher;
use libra_logger::prelude::*;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::{
    epoch_state::EpochState, proof::AccumulatorExtensionProof,
    validator_verifier::ValidatorVerifier,
};
#[cfg(test)]
use safety_rules::ConsensusState;
use safety_rules::TSafetyRules;
use std::{sync::Arc, time::Duration};
use termion::color::*;

pub enum UnverifiedEvent {
    ProposalMsg(Box<ProposalMsg>),
    VoteMsg(Box<VoteMsg>),
    SyncInfo(Box<SyncInfo>),
}

impl UnverifiedEvent {
    pub fn verify(self, validator: &ValidatorVerifier) -> Result<VerifiedEvent> {
        Ok(match self {
            UnverifiedEvent::ProposalMsg(p) => {
                p.verify(validator)?;
                VerifiedEvent::ProposalMsg(p)
            }
            UnverifiedEvent::VoteMsg(v) => {
                v.verify(validator)?;
                VerifiedEvent::VoteMsg(v)
            }
            UnverifiedEvent::SyncInfo(s) => {
                s.verify(validator)?;
                VerifiedEvent::SyncInfo(s)
            }
        })
    }

    pub fn epoch(&self) -> u64 {
        match self {
            UnverifiedEvent::ProposalMsg(p) => p.epoch(),
            UnverifiedEvent::VoteMsg(v) => v.epoch(),
            UnverifiedEvent::SyncInfo(s) => s.epoch(),
        }
    }
}

impl From<ConsensusMsg> for UnverifiedEvent {
    fn from(value: ConsensusMsg) -> Self {
        match value {
            ConsensusMsg::ProposalMsg(m) => UnverifiedEvent::ProposalMsg(m),
            ConsensusMsg::VoteMsg(m) => UnverifiedEvent::VoteMsg(m),
            ConsensusMsg::SyncInfo(m) => UnverifiedEvent::SyncInfo(m),
            _ => unreachable!("Unexpected conversion"),
        }
    }
}

pub enum VerifiedEvent {
    ProposalMsg(Box<ProposalMsg>),
    VoteMsg(Box<VoteMsg>),
    SyncInfo(Box<SyncInfo>),
}

#[cfg(test)]
#[path = "round_manager_test.rs"]
mod round_manager_test;

#[cfg(any(test, feature = "fuzzing"))]
#[path = "round_manager_fuzzing.rs"]
pub mod round_manager_fuzzing;

/// If the node can't recover corresponding blocks from local storage, RecoveryManager is responsible
/// for processing the events carrying sync info and use the info to retrieve blocks from peers
pub struct RecoveryManager {
    epoch_state: EpochState,
    network: NetworkSender,
    storage: Arc<dyn PersistentLivenessStorage>,
    state_computer: Arc<dyn StateComputer>,
    last_committed_round: Round,
}

impl RecoveryManager {
    pub fn new(
        epoch_state: EpochState,
        network: NetworkSender,
        storage: Arc<dyn PersistentLivenessStorage>,
        state_computer: Arc<dyn StateComputer>,
        last_committed_round: Round,
    ) -> Self {
        RecoveryManager {
            epoch_state,
            network,
            storage,
            state_computer,
            last_committed_round,
        }
    }

    pub async fn process_proposal_msg(
        &mut self,
        proposal_msg: ProposalMsg,
    ) -> Result<RecoveryData> {
        let author = proposal_msg.proposer();
        let sync_info = proposal_msg.sync_info();
        self.sync_up(&sync_info, author).await
    }

    pub async fn process_vote_msg(&mut self, vote_msg: VoteMsg) -> Result<RecoveryData> {
        let author = vote_msg.vote().author();
        let sync_info = vote_msg.sync_info();
        self.sync_up(&sync_info, author).await
    }

    async fn sync_up(&mut self, sync_info: &SyncInfo, peer: Author) -> Result<RecoveryData> {
        sync_info.verify(&self.epoch_state.verifier)?;
        ensure!(
            sync_info.highest_round() > self.last_committed_round,
            "[RecoveryManager] Received sync info has lower round number than committed block"
        );
        ensure!(
            sync_info.epoch() == self.epoch_state.epoch,
            "[RecoveryManager] Received sync info is in different epoch than committed block"
        );
        let mut retriever = BlockRetriever::new(self.network.clone(), peer);
        let recovery_data = BlockStore::fast_forward_sync(
            &sync_info.highest_commit_cert(),
            &mut retriever,
            self.storage.clone(),
            self.state_computer.clone(),
        )
        .await?;

        Ok(recovery_data)
    }

    pub fn epoch_state(&self) -> &EpochState {
        &self.epoch_state
    }
}

/// Consensus SMR is working in an event based fashion: RoundManager is responsible for
/// processing the individual events (e.g., process_new_round, process_proposal, process_vote,
/// etc.). It is exposing the async processing functions for each event type.
/// The caller is responsible for running the event loops and driving the execution via some
/// executors.
pub struct RoundManager {
    epoch_state: EpochState,
    block_store: Arc<BlockStore>,
    round_state: RoundState,
    proposer_election: Box<dyn ProposerElection + Send + Sync>,
    proposal_generator: ProposalGenerator,
    safety_rules: MetricsSafetyRules,
    network: NetworkSender,
    txn_manager: Arc<dyn TxnManager>,
    storage: Arc<dyn PersistentLivenessStorage>,
}

impl RoundManager {
    pub fn new(
        epoch_state: EpochState,
        block_store: Arc<BlockStore>,
        round_state: RoundState,
        proposer_election: Box<dyn ProposerElection + Send + Sync>,
        proposal_generator: ProposalGenerator,
        safety_rules: MetricsSafetyRules,
        network: NetworkSender,
        txn_manager: Arc<dyn TxnManager>,
        storage: Arc<dyn PersistentLivenessStorage>,
    ) -> Self {
        Self {
            epoch_state,
            block_store,
            round_state,
            proposer_election,
            proposal_generator,
            safety_rules,
            txn_manager,
            network,
            storage,
        }
    }

    fn create_block_retriever(&self, author: Author) -> BlockRetriever {
        BlockRetriever::new(self.network.clone(), author)
    }

    /// Leader:
    ///
    /// This event is triggered by a new quorum certificate at the previous round or a
    /// timeout certificate at the previous round.  In either case, if this replica is the new
    /// proposer for this round, it is ready to propose and guarantee that it can create a proposal
    /// that all honest replicas can vote for.  While this method should only be invoked at most
    /// once per round, we ensure that only at most one proposal can get generated per round to
    /// avoid accidental equivocation of proposals.
    ///
    /// Replica:
    ///
    /// Do nothing
    async fn process_new_round_event(
        &mut self,
        new_round_event: NewRoundEvent,
    ) -> anyhow::Result<()> {
        debug!("Processing {}", new_round_event);
        counters::CURRENT_ROUND.set(new_round_event.round as i64);
        counters::ROUND_TIMEOUT_MS.set(new_round_event.timeout.as_millis() as i64);
        match new_round_event.reason {
            NewRoundReason::QCReady => {
                counters::QC_ROUNDS_COUNT.inc();
            }
            NewRoundReason::Timeout => {
                counters::TIMEOUT_ROUNDS_COUNT.inc();
            }
        };
        if self
            .proposer_election
            .is_valid_proposer(self.proposal_generator.author(), new_round_event.round)
        {
            let proposal_msg = self.generate_proposal(new_round_event).await?;
            let mut network = self.network.clone();
            network.broadcast_proposal(proposal_msg).await;
            counters::PROPOSALS_COUNT.inc();
        }
        Ok(())
    }

    async fn generate_proposal(
        &mut self,
        new_round_event: NewRoundEvent,
    ) -> anyhow::Result<ProposalMsg> {
        // Proposal generator will ensure that at most one proposal is generated per round
        let proposal = self
            .proposal_generator
            .generate_proposal(new_round_event.round)
            .await?;
        let signed_proposal = self.safety_rules.sign_proposal(proposal)?;
        self.txn_manager.trace_transactions(&signed_proposal);
        trace_edge!("parent_proposal", {"block", signed_proposal.parent_id()}, {"block", signed_proposal.id()});
        trace_event!("round_manager::generate_proposal", {"block", signed_proposal.id()});
        debug!("Propose {}", signed_proposal);
        // return proposal
        Ok(ProposalMsg::new(
            signed_proposal,
            self.block_store.sync_info(),
        ))
    }

    /// Process the proposal message:
    /// 1. ensure after processing sync info, we're at the same round as the proposal
    /// 2. execute and decide whether to vode for the proposal
    pub async fn process_proposal_msg(&mut self, proposal_msg: ProposalMsg) -> anyhow::Result<()> {
        trace_event!("round_manager::pre_process_proposal", {"block", proposal_msg.proposal().id()});
        self.ensure_round_and_sync_up(
            proposal_msg.proposal().round(),
            proposal_msg.sync_info(),
            proposal_msg.proposer(),
            true,
        )
        .await
        .context("[RoundManager] Process proposal")?;
        self.process_proposal(proposal_msg.take_proposal()).await
    }

    /// Sync to the sync info sending from peer if it has newer certificates, if we have newer certificates
    /// and help_remote is set, send it back the local sync info.
    async fn sync_up(
        &mut self,
        sync_info: &SyncInfo,
        author: Author,
        help_remote: bool,
    ) -> anyhow::Result<()> {
        let local_sync_info = self.block_store.sync_info();
        if help_remote && local_sync_info.has_newer_certificates(&sync_info) {
            counters::SYNC_INFO_MSGS_SENT_COUNT.inc();
            debug!(
                "Peer {} has stale state {}, send it back {}",
                author.short_str(),
                sync_info,
                local_sync_info,
            );
            self.network.send_sync_info(local_sync_info.clone(), author);
        }
        if sync_info.has_newer_certificates(&local_sync_info) {
            debug!(
                "Local state {} is stale than peer {} remote state {}",
                local_sync_info,
                author.short_str(),
                sync_info
            );
            // Some information in SyncInfo is ahead of what we have locally.
            // First verify the SyncInfo (didn't verify it in the yet).
            sync_info
                .verify(&self.epoch_state().verifier)
                .map_err(|e| {
                    security_log(SecurityEvent::InvalidSyncInfoMsg)
                        .error(&e)
                        .data(&sync_info)
                        .log();
                    e
                })?;
            self.block_store
                .add_certs(&sync_info, self.create_block_retriever(author))
                .await?;

            // Update safety rules and round_state and potentially start a new round.
            self.process_certificates().await?;
        }
        Ok(())
    }

    /// The function makes sure that it ensures the message_round equal to what we have locally,
    /// brings the missing dependencies from the QC and LedgerInfo of the given sync info and
    /// update the round_state with the certificates if succeed.
    /// Returns Error in case sync mgr failed to bring the missing dependencies.
    /// We'll try to help the remote if the SyncInfo lags behind and the flag is set.
    pub async fn ensure_round_and_sync_up(
        &mut self,
        message_round: Round,
        sync_info: &SyncInfo,
        author: Author,
        help_remote: bool,
    ) -> anyhow::Result<()> {
        ensure!(
            message_round >= self.round_state.current_round(),
            "round {} is stale than local {}",
            message_round,
            self.round_state.current_round()
        );
        self.sync_up(sync_info, author, help_remote).await?;
        ensure!(
            message_round == self.round_state.current_round(),
            "After sync, round {} doesn't match local {}",
            message_round,
            self.round_state.current_round()
        );
        Ok(())
    }

    /// Process the SyncInfo sent by peers to catch up to latest state.
    pub async fn process_sync_info_msg(
        &mut self,
        sync_info: SyncInfo,
        peer: Author,
    ) -> anyhow::Result<()> {
        debug!("Received a sync info msg: {}", sync_info);
        // To avoid a ping-pong cycle between two peers that move forward together.
        self.ensure_round_and_sync_up(sync_info.highest_round() + 1, &sync_info, peer, false)
            .await
            .context("[RoundManager] Failed to process sync info msg")
    }

    /// The replica broadcasts a "timeout vote message", which includes the round signature, which
    /// can be aggregated to a TimeoutCertificate.
    /// The timeout vote message can be one of the following three options:
    /// 1) In case a validator has previously voted in this round, it repeats the same vote and sign
    /// a timeout.
    /// 2) Otherwise vote for a NIL block and sign a timeout.
    pub async fn process_local_timeout(&mut self, round: Round) -> anyhow::Result<()> {
        ensure!(
            self.round_state.process_local_timeout(round),
            "[RoundManager] local timeout is stale"
        );

        let (use_last_vote, mut timeout_vote) = match self.round_state.vote_sent() {
            Some(vote) if vote.vote_data().proposed().round() == round => (true, vote),
            _ => {
                // Didn't vote in this round yet, generate a backup vote
                let nil_block = self.proposal_generator.generate_nil_block(round)?;
                debug!("Planning to vote for a NIL block {}", nil_block);
                counters::VOTE_NIL_COUNT.inc();
                let nil_vote = self.execute_and_vote(nil_block).await?;
                (false, nil_vote)
            }
        };

        warn!(
            "Round {} timed out: {}, expected round proposer was {:?}, broadcasting the vote to all replicas",
            round,
            if use_last_vote { "already executed and voted at this round" } else { "will try to generate a backup vote" },
            self.proposer_election.get_valid_proposer(round),
        );

        if !timeout_vote.is_timeout() {
            let timeout = timeout_vote.timeout();
            let signature = self
                .safety_rules
                .sign_timeout(&timeout)
                .context("[RoundManager] SafetyRules signs timeout")?;
            timeout_vote.add_timeout_signature(signature);
        }

        self.round_state.record_vote(timeout_vote.clone());
        let timeout_vote_msg = VoteMsg::new(timeout_vote, self.block_store.sync_info());
        self.network.broadcast_vote(timeout_vote_msg).await;
        Ok(())
    }

    /// This function is called only after all the dependencies of the given QC have been retrieved.
    async fn process_certificates(&mut self) -> anyhow::Result<()> {
        let sync_info = self.block_store.sync_info();
        if let Some(new_round_event) = self.round_state.process_certificates(sync_info) {
            self.process_new_round_event(new_round_event).await?;
        }
        Ok(())
    }

    /// This function processes a proposal for the current round:
    /// 1. Filter if it's proposed by valid proposer.
    /// 2. Execute and add it to a block store.
    /// 3. Try to vote for it following the safety rules.
    /// 4. In case a validator chooses to vote, send the vote to the representatives at the next
    /// round.
    async fn process_proposal(&mut self, proposal: Block) -> Result<()> {
        ensure!(
            self.proposer_election.is_valid_proposal(&proposal),
            "[RoundManager] Proposer {} for block {} is not a valid proposer for this round",
            proposal
                .author()
                .expect("Proposal should be verified having an author"),
            proposal,
        );

        let block_time_since_epoch = Duration::from_micros(proposal.timestamp_usecs());

        ensure!(
            block_time_since_epoch < self.round_state.current_round_deadline(),
            "[RoundManager] Waiting until proposal block timestamp usecs {:?} \
            would exceed the round duration {:?}, hence will not vote for this round",
            block_time_since_epoch,
            self.round_state.current_round_deadline(),
        );

        debug!("RoundManager: process_proposed_block {}", proposal);

        if let Some(time_to_receival) = duration_since_epoch().checked_sub(block_time_since_epoch) {
            counters::CREATION_TO_RECEIVAL_S.observe_duration(time_to_receival);
        }

        let proposal_round = proposal.round();

        let vote = self
            .execute_and_vote(proposal)
            .await
            .context("[RoundManager] Process proposal")?;

        let recipients = self
            .proposer_election
            .get_valid_proposer(proposal_round + 1);
        debug!("{}Voted: {} {}", Fg(Green), Fg(Reset), vote);

        self.round_state.record_vote(vote.clone());
        let vote_msg = VoteMsg::new(vote, self.block_store.sync_info());
        self.network.send_vote(vote_msg, vec![recipients]).await;
        Ok(())
    }

    /// The function generates a VoteMsg for a given proposed_block:
    /// * first execute the block and add it to the block store
    /// * then verify the voting rules
    /// * save the updated state to consensus DB
    /// * return a VoteMsg with the LedgerInfo to be committed in case the vote gathers QC.
    async fn execute_and_vote(&mut self, proposed_block: Block) -> anyhow::Result<Vote> {
        trace_code_block!("round_manager::execute_and_vote", {"block", proposed_block.id()});
        let executed_block = self
            .block_store
            .execute_and_insert_block(proposed_block)
            .context("[RoundManager] Failed to execute_and_insert the block")?;
        // notify mempool about failed txn
        let compute_result = executed_block.compute_result();
        if let Err(e) = self
            .txn_manager
            .notify(executed_block.block(), compute_result)
            .await
        {
            error!(
                "[RoundManager] Failed to notify mempool of rejected txns: {:?}",
                e
            );
        }
        let block = executed_block.block();

        // Short circuit if already voted.
        ensure!(
            self.round_state.vote_sent().is_none(),
            "[RoundManager] Already vote on this round {}",
            self.round_state.current_round()
        );

        let parent_block = self
            .block_store
            .get_block(executed_block.parent_id())
            .expect("[RoundManager] Parent block not found after execution");

        let vote_proposal = VoteProposal::new(
            AccumulatorExtensionProof::<TransactionAccumulatorHasher>::new(
                parent_block.compute_result().frozen_subtree_roots().clone(),
                parent_block.compute_result().num_leaves(),
                executed_block
                    .compute_result()
                    .transaction_info_hashes()
                    .clone(),
            ),
            block.clone(),
            executed_block.compute_result().epoch_state().clone(),
        );

        let vote = self
            .safety_rules
            .construct_and_sign_vote(&vote_proposal)
            .context(format!(
                "[RoundManager] SafetyRules {}Rejected{} {}",
                Fg(Red),
                Fg(Reset),
                block
            ))?;

        let consensus_state = self.safety_rules.consensus_state()?;
        counters::LAST_VOTE_ROUND.set(consensus_state.last_voted_round() as i64);
        counters::PREFERRED_BLOCK_ROUND.set(consensus_state.preferred_round() as i64);
        self.storage
            .save_vote(&vote)
            .context("[RoundManager] Fail to persist last vote")?;

        Ok(vote)
    }

    /// Upon new vote:
    /// 1. Ensures we're processing the vote from the same round as local round
    /// 2. Filter out votes for rounds that should not be processed by this validator (to avoid
    /// potential attacks).
    /// 2. Add the vote to the pending votes and check whether it finishes a QC.
    /// 3. Once the QC/TC successfully formed, notify the RoundState.
    pub async fn process_vote_msg(&mut self, vote_msg: VoteMsg) -> anyhow::Result<()> {
        trace_code_block!("round_manager::process_vote", {"block", vote_msg.proposed_block_id()});
        // Check whether this validator is a valid recipient of the vote.
        self.ensure_round_and_sync_up(
            vote_msg.vote().vote_data().proposed().round(),
            vote_msg.sync_info(),
            vote_msg.vote().author(),
            true,
        )
        .await
        .context("[RoundManager] Stop processing vote")?;
        self.process_vote(vote_msg.vote())
            .await
            .context("[RoundManager] Add a new vote")
    }

    /// Add a vote to the pending votes.
    /// If a new QC / TC is formed then
    /// 1) fetch missing dependencies if required, and then
    /// 2) call process_certificates(), which will start a new round in return.
    async fn process_vote(&mut self, vote: &Vote) -> anyhow::Result<()> {
        if !vote.is_timeout() {
            // Unlike timeout votes regular votes are sent to the leaders of the next round only.
            let next_round = vote.vote_data().proposed().round() + 1;
            ensure!(
                self.proposer_election
                    .is_valid_proposer(self.proposal_generator.author(), next_round),
                "[RoundManager] Received {}, but I am not a valid proposer for round {}, ignore.",
                vote,
                next_round
            );
        }
        debug!("Add vote: {}", vote);
        let block_id = vote.vote_data().proposed().id();
        // Check if the block already had a QC
        if self
            .block_store
            .get_quorum_cert_for_block(block_id)
            .is_some()
        {
            return Ok(());
        }
        // Add the vote and check whether it completes a new QC or a TC
        match self
            .round_state
            .insert_vote(vote, &self.epoch_state.verifier)
        {
            VoteReceptionResult::NewQuorumCertificate(qc) => {
                // Note that the block might not be present locally, in which case we cannot calculate
                // time between block creation and qc
                if let Some(time_to_qc) = self.block_store.get_block(block_id).and_then(|block| {
                    duration_since_epoch()
                        .checked_sub(Duration::from_micros(block.timestamp_usecs()))
                }) {
                    counters::CREATION_TO_QC_S.observe_duration(time_to_qc);
                }

                self.new_qc_aggregated(qc, vote.author()).await
            }
            VoteReceptionResult::NewTimeoutCertificate(tc) => self.new_tc_aggregated(tc).await,
            _ => Ok(()),
        }
    }

    async fn new_qc_aggregated(
        &mut self,
        qc: Arc<QuorumCert>,
        preferred_peer: Author,
    ) -> anyhow::Result<()> {
        self.block_store
            .insert_quorum_cert(&qc, &mut self.create_block_retriever(preferred_peer))
            .await
            .context("[RoundManager] Failed to process a newly aggregated QC")?;
        self.process_certificates().await
    }

    async fn new_tc_aggregated(&mut self, tc: Arc<TimeoutCertificate>) -> anyhow::Result<()> {
        self.block_store
            .insert_timeout_certificate(tc.clone())
            .context("[RoundManager] Failed to process a newly aggregated TC")?;

        // Process local highest qc should be no-op
        self.process_certificates().await
    }

    /// Retrieve a n chained blocks from the block store starting from
    /// an initial parent id, returning with <n (as many as possible) if
    /// id or its ancestors can not be found.
    ///
    /// The current version of the function is not really async, but keeping it this way for
    /// future possible changes.
    pub async fn process_block_retrieval(
        &self,
        request: IncomingBlockRetrievalRequest,
    ) -> anyhow::Result<()> {
        let mut blocks = vec![];
        let mut status = BlockRetrievalStatus::Succeeded;
        let mut id = request.req.block_id();
        while (blocks.len() as u64) < request.req.num_blocks() {
            if let Some(executed_block) = self.block_store.get_block(id) {
                id = executed_block.parent_id();
                blocks.push(executed_block.block().clone());
            } else {
                status = BlockRetrievalStatus::NotEnoughBlocks;
                break;
            }
        }

        if blocks.is_empty() {
            status = BlockRetrievalStatus::IdNotFound;
        }

        let response = Box::new(BlockRetrievalResponse::new(status, blocks));
        lcs::to_bytes(&ConsensusMsg::BlockRetrievalResponse(response))
            .and_then(|bytes| {
                request
                    .response_sender
                    .send(Ok(bytes.into()))
                    .map_err(|e| lcs::Error::Custom(format!("{:?}", e)))
            })
            .context("[RoundManager] Failed to process block retrieval")
    }

    /// To jump start new round with the current certificates we have.
    pub async fn start(&mut self, last_vote_sent: Option<Vote>) {
        let new_round_event = self
            .round_state
            .process_certificates(self.block_store.sync_info())
            .expect("Can not jump start a round_state from existing certificates.");
        if let Some(vote) = last_vote_sent {
            self.round_state.record_vote(vote);
        }
        if let Err(e) = self.process_new_round_event(new_round_event).await {
            error!("[RoundManager] Error during start: {:?}", e);
        }
    }

    /// Inspect the current consensus state.
    #[cfg(test)]
    pub fn consensus_state(&mut self) -> ConsensusState {
        self.safety_rules.consensus_state().unwrap()
    }

    pub fn epoch_state(&self) -> &EpochState {
        &self.epoch_state
    }

    pub fn round_state(&self) -> &RoundState {
        &self.round_state
    }
}
