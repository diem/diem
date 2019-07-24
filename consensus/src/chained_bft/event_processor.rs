// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use crate::chained_bft::safety::safety_rules::ConsensusState;
use crate::{
    chained_bft::{
        block_storage::{
            BlockReader, BlockStore, InsertError, NeedFetchResult, VoteReceptionResult,
        },
        common::{Author, Payload, Round},
        consensus_types::{
            block::Block,
            proposal_msg::ProposalMsg,
            quorum_cert::QuorumCert,
            sync_info::SyncInfo,
            timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate, TimeoutMsg},
            vote_data::VoteData,
            vote_msg::VoteMsg,
        },
        epoch_manager::EpochManager,
        liveness::{
            pacemaker::{NewRoundEvent, NewRoundReason, Pacemaker},
            proposal_generator::{ProposalGenerationError, ProposalGenerator},
            proposer_election::ProposerElection,
        },
        network::{BlockRetrievalRequest, BlockRetrievalResponse, ConsensusNetworkImpl},
        persistent_storage::PersistentStorage,
        safety::safety_rules::SafetyRules,
        sync_manager::{SyncManager, SyncMgrContext},
    },
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::{
        duration_since_epoch, wait_if_possible, TimeService, WaitingError, WaitingSuccess,
    },
};
use logger::prelude::*;
use network::proto::BlockRetrievalStatus;
use std::{sync::Arc, time::Duration};
use termion::color::*;
use types::crypto_proxies::LedgerInfoWithSignatures;

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
    author: Author,
    block_store: Arc<BlockStore<T>>,
    pacemaker: Pacemaker,
    proposer_election: Box<dyn ProposerElection<T> + Send + Sync>,
    proposal_generator: ProposalGenerator<T>,
    safety_rules: SafetyRules,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    txn_manager: Arc<dyn TxnManager<Payload = T>>,
    network: ConsensusNetworkImpl,
    storage: Arc<dyn PersistentStorage<T>>,
    sync_manager: SyncManager<T>,
    time_service: Arc<dyn TimeService>,
    enforce_increasing_timestamps: bool,
    // Cache of the last sent vote message.
    last_vote_sent: Option<(VoteMsg, Round)>,
    epoch_mgr: Arc<EpochManager>,
}

