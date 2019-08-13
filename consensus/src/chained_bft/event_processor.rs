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
        },
        liveness::{
            pacemaker::{NewRoundEvent, NewRoundReason, Pacemaker},
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
        },
        network::{
            BlockRetrievalRequest, BlockRetrievalResponse, ChunkRetrievalRequest,
            ConsensusNetworkImpl,
        },
        persistent_storage::PersistentStorage,
        safety::{safety_rules::SafetyRules, vote_msg::VoteMsg},
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
use nextgen_crypto::ed25519::*;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use termion::color::*;
use types::ledger_info::LedgerInfoWithSignatures;

/// Result of initial proposal processing
/// - Done(proposal_option) indicates that the proposal is processed, and `proposal_option` contains
/// winning proposal if it was found
/// - NeedSync means separate task mast be spawned for state synchronization in the background
#[derive(Debug, PartialEq, Eq)]
pub enum ProcessProposalResult<T> {
    Done(Option<ProposalMsg<T>>),
    NeedSync(ProposalMsg<T>),
}

/// Consensus SMR is working in an event based fashion: EventProcessor is responsible for
/// processing the individual events (e.g., process_new_round, process_proposal, process_vote,
/// etc.). It is exposing the async processing functions for each event type.
/// The caller is responsible for running the event loops and driving the execution via some
/// executors.
pub struct EventProcessor<T> {
    author: Author,
    block_store: Arc<BlockStore<T>>,
    pacemaker: Arc<dyn Pacemaker>,
    proposer_election: Arc<dyn ProposerElection<T> + Send + Sync>,
    proposal_generator: ProposalGenerator<T>,
    safety_rules: Arc<RwLock<SafetyRules<T>>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    txn_manager: Arc<dyn TxnManager<Payload = T>>,
    network: ConsensusNetworkImpl,
    storage: Arc<dyn PersistentStorage<T>>,
    sync_manager: SyncManager<T>,
    time_service: Arc<dyn TimeService>,
    enforce_increasing_timestamps: bool,
    // Cache of the last sent vote message.
    last_vote_sent: Arc<RwLock<Option<(VoteMsg, Round)>>>,
}

