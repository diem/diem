// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{
            BlockReader, BlockRetriever, BlockStore, PendingVotes, VoteReceptionResult,
        },
        liveness::{
            pacemaker::{NewRoundEvent, NewRoundReason, Pacemaker},
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
        },
        network::NetworkSender,
        persistent_liveness_storage::PersistentLivenessStorage,
    },
    counters,
    state_replication::TxnManager,
    util::time_service::{
        duration_since_epoch, wait_if_possible, TimeService, WaitingError, WaitingSuccess,
    },
};
use anyhow::{ensure, format_err, Context};
use consensus_types::{
    accumulator_extension_proof::AccumulatorExtensionProof,
    block::Block,
    common::{Author, Payload, Round},
    proposal_msg::ProposalMsg,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
    timeout_certificate::TimeoutCertificate,
    vote::Vote,
    vote_msg::VoteMsg,
    vote_proposal::VoteProposal,
};
use libra_crypto::hash::TransactionAccumulatorHasher;
use libra_logger::prelude::*;
use libra_prost_ext::MessageExt;
use libra_types::crypto_proxies::{
    LedgerInfoWithSignatures, ValidatorChangeProof, ValidatorVerifier,
};
use mirai_annotations::{
    debug_checked_precondition, debug_checked_precondition_eq, debug_checked_verify,
    debug_checked_verify_eq,
};
use network::proto::{ConsensusMsg, ConsensusMsg_oneof};

use crate::chained_bft::network::IncomingBlockRetrievalRequest;
use consensus_types::block_retrieval::{BlockRetrievalResponse, BlockRetrievalStatus};
#[cfg(test)]
use safety_rules::ConsensusState;
use safety_rules::TSafetyRules;
use std::convert::TryInto;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use termion::color::*;

#[cfg(test)]
#[path = "event_processor_test.rs"]
mod event_processor_test;

#[cfg(any(feature = "fuzzing", test))]
#[path = "event_processor_fuzzing.rs"]
pub mod event_processor_fuzzing;

/// Consensus SMR is working in an event based fashion: EventProcessor is responsible for
/// processing the individual events (e.g., process_new_round, process_proposal, process_vote,
/// etc.). It is exposing the async processing functions for each event type.
/// The caller is responsible for running the event loops and driving the execution via some
/// executors.
pub struct EventProcessor<T> {
    block_store: Arc<BlockStore<T>>,
    pending_votes: PendingVotes,
    pacemaker: Pacemaker,
    proposer_election: Box<dyn ProposerElection<T> + Send + Sync>,
    proposal_generator: ProposalGenerator<T>,
    safety_rules: Box<dyn TSafetyRules<T> + Send + Sync>,
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    network: NetworkSender,
    storage: Arc<dyn PersistentLivenessStorage<T>>,
    time_service: Arc<dyn TimeService>,
    // Cache of the last sent vote message.
    last_vote_sent: Option<(Vote, Round)>,
    validators: Arc<ValidatorVerifier>,
}

impl<T: Payload> EventProcessor<T> {
    pub fn new(
        block_store: Arc<BlockStore<T>>,
        last_vote: Option<Vote>,
        pacemaker: Pacemaker,
        proposer_election: Box<dyn ProposerElection<T> + Send + Sync>,
        proposal_generator: ProposalGenerator<T>,
        safety_rules: Box<dyn TSafetyRules<T> + Send + Sync>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
        network: NetworkSender,
        storage: Arc<dyn PersistentLivenessStorage<T>>,
        time_service: Arc<dyn TimeService>,
        validators: Arc<ValidatorVerifier>,
    ) -> Self {
        counters::BLOCK_RETRIEVAL_COUNT.get();
        counters::STATE_SYNC_COUNT.get();
        let last_vote_sent = last_vote.map(|v| {
            let round = v.vote_data().proposed().round();
            (v, round)
        });
        let pending_votes = PendingVotes::new();

        Self {
            block_store,
            pending_votes,
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules,
            txn_manager,
            network,
            storage,
            time_service,
            last_vote_sent,
            validators,
        }
    }