impl<T: Payload> EventProcessor<T> {
    pub fn new(
        author: Author,
        block_store: Arc<BlockStore<T>>,
        pacemaker: Pacemaker,
        proposer_election: Box<dyn ProposerElection<T> + Send + Sync>,
        proposal_generator: ProposalGenerator<T>,
        safety_rules: SafetyRules,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        txn_manager: Arc<dyn TxnManager<Payload = T>>,
        network: ConsensusNetworkImpl,
        storage: Arc<dyn PersistentStorage<T>>,
        time_service: Arc<dyn TimeService>,
        enforce_increasing_timestamps: bool,
        epoch_mgr: Arc<EpochManager>,
    ) -> Self {
        let sync_manager = SyncManager::new(
            Arc::clone(&block_store),
            Arc::clone(&storage),
            network.clone(),
            Arc::clone(&state_computer),
        );
        Self {
            author,
            block_store,
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules,
            state_computer,
            txn_manager,
            network,
            storage,
            sync_manager,
            time_service,
            enforce_increasing_timestamps,
            last_vote_sent: None,
            epoch_mgr,
        }
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
    async fn process_new_round_event(&self, new_round_event: NewRoundEvent) {
        debug!("Processing {}", new_round_event);
        counters::CURRENT_ROUND.set(new_round_event.round as i64);
        counters::ROUND_TIMEOUT_MS.set(new_round_event.timeout.as_millis() as i64);
        match new_round_event.reason {
            NewRoundReason::QCReady => {
                counters::QC_ROUNDS_COUNT.inc();
            }
            NewRoundReason::Timeout { .. } => {
                counters::TIMEOUT_ROUNDS_COUNT.inc();
            }
        };
        if self
            .proposer_election
            .is_valid_proposer(self.author, new_round_event.round)
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
        &self,
        new_round_event: NewRoundEvent,
    ) -> Result<ProposalMsg<T>, ProposalGenerationError> {
        // Proposal generator will ensure that at most one proposal is generated per round
        let proposal = self
            .proposal_generator
            .generate_proposal(
                new_round_event.round,
                self.pacemaker.current_round_deadline(),
            )
            .await?;
        debug!("Propose {}", proposal);
        // should we include a TC?
        let timeout_certificate = match &new_round_event.reason {
            NewRoundReason::Timeout { cert }
                if cert.round() > proposal.quorum_cert().certified_block_round() =>
            {
                Some(cert.clone())
            }
            _ => None,
        };

        let sync_info = SyncInfo::new(
            (*proposal.quorum_cert()).clone(),
            (*self.block_store.highest_ledger_info()).clone(),
            timeout_certificate,
        );
        // return proposal
        Ok(ProposalMsg {
            proposal,
            sync_info,
        })
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
    /// 1. sync up to the SyncInfo including committing to the committed state the HLI carries
    /// and fetch all the blocks from the committed state to the HQC
    /// 2. forwarding the proposals to the ProposerElection queue,
    /// which is going to eventually trigger one winning proposal per round
    async fn pre_process_proposal(&mut self, proposal_msg: ProposalMsg<T>) -> Option<Block<T>> {
        debug!("EventProcessor: receive proposal {}", proposal_msg);
        // Pacemaker is going to be updated with all the proposal certificates later,
        // but it's known that the pacemaker's round is not going to decrease so we can already
        // filter out the proposals from old rounds.
        let current_round = self.pacemaker.current_round();
        if proposal_msg.proposal.round() < current_round {
            warn!(
                "Proposal {} is ignored because its round {} < current round {}",
                proposal_msg,
                proposal_msg.proposal.round(),
                current_round
            );
            return None;
        }
        if self
            .proposer_election
            .is_valid_proposer(proposal_msg.proposer(), proposal_msg.proposal.round())
            .is_none()
        {
            warn!(
                "Proposer {} for block {} is not a valid proposer for this round",
                proposal_msg.proposer(),
                proposal_msg.proposal
            );
            return None;
        }
        if let Err(e) = self
            .sync_up(&proposal_msg.sync_info, proposal_msg.proposer(), true)
            .await
        {
            warn!(
                "Dependencies of proposal {} could not be added to the block store: {:?}",
                proposal_msg, e
            );
            return None;
        }

        // pacemaker may catch up with the SyncInfo, check again
        let current_round = self.pacemaker.current_round();
        if proposal_msg.proposal.round() != current_round {
            warn!(
                "Proposal {} is ignored because its round {} != current round {}",
                proposal_msg,
                proposal_msg.proposal.round(),
                current_round
            );
            return None;
        }

        self.proposer_election
            .process_proposal(proposal_msg.proposal)
    }

    /// Upon receiving TimeoutMsg, ensure that any branches with higher quorum certificates are
    /// populated to this replica prior to processing the pacemaker timeout.  This ensures that when
    /// a pacemaker timeout certificate is formed with 2f+1 timeouts, the next proposer will be
    /// able to chain a proposal block to a highest quorum certificate such that all honest replicas
    /// can vote for it.
    pub async fn process_remote_timeout_msg(&mut self, timeout_msg: TimeoutMsg) {
        debug!(
            "Received timeout msg for round {} from {}",
            timeout_msg.pacemaker_timeout().round(),
            timeout_msg.author().short_str()
        );

        if self
            .sync_up(timeout_msg.sync_info(), timeout_msg.author(), true)
            .await
            .is_err()
        {
            warn!("Stop timeout msg processing because of sync up error.");
            return;
        };
        if let Some(vote) = timeout_msg.pacemaker_timeout().vote_msg() {
            self.add_vote(vote.clone(), self.epoch_mgr.quorum_size())
                .await;
        }
        if let Some(new_round_event) = self.pacemaker.process_remote_timeout(
            timeout_msg.pacemaker_timeout().clone(),
            self.epoch_mgr.quorum_size(),
        ) {
            self.process_new_round_event(new_round_event).await;
        }
    }

    /// In case some peer's round or HQC is stale, send a SyncInfo message to that peer.
    async fn help_remote_if_stale(
        &self,
        peer: Author,
        remote_round: Round,
        remote_hqc_round: Round,
    ) {
        if self.author == peer {
            return;
        }
        // pacemaker's round is sync_info.highest_round() + 1
        if remote_round + 1 < self.pacemaker.current_round()
            || remote_hqc_round
                < self
                    .block_store
                    .highest_quorum_cert()
                    .certified_block_round()
        {
            let sync_info = SyncInfo::new(
                self.block_store.highest_quorum_cert().as_ref().clone(),
                self.block_store.highest_ledger_info().as_ref().clone(),
                self.pacemaker.highest_timeout_certificate(),
            );

            debug!(
                "Peer {} is at round {} with hqc round {}, sending it {}",
                peer.short_str(),
                remote_round,
                remote_hqc_round,
                sync_info,
            );
            counters::SYNC_INFO_MSGS_SENT_COUNT.inc();
            self.network.send_sync_info(sync_info, peer).await;
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
    ) -> failure::Result<()> {
        if help_remote {
            self.help_remote_if_stale(author, sync_info.highest_round(), sync_info.hqc_round())
                .await;
        }

        let current_hqc_round = self
            .block_store
            .highest_quorum_cert()
            .certified_block_round();

        if current_hqc_round < sync_info.hqc_round() {
            debug!(
                "Starting sync: current_hqc_round = {}, sync_info_hqc_round = {}",
                current_hqc_round,
                sync_info.hqc_round(),
            );
            let deadline = self.pacemaker.current_round_deadline();
            let sync_mgr_context = SyncMgrContext::new(sync_info, author);
            self.sync_manager
                .sync_to(deadline, sync_mgr_context)
                .await
                .map_err(|e| {
                    warn!(
                        "Fail to sync up to HQC @ round {}: {:?}",
                        sync_info.hqc_round(),
                        e
                    );
                    e
                })?;
            debug!("Caught up to HQC at round {}", sync_info.hqc_round());
        }

        self.process_certificates(
            &sync_info.highest_quorum_cert(),
            sync_info.highest_timeout_certificate(),
        )
        .await;
        Ok(())
    }

    /// Process the SyncInfo sent by peers to catch up to latest state.
    pub async fn process_sync_info_msg(&mut self, sync_info: SyncInfo, peer: Author) {
        debug!("Received a sync info msg: {}", sync_info);
        counters::SYNC_INFO_MSGS_RECEIVED_COUNT.inc();
        // To avoid a ping-pong cycle between two peers that move forward together.
        if let Err(e) = self.sync_up(&sync_info, peer, false).await {
            error!("Fail to process sync info: {:?}", e);
        }
    }

    /// The replica stops voting for this round and saves its consensus state.  Voting is halted
    /// to ensure that the next proposer can make a proposal that can be voted on by all replicas.
    /// Saving the consensus state ensures that on restart, the replicas will not waste time
    /// on previous rounds.
    pub async fn process_local_timeout(&mut self, round: Round) {
        if !self.pacemaker.process_local_timeout(round) {
            return;
        }
        let last_vote_round = self.safety_rules.consensus_state().last_vote_round();
        warn!(
            "Round {} timed out: {}, expected round proposer was {:?}, broadcasting new round to all replicas",
            round,
            if last_vote_round == round { "already executed and voted at this round" } else { "will try to generate a backup vote" },
            self.proposer_election.get_valid_proposers(round).iter().map(|p| p.short_str()).collect::<Vec<String>>(),
        );

        let vote_msg_to_attach = match self.last_vote_sent.as_ref() {
            Some((vote, vote_round)) if (*vote_round == round) => Some(vote.clone()),
            _ => {
                // Try to generate a backup vote
                match self.gen_backup_vote(round).await {
                    Ok(backup_vote_msg) => {
                        self.last_vote_sent
                            .replace((backup_vote_msg.clone(), round));
                        Some(backup_vote_msg)
                    }
                    Err(e) => {
                        warn!("Failed to generate a backup vote: {}", e);
                        None
                    }
                }
            }
        };

        // Stop voting at this round, persist the consensus state to support restarting from
        // a recent round (i.e. > the last vote round)  and then send the SyncInfo
        let consensus_state = self.safety_rules.increase_last_vote_round(round);

        if let Some(consensus_state) = consensus_state {
            if let Err(e) = self.storage.save_consensus_state(consensus_state) {
                error!("Failed to persist consensus state after increasing the last vote round due to {:?}", e);
                return;
            }
        }

        self.network
            .broadcast_timeout_msg(TimeoutMsg::new(
                SyncInfo::new(
                    self.block_store.highest_quorum_cert().as_ref().clone(),
                    self.block_store.highest_ledger_info().as_ref().clone(),
                    self.pacemaker.highest_timeout_certificate(),
                ),
                PacemakerTimeout::new(round, self.block_store.signer(), vote_msg_to_attach),
                self.block_store.signer(),
            ))
            .await;
    }

    async fn gen_backup_vote(&mut self, round: Round) -> failure::Result<VoteMsg> {
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

    async fn process_certificates(
        &mut self,
        qc: &QuorumCert,
        tc: Option<&PacemakerTimeoutCertificate>,
    ) {
        self.safety_rules.update(qc);

        let mut highest_committed_proposal_round = None;
        if let Some(new_commit) = qc.committed_block_id() {
            if let Some(block) = self.block_store.get_block(new_commit) {
                let finality_proof = qc.ledger_info().clone();
                // We don't want to use NIL commits for pacemaker round interval calculations.
                if !block.is_nil_block() {
                    highest_committed_proposal_round = Some(block.round());
                }
                self.process_commit(block, finality_proof).await;
            }
        }
        if let Some(new_round_event) = self.pacemaker.process_certificates(
            qc.certified_block_round(),
            highest_committed_proposal_round,
            tc,
        ) {
            self.process_new_round_event(new_round_event).await;
        }
    }

    /// This function processes a proposal that was chosen as a representative of its round:
    /// 1. Add it to a block store.
    /// 2. Try to vote for it following the safety rules.
    /// 3. In case a validator chooses to vote, send the vote to the representatives at the next
    /// position.
    async fn process_proposed_block(&mut self, proposal: Block<T>) {
        if let Some(time_to_receival) =
            duration_since_epoch().checked_sub(Duration::from_micros(proposal.timestamp_usecs()))
        {
            counters::CREATION_TO_RECEIVAL_S.observe_duration(time_to_receival);
        }

        let proposal_round = proposal.round();
        let vote_msg = match self.execute_and_vote(proposal).await {
            Err(_) => {
                return;
            }
            Ok(vote_msg) => vote_msg,
        };

        self.last_vote_sent
            .replace((vote_msg.clone(), proposal_round));
        let recipients = self
            .proposer_election
            .get_valid_proposers(proposal_round + 1);
        debug!("{}Voted: {} {}", Fg(Green), Fg(Reset), vote_msg);
        self.network.send_vote(vote_msg, recipients).await;
    }

    async fn wait_before_vote_if_needed(
        &self,
        block_timestamp_us: u64,
    ) -> Result<(), WaitingError> {
        let current_round_deadline = self.pacemaker.current_round_deadline();
        if self.enforce_increasing_timestamps {
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
                            counters::VOTE_WAIT_WAS_REQUIRED_COUNT.inc();
                        }
                        WaitingSuccess::NoWaitRequired { .. } => {
                            counters::VOTE_SUCCESS_WAIT_S.observe_duration(Duration::new(0, 0));
                            counters::VOTE_NO_WAIT_REQUIRED_COUNT.inc();
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
                            counters::VOTE_MAX_WAIT_EXCEEDED_COUNT.inc();
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
                            counters::VOTE_WAIT_FAILED_COUNT.inc();
                        }
                    };
                    return Err(waiting_error);
                }
            }
        }
        Ok(())
    }