impl<T: Payload> EventProcessor<T> {
    pub fn new(
        author: Author,
        block_store: Arc<BlockStore<T>>,
        pacemaker: Arc<dyn Pacemaker>,
        proposer_election: Arc<dyn ProposerElection<T> + Send + Sync>,
        proposal_generator: ProposalGenerator<T>,
        safety_rules: Arc<RwLock<SafetyRules<T>>>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        txn_manager: Arc<dyn TxnManager<Payload = T>>,
        network: ConsensusNetworkImpl,
        storage: Arc<dyn PersistentStorage<T>>,
        time_service: Arc<dyn TimeService>,
        enforce_increasing_timestamps: bool,
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
            last_vote_sent: Arc::new(RwLock::new(None)),
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
    pub async fn process_new_round_event(&self, new_round_event: NewRoundEvent) {
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

        // Proposal generator will ensure that at most one proposal is generated per round
        let proposal = match self
            .proposal_generator
            .generate_proposal(
                new_round_event.round,
                self.pacemaker.current_round_deadline(),
            )
            .await
        {
            Err(e) => {
                error!("Error while generating proposal: {:?}", e);
                return;
            }
            Ok(proposal) => proposal,
        };
        let mut network = self.network.clone();
        debug!("Propose {}", proposal);
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
        network
            .broadcast_proposal(ProposalMsg {
                proposal,
                sync_info,
            })
            .await;
        counters::PROPOSALS_COUNT.inc();
    }

    /// The function is responsible for processing the incoming proposals and the Quorum
    /// Certificate. 1. commit to the committed state the new QC carries
    /// 2. fetch all the blocks from the committed state to the QC
    /// 3. forwarding the proposals to the ProposerElection queue,
    /// which is going to eventually trigger one winning proposal per round
    /// (to be processed via a separate function).
    /// The reason for separating `process_proposal` from `process_winning_proposal` is to
    /// (a) asynchronously prefetch dependencies and
    /// (b) allow the proposer election to choose one proposal out of many.
    pub async fn process_proposal(&self, proposal: ProposalMsg<T>) -> ProcessProposalResult<T> {
        debug!("Receive proposal {}", proposal);
        // Pacemaker is going to be updated with all the proposal certificates later,
        // but it's known that the pacemaker's round is not going to decrease so we can already
        // filter out the proposals from old rounds.
        let current_round = self.pacemaker.current_round();
        self.help_remote_if_stale(
            proposal.proposer(),
            proposal.proposal.round(),
            proposal
                .sync_info
                .highest_quorum_cert()
                .certified_block_round(),
        )
        .await;
        if proposal.proposal.round() < self.pacemaker.current_round() {
            warn!(
                "Proposal {} is ignored because its round {} < current round {}",
                proposal,
                proposal.proposal.round(),
                current_round
            );
            return ProcessProposalResult::Done(None);
        }
        if self
            .proposer_election
            .is_valid_proposer(proposal.proposer(), proposal.proposal.round())
            .is_none()
        {
            warn!(
                "Proposer {} for block {} is not a valid proposer for this round",
                proposal.proposer(),
                proposal.proposal
            );
            return ProcessProposalResult::Done(None);
        }

        match self
            .block_store
            .need_fetch_for_quorum_cert(proposal.proposal.quorum_cert())
        {
            NeedFetchResult::NeedFetch => {
                return ProcessProposalResult::NeedSync(proposal);
            }
            NeedFetchResult::QCRoundBeforeRoot => {
                warn!("Proposal {} has a highest quorum certificate with round older than root round {}", proposal, self.block_store.root().round());
                return ProcessProposalResult::Done(None);
            }
            NeedFetchResult::QCBlockExist => {
                if let Err(e) = self
                    .block_store
                    .insert_single_quorum_cert(proposal.proposal.quorum_cert().clone())
                {
                    warn!(
                        "Quorum certificate for proposal {} could not be inserted to the block store: {:?}",
                        proposal, e
                    );
                    return ProcessProposalResult::Done(None);
                }
            }
            NeedFetchResult::QCAlreadyExist => (),
        }
        ProcessProposalResult::Done(self.finish_proposal_processing(proposal).await)
    }

    /// Finish proposal processing: note that multiple tasks can execute this function in parallel
    /// so be careful with the updates. The safest thing to do is to pass the proposal further
    /// to the proposal election.
    /// This function is invoked when all the dependencies for the given proposal are ready.
    async fn finish_proposal_processing(
        &self,
        proposal_msg: ProposalMsg<T>,
    ) -> Option<ProposalMsg<T>> {
        self.process_certificates(
            proposal_msg.proposal.quorum_cert(),
            proposal_msg.sync_info.highest_timeout_certificate(),
        )
        .await;

        let current_round = self.pacemaker.current_round();
        if self.pacemaker.current_round() != proposal_msg.proposal.round() {
            warn!(
                "Proposal {} is ignored because its round {} != current round {}",
                proposal_msg,
                proposal_msg.proposal.round(),
                current_round
            );
            return None;
        }

        self.proposer_election.process_proposal(proposal_msg)
    }

    /// Takes mutable reference to avoid race with other processing and perform state
    /// synchronization, then completes processing proposal in dedicated task
    pub async fn sync_and_process_proposal(
        &mut self,
        proposal: ProposalMsg<T>,
    ) -> Option<ProposalMsg<T>> {
        if let Err(e) = self
            .sync_up(&proposal.sync_info, Some(proposal.proposer()))
            .await
        {
            warn!(
                "Dependencies of proposal {} could not be added to the block store: {:?}",
                proposal, e
            );
            return None;
        }
        self.finish_proposal_processing(proposal).await
    }

    /// Upon receiving TimeoutMsg, ensure that any branches with higher quorum certificates are
    /// populated to this replica prior to processing the pacemaker timeout.  This ensures that when
    /// a pacemaker timeout certificate is formed with 2f+1 timeouts, the next proposer will be
    /// able to chain a proposal block to a highest quorum certificate such that all honest replicas
    /// can vote for it.
    pub async fn process_timeout_msg(&mut self, timeout_msg: TimeoutMsg, quorum_size: usize) {
        debug!(
            "Received timeout msg for round {} from {}",
            timeout_msg.pacemaker_timeout().round(),
            timeout_msg.author().short_str()
        );

        self.help_remote_if_stale(
            timeout_msg.author(),
            timeout_msg.pacemaker_timeout().round(),
            timeout_msg
                .sync_info()
                .highest_quorum_cert()
                .certified_block_round(),
        )
        .await;

        if self
            .sync_up(timeout_msg.sync_info(), Some(timeout_msg.author()))
            .await
            .is_err()
        {
            warn!("Stop timeout msg processing because of sync up error.");
            return;
        };
        if let Some(vote) = timeout_msg.pacemaker_timeout().vote_msg() {
            if let Some(_qc) = self.add_vote(vote.clone(), quorum_size).await {
                counters::TIMEOUT_VOTES_FORM_QC_COUNT.inc();
            }
        }
        self.pacemaker
            .process_remote_timeout(timeout_msg.pacemaker_timeout().clone())
            .await;
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
        if remote_round < self.pacemaker.current_round()
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
                "Peer {} is at round {} with hqc round {}, sending it a SyncInfo {}",
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
    /// of the given sync info.
    /// Returns Error in case sync mgr failed to bring the missing dependencies.
    async fn sync_up(
        &mut self,
        sync_info: &SyncInfo,
        preferred_peer: Option<Author>,
    ) -> failure::Result<()> {
        let current_hqc_round = self
            .block_store
            .highest_quorum_cert()
            .certified_block_round();
        let sync_info_hqc_round = sync_info.highest_quorum_cert().certified_block_round();

        if current_hqc_round >= sync_info_hqc_round {
            return Ok(());
        }

        debug!(
            "Starting sync: current_hqc_round = {}, sync_info_hqc_round = {}",
            current_hqc_round, sync_info_hqc_round,
        );
        let deadline = self.pacemaker.current_round_deadline();
        let sync_mgr_context = SyncMgrContext::new(sync_info, preferred_peer);
        self.sync_manager
            .sync_to(deadline, sync_mgr_context)
            .await
            .map_err(|e| {
                warn!(
                    "Fail to sync up to HQC @ round {}: {:?}",
                    sync_info_hqc_round, e
                );
                e
            })?;
        debug!("Caught up to HQC at round {}", sync_info_hqc_round);
        self.process_certificates(
            sync_info.highest_quorum_cert(),
            sync_info.highest_timeout_certificate(),
        )
        .await;
        Ok(())
    }

    pub async fn process_sync_info_msg(&mut self, sync_info: SyncInfo, peer: Author) {
        debug!("Received a sync info msg: {}", sync_info);
        counters::SYNC_INFO_MSGS_RECEIVED_COUNT.inc();
        // Bring missing dependencies, update the pacemaker.
        if let Err(e) = self.sync_up(&sync_info, Some(peer)).await {
            error!("Error processing sync info msg: {:?}", e);
        }
    }

    /// The replica stops voting for this round and saves its consensus state.  Voting is halted
    /// to ensure that the next proposer can make a proposal that can be voted on by all replicas.
    /// Saving the consensus state ensures that on restart, the replicas will not waste time
    /// on previous rounds.
    pub async fn process_outgoing_pacemaker_timeout(&self, round: Round) -> Option<TimeoutMsg> {
        let last_vote_sent = self.last_vote_sent.read().unwrap().as_ref().cloned();
        let vote_msg_to_attach = match last_vote_sent.as_ref() {
            Some((vote, vote_round)) if (*vote_round == round) => Some(vote.clone()),
            _ => {
                // Try to generate a NIL vote
                match self.gen_nil_vote(round).await {
                    Ok(nil_vote_msg) => {
                        self.last_vote_sent
                            .write()
                            .unwrap()
                            .replace((nil_vote_msg.clone(), round));
                        Some(nil_vote_msg)
                    }
                    Err(e) => {
                        warn!("Failed to generate a NIL vote: {}", e);
                        None
                    }
                }
            }
        };

        // Stop voting at this round, persist the consensus state to support restarting from
        // a recent round (i.e. > the last vote round)  and then send the highest quorum
        // certificate known
        let consensus_state = self
            .safety_rules
            .write()
            .unwrap()
            .increase_last_vote_round(round);
        if let Some(consensus_state) = consensus_state {
            if let Err(e) = self.storage.save_consensus_state(consensus_state) {
                error!("Failed to persist consensus state after increasing the last vote round due to {:?}", e);
                return None;
            }
        }

        let last_vote_round = self
            .safety_rules
            .read()
            .unwrap()
            .consensus_state()
            .last_vote_round();
        warn!(
            "Round {} timed out and {}, expected round proposer was {:?}, broadcasting new round to all replicas",
            round,
            if last_vote_round == round { "already executed and voted at this round" } else { "will never vote at this round" },
            self.proposer_election.get_valid_proposers(round),
        );

        Some(TimeoutMsg::new(
            SyncInfo::new(
                self.block_store.highest_quorum_cert().as_ref().clone(),
                self.block_store.highest_ledger_info().as_ref().clone(),
                self.pacemaker.highest_timeout_certificate(),
            ),
            PacemakerTimeout::new(round, self.block_store.signer(), vote_msg_to_attach),
            self.block_store.signer(),
        ))
    }

    async fn gen_nil_vote(&self, round: Round) -> failure::Result<VoteMsg> {
        let block = self.proposal_generator.generate_nil_block(round)?;
        self.execute_and_vote(block).await
    }

    async fn process_certificates(
        &self,
        qc: &QuorumCert,
        tc: Option<&PacemakerTimeoutCertificate>,
    ) {
        self.safety_rules.write().unwrap().update(qc);

        let mut highest_commit_round = None;
        if let Some(new_commit) = qc.committed_block_id() {
            if let Some(block) = self.block_store.get_block(new_commit) {
                let finality_proof = qc.ledger_info().clone();
                highest_commit_round = Some(block.round());
                self.process_commit(block, finality_proof).await;
            }
        }
        self.pacemaker
            .process_certificates(qc.certified_block_round(), highest_commit_round, tc)
            .await;
    }

    /// This function processes a proposal_msg that was chosen as a representative of its round:
    /// 1. Add it to a block store.
    /// 2. Try to vote for it following the safety rules.
    /// 3. In case a validator chooses to vote, send the vote to the representatives at the next
    /// position.
    pub async fn process_winning_proposal(&self, proposal_msg: ProposalMsg<T>) {
        if let Some(time_to_receival) = duration_since_epoch().checked_sub(Duration::from_micros(
            proposal_msg.proposal.timestamp_usecs(),
        )) {
            counters::CREATION_TO_RECEIVAL_MS.observe(time_to_receival.as_millis() as f64);
        }

        let proposal_round = proposal_msg.proposal.round();
        let vote_msg = match self.execute_and_vote(proposal_msg.proposal).await {
            Err(_) => {
                return;
            }
            Ok(vote_msg) => vote_msg,
        };

        self.last_vote_sent
            .write()
            .unwrap()
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
                            counters::VOTE_SUCCESS_WAIT_MS
                                .observe(wait_duration.as_millis() as f64);
                            counters::VOTE_WAIT_WAS_REQUIRED_COUNT.inc();
                        }
                        WaitingSuccess::NoWaitRequired { .. } => {
                            counters::VOTE_SUCCESS_WAIT_MS.observe(0.0);
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
                            counters::VOTE_FAILURE_WAIT_MS.observe(0.0);
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
                            counters::VOTE_FAILURE_WAIT_MS
                                .observe(wait_duration.as_millis() as f64);
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
    async fn execute_and_vote(&self, proposed_block: Block<T>) -> failure::Result<VoteMsg> {
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
            .write()
            .unwrap()
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
        let executed_state = self
            .block_store
            .get_state_for_block(proposal_id)
            .expect("Block proposed_block: no execution state found for inserted block.");

        let ledger_info_placeholder = self
            .block_store
            .ledger_info_placeholder(vote_info.potential_commit_id());
        Ok(VoteMsg::new(
            proposal_id,
            executed_state,
            block.round(),
            vote_info.parent_block_id(),
            vote_info.parent_block_round(),
            vote_info.grandparent_block_id(),
            vote_info.grandparent_block_round(),
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
    #[allow(clippy::collapsible_if)] // Collapsing here would make if look ugly
    pub async fn process_vote(&self, vote: VoteMsg, quorum_size: usize) {
        // Check whether this validator is a valid recipient of the vote.
        let next_round = vote.round() + 1;
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

        self.add_vote(vote, quorum_size).await;
    }

    /// Add a vote. Fetch missing dependencies if required.
    /// The `duplicates_expected` field is used for cases, in which some of the votes might
    /// be duplicated (e.g., when the votes are attached to the timeout messages).
    /// If a QC is formed then
    /// 1) fetch missing dependencies if required, and then
    /// 2) pass the new QC to the pacemaker, which can generate a new round in return.
    /// The function returns an Option for a newly generate QuorumCert in case it's been
    /// successfully added with all its dependencies.
    async fn add_vote(&self, vote: VoteMsg, quorum_size: usize) -> Option<Arc<QuorumCert>> {
        let deadline = self.pacemaker.current_round_deadline();
        let preferred_peer = Some(vote.author());
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
        finality_proof: LedgerInfoWithSignatures<Ed25519Signature>,
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
                counters::CREATION_TO_COMMIT_MS.observe(time_to_commit.as_millis() as f64);
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

    /// Retrieve the chunk from storage and send it back.
    /// We'll also try to add the QuorumCert into block store if it's for a existing block and
    /// potentially commit.
    pub async fn process_chunk_retrieval(&self, request: ChunkRetrievalRequest) {
        let response = self
            .sync_manager
            .get_chunk(
                request.start_version,
                request.target_version,
                request.batch_size,
            )
            .await;

        if let Err(e) = request.response_sender.send(response) {
            error!("Failed to return the requested chunk: {:?}", e);
        }
    }

    /// Inspect the current consensus state.
    #[cfg(test)]
    pub fn consensus_state(&self) -> ConsensusState {
        self.safety_rules.read().unwrap().consensus_state()
    }
}