    fn create_block_retriever(&self, deadline: Instant, author: Author) -> BlockRetriever {
        BlockRetriever::new(self.network.clone(), deadline, author)
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
    async fn process_new_round_event(&mut self, new_round_event: NewRoundEvent) {
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
            .is_none()
        {
            return;
        }
        let proposal_msg = match self.generate_proposal(new_round_event).await {
            Ok(x) => x,
            Err(e) => {
                error!("Error while generating proposal: {:?}", e);
                return;
            }
        };
        let mut network = self.network.clone();
        network.broadcast_proposal(proposal_msg).await;
        counters::PROPOSALS_COUNT.inc();
    }

    async fn generate_proposal(
        &mut self,
        new_round_event: NewRoundEvent,
    ) -> anyhow::Result<ProposalMsg<T>> {
        // Proposal generator will ensure that at most one proposal is generated per round
        let proposal = self
            .proposal_generator
            .generate_proposal(
                new_round_event.round,
                self.pacemaker.current_round_deadline(),
            )
            .await?;
        let signed_proposal = self.safety_rules.sign_proposal(proposal)?;
        debug!("Propose {}", signed_proposal);
        // return proposal
        Ok(ProposalMsg::new(signed_proposal, self.gen_sync_info()))
    }

    /// Process a ProposalMsg, pre_process would bring all the dependencies and filter out invalid
    /// proposal, process_proposed_block would execute and decide whether to vote for it.
    pub async fn process_proposal_msg(&mut self, proposal_msg: ProposalMsg<T>) {
        if let Some(block) = self.pre_process_proposal(proposal_msg).await {
            self.process_proposed_block(block).await
        }
    }

    /// The function is responsible for processing the incoming proposals and the Quorum
    /// Certificate.
    /// 1. sync up to the SyncInfo including committing to the committed state the HCC carries
    /// and fetch all the blocks from the committed state to the HQC
    /// 2. forwarding the proposals to the ProposerElection queue,
    /// which is going to eventually trigger one winning proposal per round
    async fn pre_process_proposal(&mut self, proposal_msg: ProposalMsg<T>) -> Option<Block<T>> {
        // Pacemaker is going to be updated with all the proposal certificates later,
        // but it's known that the pacemaker's round is not going to decrease so we can already
        // filter out the proposals from old rounds.
        let current_round = self.pacemaker.current_round();
        if proposal_msg.round() < current_round {
            return None;
        }
        if self
            .proposer_election
            .is_valid_proposer(proposal_msg.proposer(), proposal_msg.round())
            .is_none()
        {
            warn!(
                "Proposer {} for block {} is not a valid proposer for this round",
                proposal_msg.proposer(),
                proposal_msg.proposal()
            );
            return None;
        }
        if let Err(e) = self
            .sync_up(proposal_msg.sync_info(), proposal_msg.proposer(), true)
            .await
        {
            warn!(
                "Dependencies of proposal {} could not be added to the block store: {}",
                proposal_msg, e
            );
            return None;
        }

        // pacemaker may catch up with the SyncInfo, check again
        let current_round = self.pacemaker.current_round();
        if proposal_msg.round() != current_round {
            return None;
        }

        self.proposer_election
            .process_proposal(proposal_msg.take_proposal())
    }

    /// In case some peer's round or HQC is stale, send a SyncInfo message to that peer.
    fn help_remote_if_stale(&self, peer: Author, remote_round: Round, remote_hqc_round: Round) {
        if self.proposal_generator.author() == peer {
            return;
        }
        // pacemaker's round is sync_info.highest_round() + 1
        if remote_round + 1 < self.pacemaker.current_round()
            || remote_hqc_round
                < self
                    .block_store
                    .highest_quorum_cert()
                    .certified_block()
                    .round()
        {
            let sync_info = self.gen_sync_info();

            debug!(
                "Peer {} is at round {} with hqc round {}, sending it {}",
                peer.short_str(),
                remote_round,
                remote_hqc_round,
                sync_info,
            );
            counters::SYNC_INFO_MSGS_SENT_COUNT.inc();
            self.network.send_sync_info(sync_info, peer);
        }
    }

    /// The function makes sure that it brings the missing dependencies from the QC and LedgerInfo
    /// of the given sync info and update the pacemaker with the certificates if succeed.
    /// Returns Error in case sync mgr failed to bring the missing dependencies.
    /// We'll try to help the remote if the SyncInfo lags behind and the flag is set.
    async fn sync_up(
        &mut self,
        sync_info: &SyncInfo,
        author: Author,
        help_remote: bool,
    ) -> anyhow::Result<()> {
        if help_remote {
            self.help_remote_if_stale(author, sync_info.highest_round(), sync_info.hqc_round());
        }

        let current_hqc_round = self
            .block_store
            .highest_quorum_cert()
            .certified_block()
            .round();

        let current_htc_round = self
            .block_store
            .highest_timeout_cert()
            .map_or(0, |tc| tc.round());

        if current_hqc_round >= sync_info.hqc_round()
            && current_htc_round >= sync_info.htc_round()
            && self.block_store.root().round()
                >= sync_info.highest_commit_cert().commit_info().round()
        {
            return Ok(());
        }

        // Some information in SyncInfo is ahead of what we have locally.
        // First verify the SyncInfo (didn't verify it in the yet).
        sync_info.verify(self.validators.as_ref()).map_err(|e| {
            security_log(SecurityEvent::InvalidSyncInfoMsg)
                .error(&e)
                .data(&sync_info)
                .log();
            e
        })?;

        let deadline = self.pacemaker.current_round_deadline();
        let now = Instant::now();
        let deadline_repr = if deadline.gt(&now) {
            deadline
                .checked_duration_since(now)
                .map_or("0 ms".to_string(), |v| format!("{:?}", v))
        } else {
            now.checked_duration_since(deadline)
                .map_or("0 ms".to_string(), |v| format!("Already late by {:?}", v))
        };
        debug!(
            "Starting sync: current_hqc_round = {}, sync_info_hqc_round = {}, deadline = {:?}",
            current_hqc_round,
            sync_info.hqc_round(),
            deadline_repr,
        );
        self.block_store
            .sync_to(&sync_info, self.create_block_retriever(deadline, author))
            .await
            .map_err(|e| {
                warn!(
                    "Fail to sync up to HQC @ round {}: {}",
                    sync_info.hqc_round(),
                    e
                );
                e
            })?;
        debug!("Caught up to HQC at round {}", sync_info.hqc_round());

        // Update the block store and potentially start a new round.
        self.process_certificates(
            sync_info.highest_quorum_cert(),
            sync_info.highest_timeout_certificate(),
        )
        .await
    }

    /// Process the SyncInfo sent by peers to catch up to latest state.
    pub async fn process_sync_info_msg(&mut self, sync_info: SyncInfo, peer: Author) {
        debug!("Received a sync info msg: {}", sync_info);
        counters::SYNC_INFO_MSGS_RECEIVED_COUNT.inc();
        // To avoid a ping-pong cycle between two peers that move forward together.
        if let Err(e) = self.sync_up(&sync_info, peer, false).await {
            error!("Fail to process sync info: {}", e);
        }
    }

    /// The replica broadcasts a "timeout vote message", which includes the round signature, which
    /// can be aggregated to a TimeoutCertificate.
    /// The timeout vote message can be one of the following three options:
    /// 1) In case a validator has previously voted in this round, it repeats the same vote.
    /// 2) In case a validator didn't vote yet but has a secondary proposal, it executes this
    /// proposal and votes.
    /// 3) If neither primary nor secondary proposals are available, vote for a NIL block.
    pub async fn process_local_timeout(&mut self, round: Round) {
        if !self.pacemaker.process_local_timeout(round) {
            // The timeout event is late: the node has already moved to another round.
            return;
        }
        let last_voted_round = self
            .safety_rules
            .consensus_state()
            .unwrap()
            .last_voted_round();
        warn!(
            "Round {} timed out: {}, expected round proposer was {:?}, broadcasting the vote to all replicas",
            round,
            if last_voted_round == round { "already executed and voted at this round" } else { "will try to generate a backup vote" },
            self.proposer_election.get_valid_proposers(round).iter().map(|p| p.short_str()).collect::<Vec<String>>(),
        );

        let mut timeout_vote = match self.last_vote_sent.as_ref() {
            Some((vote, vote_round)) if (*vote_round == round) => vote.clone(),
            _ => {
                // Didn't vote in this round yet, generate a backup vote
                let backup_vote_res = self.gen_backup_vote(round).await;
                match backup_vote_res {
                    Ok(backup_vote) => backup_vote,
                    Err(e) => {
                        error!("Failed to generate a backup vote: {:?}", e);
                        return;
                    }
                }
            }
        };

        if !timeout_vote.is_timeout() {
            let timeout = timeout_vote.timeout();
            let response = self.safety_rules.sign_timeout(&timeout);
            match response {
                Ok(signature) => timeout_vote.add_timeout_signature(signature),
                Err(e) => {
                    error!("{}Rejected{} {}: {:?}", Fg(Red), Fg(Reset), timeout, e);
                    return;
                }
            }
        }

        let timeout_vote_msg = VoteMsg::new(timeout_vote, self.gen_sync_info());
        self.network.broadcast_vote(timeout_vote_msg).await
    }

    async fn gen_backup_vote(&mut self, round: Round) -> anyhow::Result<Vote> {
        // We generally assume that this function is called only if no votes have been sent in this
        // round, but having a duplicate proposal here would work ok because block store makes
        // sure the calls to `execute_and_insert_block` are idempotent.

        // Either use the best proposal received in this round or a NIL block if nothing available.
        let block = match self.proposer_election.take_backup_proposal(round) {
            Some(b) => {
                debug!("Planning to vote for a backup proposal {}", b);
                counters::VOTE_SECONDARY_PROPOSAL_COUNT.inc();
                b
            }
            None => {
                let nil_block = self.proposal_generator.generate_nil_block(round)?;
                debug!("Planning to vote for a NIL block {}", nil_block);
                counters::VOTE_NIL_COUNT.inc();
                nil_block
            }
        };
        self.execute_and_vote(block).await
    }

    /// This function is called only after all the dependencies of the given QC have been retrieved.
    async fn process_certificates(
        &mut self,
        qc: &QuorumCert,
        tc: Option<&TimeoutCertificate>,
    ) -> anyhow::Result<()> {
        self.safety_rules.update(qc)?;
        let consensus_state = self.safety_rules.consensus_state()?;
        counters::PREFERRED_BLOCK_ROUND.set(consensus_state.preferred_round() as i64);

        let mut highest_committed_proposal_round = None;
        if let Some(block) = self.block_store.get_block(qc.commit_info().id()) {
            if block.round() > self.block_store.root().round() {
                // We don't want to use NIL commits for pacemaker round interval calculations.
                if !block.is_nil_block() {
                    highest_committed_proposal_round = Some(block.round());
                }
                let finality_proof = qc.ledger_info().clone();
                self.process_commit(finality_proof).await;
            }
        }
        let mut tc_round = None;
        if let Some(timeout_cert) = tc {
            tc_round = Some(timeout_cert.round());
            self.block_store
                .insert_timeout_certificate(Arc::new(timeout_cert.clone()))
                .context("Failed to process TC")?;
        }
        if let Some(new_round_event) = self.pacemaker.process_certificates(
            Some(qc.certified_block().round()),
            tc_round,
            highest_committed_proposal_round,
        ) {
            self.process_new_round_event(new_round_event).await;
        }
        Ok(())
    }

    /// This function processes a proposal that was chosen as a representative of its round:
    /// 1. Add it to a block store.
    /// 2. Try to vote for it following the safety rules.
    /// 3. In case a validator chooses to vote, send the vote to the representatives at the next
    /// position.
    async fn process_proposed_block(&mut self, proposal: Block<T>) {
        debug!("EventProcessor: process_proposed_block {}", proposal);
        // Safety invariant: For any valid proposed block, its parent block == the block pointed to
        // by its QC.
        debug_checked_precondition_eq!(
            proposal.parent_id(),
            proposal.quorum_cert().certified_block().id()
        );
        // Safety invariant: QC of the parent block is present in the block store
        // (Ensured by the call to pre-process proposal before this function is called).
        debug_checked_precondition!(self
            .block_store
            .get_quorum_cert_for_block(proposal.parent_id())
            .is_some());

        if let Some(time_to_receival) =
            duration_since_epoch().checked_sub(Duration::from_micros(proposal.timestamp_usecs()))
        {
            counters::CREATION_TO_RECEIVAL_S.observe_duration(time_to_receival);
        }

        let proposal_round = proposal.round();
        // Creating these variables here since proposal gets moved in the call to execute_and_vote.
        // Used in MIRAI annotation later.
        let proposal_id = proposal.id();
        let proposal_parent_id = proposal.parent_id();
        let certified_parent_block_round = proposal.quorum_cert().parent_block().round();

        let vote = match self.execute_and_vote(proposal).await {
            Err(e) => {
                warn!("{:?}", e);
                return;
            }
            Ok(vote) => vote,
        };

        // Safety invariant: The vote being sent is for the proposal that was received.
        debug_checked_verify_eq!(proposal_id, vote.vote_data().proposed().id());
        // Safety invariant: The last voted round is updated to be the same as the proposed block's
        // round. At this point, the replica has decided to vote for the proposed block.
        debug_checked_verify_eq!(
            self.safety_rules
                .consensus_state()
                .unwrap()
                .last_voted_round(),
            proposal_round
        );
        // Safety invariant: qc_parent <-- qc
        // the preferred block round must be at least as large as qc_parent's round.
        debug_checked_verify!(
            self.safety_rules
                .consensus_state()
                .unwrap()
                .preferred_round()
                >= certified_parent_block_round
        );

        let recipients = self
            .proposer_election
            .get_valid_proposers(proposal_round + 1);
        debug!("{}Voted: {} {}", Fg(Green), Fg(Reset), vote);

        // Safety invariant: The parent block must be present in the block store and the replica
        // only votes for blocks with round greater than the parent block's round.
        debug_checked_verify!(self
            .block_store
            .get_block(proposal_parent_id)
            .map_or(false, |parent_block| parent_block.round() < proposal_round));
        let vote_msg = VoteMsg::new(vote, self.gen_sync_info());
        self.network.send_vote(vote_msg, recipients).await;
    }

    async fn wait_before_vote_if_needed(
        &self,
        block_timestamp_us: u64,
    ) -> Result<(), WaitingError> {
        let current_round_deadline = self.pacemaker.current_round_deadline();
        match wait_if_possible(
            self.time_service.as_ref(),
            Duration::from_micros(block_timestamp_us),
            current_round_deadline,
        )
        .await
        {
            Ok(waiting_success) => {
                debug!("Success with {:?} for being able to vote", waiting_success);

                match waiting_success {
                    WaitingSuccess::WaitWasRequired { wait_duration, .. } => {
                        counters::VOTE_SUCCESS_WAIT_S.observe_duration(wait_duration);
                        counters::VOTES_COUNT
                            .with_label_values(&["wait_was_required"])
                            .inc();
                    }
                    WaitingSuccess::NoWaitRequired { .. } => {
                        counters::VOTE_SUCCESS_WAIT_S.observe_duration(Duration::new(0, 0));
                        counters::VOTES_COUNT
                            .with_label_values(&["no_wait_required"])
                            .inc();
                    }
                }
            }
            Err(waiting_error) => {
                match waiting_error {
                    WaitingError::MaxWaitExceeded => {
                        error!(
                                "Waiting until proposal block timestamp usecs {:?} would exceed the round duration {:?}, hence will not vote for this round",
                                block_timestamp_us,
                                current_round_deadline);
                        counters::VOTE_FAILURE_WAIT_S.observe_duration(Duration::new(0, 0));
                        counters::VOTES_COUNT
                            .with_label_values(&["max_wait_exceeded"])
                            .inc();
                    }
                    WaitingError::WaitFailed {
                        current_duration_since_epoch,
                        wait_duration,
                    } => {
                        error!(
                                "Even after waiting for {:?}, proposal block timestamp usecs {:?} >= current timestamp usecs {:?}, will not vote for this round",
                                wait_duration,
                                block_timestamp_us,
                                current_duration_since_epoch);
                        counters::VOTE_FAILURE_WAIT_S.observe_duration(wait_duration);
                        counters::VOTES_COUNT
                            .with_label_values(&["wait_failed"])
                            .inc();
                    }
                };
                return Err(waiting_error);
            }
        }
        Ok(())
    }

    /// Generate sync info that can be attached to an outgoing message
    fn gen_sync_info(&self) -> SyncInfo {
        let hqc = self.block_store.highest_quorum_cert().as_ref().clone();
        // No need to include HTC if it's lower than HQC
        let htc = self
            .block_store
            .highest_timeout_cert()
            .filter(|tc| tc.round() > hqc.certified_block().round())
            .map(|tc| tc.as_ref().clone());
        SyncInfo::new(
            hqc,
            self.block_store.highest_commit_cert().as_ref().clone(),
            htc,
        )
    }

    /// The function generates a VoteMsg for a given proposed_block:
    /// * first execute the block and add it to the block store
    /// * then verify the voting rules
    /// * save the updated state to consensus DB
    /// * return a VoteMsg with the LedgerInfo to be committed in case the vote gathers QC.
    ///
    /// This function assumes that it might be called from different tasks concurrently.
    async fn execute_and_vote(&mut self, proposed_block: Block<T>) -> anyhow::Result<Vote> {
        let executed_block = self
            .block_store
            .execute_and_insert_block(proposed_block)
            .context("Failed to execute_and_insert the block")?;
        let block = executed_block.block();

        // Checking pacemaker round again, because multiple proposed_block can now race
        // during async block retrieval
        ensure!(
            block.round() == self.pacemaker.current_round(),
            "Proposal {} rejected because round is incorrect. Pacemaker: {}, proposed_block: {}",
            block,
            self.pacemaker.current_round(),
            block.round(),
        );

        let parent_block = self
            .block_store
            .get_block(executed_block.parent_id())
            .ok_or_else(|| format_err!("Parent block not found in block store"))?;

        self.wait_before_vote_if_needed(block.timestamp_usecs())
            .await?;

        let vote_proposal = VoteProposal::new(
            AccumulatorExtensionProof::<TransactionAccumulatorHasher>::new(
                parent_block
                    .executed_trees()
                    .txn_accumulator()
                    .frozen_subtree_roots()
                    .clone(),
                parent_block.executed_trees().txn_accumulator().num_leaves(),
                executed_block.transaction_info_hashes(),
            ),
            block.clone(),
            executed_block.compute_result().executed_state.validators,
        );

        let vote = self
            .safety_rules
            .construct_and_sign_vote(&vote_proposal)
            .with_context(|| format!("{}Rejected{} {}", Fg(Red), Fg(Reset), block))?;

        let consensus_state = self.safety_rules.consensus_state()?;
        counters::LAST_VOTE_ROUND.set(consensus_state.last_voted_round() as i64);

        self.storage
            .save_state(&vote)
            .context("Fail to persist consensus state")?;
        self.last_vote_sent.replace((vote.clone(), block.round()));
        Ok(vote)
    }

    /// Upon new vote:
    /// 1. Filter out votes for rounds that should not be processed by this validator (to avoid
    /// potential attacks).
    /// 2. Add the vote to the store and check whether it finishes a QC.
    /// 3. Once the QC successfully formed, notify the Pacemaker.
    pub async fn process_vote(&mut self, vote_msg: VoteMsg) {
        // Check whether this validator is a valid recipient of the vote.
        if !vote_msg.vote().is_timeout() {
            // Unlike timeout votes regular votes are sent to the leaders of the next round only.
            let next_round = vote_msg.vote().vote_data().proposed().round() + 1;
            if self
                .proposer_election
                .is_valid_proposer(self.proposal_generator.author(), next_round)
                .is_none()
            {
                debug!(
                    "Received {}, but I am not a valid proposer for round {}, ignore.",
                    vote_msg, next_round
                );
                security_log(SecurityEvent::InvalidConsensusVote)
                    .error("InvalidProposer")
                    .data(vote_msg)
                    .data(next_round)
                    .log();
                return;
            }
        } else {
            // Sync up for timeout votes only.
            if self
                .sync_up(vote_msg.sync_info(), vote_msg.vote().author(), true)
                .await
                .is_err()
            {
                warn!("Stop vote processing because of sync up error.");
                return;
            };
        }
        if let Err(e) = self.add_vote(vote_msg.vote()).await {
            error!("Error adding a new vote: {:?}", e);
        }
    }

    /// Add a vote to the pending votes.
    /// If a new QC / TC is formed then
    /// 1) fetch missing dependencies if required, and then
    /// 2) call process_certificates(), which will start a new round in return.
    async fn add_vote(&mut self, vote: &Vote) -> anyhow::Result<()> {
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
        let res = self.pending_votes.insert_vote(vote, &self.validators);
        match res {
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
        let deadline = self.pacemaker.current_round_deadline();
        // Process local highest commit cert should be no-op, this will sync us to the QC
        self.block_store
            .sync_to(
                &SyncInfo::new(
                    qc.as_ref().clone(),
                    self.block_store.highest_commit_cert().as_ref().clone(),
                    None,
                ),
                self.create_block_retriever(deadline, preferred_peer),
            )
            .await
            .context("Failed to process a newly aggregated QC")?;
        self.process_certificates(qc.as_ref(), None).await
    }

    async fn new_tc_aggregated(&mut self, tc: Arc<TimeoutCertificate>) -> anyhow::Result<()> {
        self.block_store
            .insert_timeout_certificate(tc.clone())
            .context("Failed to process a newly aggregated TC")?;

        // Process local highest qc should be no-op
        self.process_certificates(
            self.block_store.highest_quorum_cert().as_ref(),
            Some(tc.as_ref()),
        )
        .await
    }

    /// Upon (potentially) new commit:
    /// 1. Commit the blocks via block store.
    /// 2. After the state is finalized, update the txn manager with the status of the committed
    /// transactions.
    async fn process_commit(&mut self, finality_proof: LedgerInfoWithSignatures) {
        let blocks_to_commit = match self.block_store.commit(finality_proof.clone()).await {
            Ok(blocks) => blocks,
            Err(e) => {
                error!("{:?}", e);
                return;
            }
        };
        // At this moment the new state is persisted and we can notify the clients.
        // Multiple blocks might be committed at once: notify about all the transactions in the
        // path from the old root to the new root.
        for committed in blocks_to_commit {
            if let Some(time_to_commit) = duration_since_epoch()
                .checked_sub(Duration::from_micros(committed.timestamp_usecs()))
            {
                counters::CREATION_TO_COMMIT_S.observe_duration(time_to_commit);
            }
            if let Some(payload) = committed.payload() {
                let compute_result = committed.compute_result();
                if let Err(e) = self
                    .txn_manager
                    .commit_txns(payload, &compute_result, committed.timestamp_usecs())
                    .await
                {
                    error!("Failed to notify mempool: {:?}", e);
                }
            }
        }
        if finality_proof.ledger_info().next_validator_set().is_some() {
            self.network
                .broadcast_epoch_change(ValidatorChangeProof::new(
                    vec![finality_proof],
                    /* more = */ false,
                ))
                .await
        }
    }

    /// Retrieve a n chained blocks from the block store starting from
    /// an initial parent id, returning with <n (as many as possible) if
    /// id or its ancestors can not be found.
    ///
    /// The current version of the function is not really async, but keeping it this way for
    /// future possible changes.
    pub async fn process_block_retrieval(&self, request: IncomingBlockRetrievalRequest) {
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

        let response = BlockRetrievalResponse::new(status, blocks);
        if let Err(e) = response
            .try_into()
            .and_then(|proto| {
                let bytes = ConsensusMsg {
                    message: Some(ConsensusMsg_oneof::RespondBlock(proto)),
                }
                .to_bytes()?;
                Ok(bytes)
            })
            .and_then(|response_data| {
                request
                    .response_sender
                    .send(Ok(response_data))
                    .map_err(|e| format_err!("{:?}", e))
            })
        {
            error!("Failed to return the requested block: {:?}", e);
        }
    }

    /// To jump start new round with the current certificates we have.
    pub async fn start(&mut self) {
        let hqc_round = Some(
            self.block_store
                .highest_quorum_cert()
                .certified_block()
                .round(),
        );
        let htc_round = self.block_store.highest_timeout_cert().map(|tc| tc.round());
        let last_committed_round = Some(self.block_store.root().round());
        let new_round_event = self
            .pacemaker
            .process_certificates(hqc_round, htc_round, last_committed_round)
            .expect("Can not jump start a pacemaker from existing certificates.");
        self.process_new_round_event(new_round_event).await;
    }

    /// Inspect the current consensus state.
    #[cfg(test)]
    pub fn consensus_state(&mut self) -> ConsensusState {
        self.safety_rules.consensus_state().unwrap()
    }

    pub fn block_store(&self) -> Arc<BlockStore<T>> {
        self.block_store.clone()
    }
}