    /// The function generates a VoteMsg for a given proposed_block:
    /// * first execute the block and add it to the block store
    /// * then verify the voting rules
    /// * save the updated state to consensus DB
    /// * return a VoteMsg with the LedgerInfo to be committed in case the vote gathers QC.
    ///
    /// This function assumes that it might be called from different tasks concurrently.
    async fn execute_and_vote(&mut self, proposed_block: Block<T>) -> failure::Result<VoteMsg> {
        let block = self
            .sync_manager
            .execute_and_insert_block(proposed_block)
            .await
            .map_err(|e| {
                debug!("Failed to execute_and_insert the block: {:?}", e);
                e
            })?;
        // Checking pacemaker round again, because multiple proposed_block can now race
        // during async block retrieval
        if self.pacemaker.current_round() != block.round() {
            debug!(
                "Proposal {} rejected because round is incorrect. Pacemaker: {}, proposed_block: {}",
                block,
                self.pacemaker.current_round(),
                block.round(),
            );
            return Err(InsertError::InvalidBlockRound.into());
        }
        self.wait_before_vote_if_needed(block.timestamp_usecs())
            .await?;

        let vote_info = self
            .safety_rules
            .voting_rule(Arc::clone(&block))
            .map_err(|e| {
                debug!("{}Rejected{} {}: {:?}", Fg(Red), Fg(Reset), block, e);
                e
            })?;
        self.storage
            .save_consensus_state(vote_info.consensus_state().clone())
            .map_err(|e| {
                debug!("Fail to persist consensus state: {:?}", e);
                e
            })?;

        let proposal_id = vote_info.proposal_id();
        let executed_state_id = self
            .block_store
            .get_compute_result(proposal_id)
            .expect("Block proposed_block: no execution state found for inserted block.")
            .executed_state
            .state_id;

        let ledger_info_placeholder = self
            .block_store
            .ledger_info_placeholder(vote_info.potential_commit_id());
        Ok(VoteMsg::new(
            VoteData::new(
                proposal_id,
                executed_state_id,
                block.round(),
                vote_info.parent_block_id(),
                vote_info.parent_block_round(),
                vote_info.grandparent_block_id(),
                vote_info.grandparent_block_round(),
            ),
            self.author,
            ledger_info_placeholder,
            self.block_store.signer(),
        ))
    }

    /// Upon new vote:
    /// 1. Filter out votes for rounds that should not be processed by this validator (to avoid
    /// potential attacks).
    /// 2. Add the vote to the store and check whether it finishes a QC.
    /// 3. Once the QC successfully formed, notify the Pacemaker.
    pub async fn process_vote(&mut self, vote: VoteMsg) {
        // Check whether this validator is a valid recipient of the vote.
        let next_round = vote.block_round() + 1;
        if self
            .proposer_election
            .is_valid_proposer(self.author, next_round)
            .is_none()
        {
            debug!(
                "Received {}, but I am not a valid proposer for round {}, ignore.",
                vote, next_round
            );
            security_log(SecurityEvent::InvalidConsensusVote)
                .error("InvalidProposer")
                .data(vote)
                .data(next_round)
                .log();
            return;
        }

        self.add_vote(vote, self.epoch_mgr.quorum_size()).await;
    }

    /// Add a vote. Fetch missing dependencies if required.
    /// The `duplicates_expected` field is used for cases, in which some of the votes might
    /// be duplicated (e.g., when the votes are attached to the timeout messages).
    /// If a QC is formed then
    /// 1) fetch missing dependencies if required, and then
    /// 2) pass the new QC to the pacemaker, which can generate a new round in return.
    /// The function returns an Option for a newly generate QuorumCert in case it's been
    /// successfully added with all its dependencies.
    async fn add_vote(&mut self, vote: VoteMsg, quorum_size: usize) -> Option<Arc<QuorumCert>> {
        let deadline = self.pacemaker.current_round_deadline();
        let preferred_peer = vote.author();
        // TODO [Reconfiguration] Verify epoch of the vote message.
        // Add the vote and check whether it completes a new QC.
        if let VoteReceptionResult::NewQuorumCertificate(qc) =
            self.block_store.insert_vote(vote, quorum_size)
        {
            if self.block_store.need_fetch_for_quorum_cert(&qc) == NeedFetchResult::NeedFetch {
                if let Err(e) = self
                    .sync_manager
                    .fetch_quorum_cert(qc.as_ref().clone(), preferred_peer, deadline)
                    .await
                {
                    error!("Error syncing to qc {}: {:?}", qc, e);
                    return None;
                }
            } else if let Err(e) = self
                .block_store
                .insert_single_quorum_cert(qc.as_ref().clone())
            {
                error!("Error inserting qc {}: {:?}", qc, e);
                return None;
            }
            self.process_certificates(qc.as_ref(), None).await;
            return Some(qc);
        };
        None
    }

    /// Upon (potentially) new commit:
    /// 0. Verify that this commit is newer than the current root.
    /// 1. Notify state computer with the finality proof.
    /// 2. After the state is finalized, update the txn manager with the status of the committed
    /// transactions.
    /// 3. Prune the tree.
    async fn process_commit(
        &self,
        committed_block: Arc<Block<T>>,
        finality_proof: LedgerInfoWithSignatures,
    ) {
        // First make sure that this commit is new.
        if committed_block.round() <= self.block_store.root().round() {
            return;
        }

        // Verify that the ledger info is indeed for the block we're planning to
        // commit.
        assert_eq!(
            finality_proof.ledger_info().consensus_block_id(),
            committed_block.id()
        );

        if let Err(e) = self.state_computer.commit(finality_proof).await {
            // We assume that state computer cannot enter an inconsistent state that might
            // violate safety of the protocol. Specifically, an executor service is going to panic
            // if it fails to persist the commit requests, which would crash the whole process
            // including consensus.
            error!(
                "Failed to persist commit, mempool will not be notified: {:?}",
                e
            );
            return;
        }
        // At this moment the new state is persisted and we can notify the clients.
        // Multiple blocks might be committed at once: notify about all the transactions in the
        // path from the old root to the new root.
        for committed in self
            .block_store
            .path_from_root(Arc::clone(&committed_block))
            .unwrap_or_else(Vec::new)
        {
            if let Some(time_to_commit) = duration_since_epoch()
                .checked_sub(Duration::from_micros(committed.timestamp_usecs()))
            {
                counters::CREATION_TO_COMMIT_S.observe_duration(time_to_commit);
            }
            let compute_result = self
                .block_store
                .get_compute_result(committed.id())
                .expect("Compute result of a pending block is unknown");
            if let Err(e) = self
                .txn_manager
                .commit_txns(
                    committed.get_payload(),
                    compute_result.as_ref(),
                    committed.timestamp_usecs(),
                )
                .await
            {
                error!("Failed to notify mempool: {:?}", e);
            }
        }
        counters::LAST_COMMITTED_ROUND.set(committed_block.round() as i64);
        debug!("{}Committed{} {}", Fg(Blue), Fg(Reset), *committed_block);
        event!("committed",
            "block_id": committed_block.id().short_str(),
            "round": committed_block.round(),
            "parent_id": committed_block.parent_id().short_str(),
        );
        self.block_store.prune_tree(committed_block.id());
    }

    /// Retrieve a n chained blocks from the block store starting from
    /// an initial parent id, returning with <n (as many as possible) if
    /// id or its ancestors can not be found.
    ///
    /// The current version of the function is not really async, but keeping it this way for
    /// future possible changes.
    pub async fn process_block_retrieval(&self, request: BlockRetrievalRequest<T>) {
        let mut blocks = vec![];
        let mut status = BlockRetrievalStatus::SUCCEEDED;
        let mut id = request.block_id;
        while (blocks.len() as u64) < request.num_blocks {
            if let Some(block) = self.block_store.get_block(id) {
                id = block.parent_id();
                blocks.push(Block::clone(block.as_ref()));
            } else {
                status = BlockRetrievalStatus::NOT_ENOUGH_BLOCKS;
                break;
            }
        }

        if blocks.is_empty() {
            status = BlockRetrievalStatus::ID_NOT_FOUND;
        }

        if let Err(e) = request
            .response_sender
            .send(BlockRetrievalResponse { status, blocks })
        {
            error!("Failed to return the requested block: {:?}", e);
        }
    }

    /// To jump start new round with the current certificates we have.
    pub async fn start(&mut self) {
        let hqc = self.block_store.highest_quorum_cert();
        let last_committed_round = self.block_store.root().round();
        let new_round_event = self
            .pacemaker
            .process_certificates(
                hqc.certified_block_round(),
                Some(last_committed_round),
                None,
            )
            .expect("Can not jump start a new round from existing certificates.");
        self.process_new_round_event(new_round_event).await;
    }

    /// Inspect the current consensus state.
    #[cfg(test)]
    pub fn consensus_state(&self) -> ConsensusState {
        self.safety_rules.consensus_state()
    }
}
