// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::BlockStore,
    counters,
    liveness::{
        leader_reputation::{ActiveInactiveHeuristic, LeaderReputation, LibraDBBackend},
        proposal_generator::ProposalGenerator,
        proposer_election::ProposerElection,
        rotating_proposer_election::{choose_leader, RotatingProposer},
        round_proposer_election::RoundProposer,
        round_state::{ExponentialTimeInterval, RoundState},
    },
    metrics_safety_rules::MetricsSafetyRules,
    network::{IncomingBlockRetrievalRequest, NetworkReceivers, NetworkSender},
    network_interface::{ConsensusMsg, ConsensusNetworkSender},
    persistent_liveness_storage::{LedgerRecoveryData, PersistentLivenessStorage, RecoveryData},
    round_manager::{RecoveryManager, RoundManager, UnverifiedEvent, VerifiedEvent},
    state_replication::{StateComputer, TxnManager},
    util::time_service::TimeService,
};
use anyhow::{bail, ensure, Context};
use channel::libra_channel;
use consensus_types::{
    common::{Author, Round},
    epoch_retrieval::EpochRetrievalRequest,
};
use futures::{select, StreamExt};
use libra_config::config::{ConsensusConfig, ConsensusProposerType, NodeConfig};
use libra_logger::prelude::*;
use libra_metrics::monitor;
use libra_types::{
    account_address::AccountAddress,
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    on_chain_config::{OnChainConfigPayload, ValidatorSet},
};
use network::protocols::network::Event;
use safety_rules::SafetyRulesManager;
use std::{cmp::Ordering, sync::Arc, time::Duration};

/// RecoveryManager is used to process events in order to sync up with peer if we can't recover from local consensusdb
/// RoundManager is used for normal event handling.
/// We suppress clippy warning here because we expect most of the time we will have RoundManager
#[allow(clippy::large_enum_variant)]
pub enum RoundProcessor {
    Recovery(RecoveryManager),
    Normal(RoundManager),
}

#[allow(clippy::large_enum_variant)]
pub enum LivenessStorageData {
    RecoveryData(RecoveryData),
    LedgerRecoveryData(LedgerRecoveryData),
}

impl LivenessStorageData {
    pub fn expect_recovery_data(self, msg: &str) -> RecoveryData {
        match self {
            LivenessStorageData::RecoveryData(data) => data,
            LivenessStorageData::LedgerRecoveryData(_) => panic!("{}", msg),
        }
    }
}

// Manager the components that shared across epoch and spawn per-epoch RoundManager with
// epoch-specific input.
pub struct EpochManager {
    author: Author,
    config: ConsensusConfig,
    time_service: Arc<dyn TimeService>,
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Arc<dyn TxnManager>,
    state_computer: Arc<dyn StateComputer>,
    storage: Arc<dyn PersistentLivenessStorage>,
    safety_rules_manager: SafetyRulesManager,
    processor: Option<RoundProcessor>,
}

impl EpochManager {
    pub fn new(
        node_config: &NodeConfig,
        time_service: Arc<dyn TimeService>,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
        network_sender: ConsensusNetworkSender,
        timeout_sender: channel::Sender<Round>,
        txn_manager: Arc<dyn TxnManager>,
        state_computer: Arc<dyn StateComputer>,
        storage: Arc<dyn PersistentLivenessStorage>,
    ) -> Self {
        let author = node_config.validator_network.as_ref().unwrap().peer_id();
        let config = node_config.consensus.clone();
        let sr_config = &node_config.consensus.safety_rules;
        let safety_rules_manager = SafetyRulesManager::new(sr_config);
        Self {
            author,
            config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage,
            safety_rules_manager,
            processor: None,
        }
    }

    fn epoch_state(&self) -> &EpochState {
        match self
            .processor
            .as_ref()
            .expect("EpochManager not started yet")
        {
            RoundProcessor::Normal(p) => p.epoch_state(),
            RoundProcessor::Recovery(p) => p.epoch_state(),
        }
    }

    fn epoch(&self) -> u64 {
        self.epoch_state().epoch
    }

    fn create_round_state(
        &self,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
    ) -> RoundState {
        // 1.5^6 ~= 11
        // Timeout goes from initial_timeout to initial_timeout*11 in 6 steps
        let time_interval = Box::new(ExponentialTimeInterval::new(
            Duration::from_millis(self.config.round_initial_timeout_ms),
            1.2,
            6,
        ));
        RoundState::new(time_interval, time_service, timeout_sender)
    }

    /// Create a proposer election handler based on proposers
    fn create_proposer_election(
        &self,
        epoch_state: &EpochState,
    ) -> Box<dyn ProposerElection + Send + Sync> {
        let proposers = epoch_state
            .verifier
            .get_ordered_account_addresses_iter()
            .collect::<Vec<_>>();
        match &self.config.proposer_type {
            ConsensusProposerType::RotatingProposer => Box::new(RotatingProposer::new(
                proposers,
                self.config.contiguous_rounds,
            )),
            // We don't really have a fixed proposer!
            ConsensusProposerType::FixedProposer => {
                let proposer = choose_leader(proposers);
                Box::new(RotatingProposer::new(
                    vec![proposer],
                    self.config.contiguous_rounds,
                ))
            }
            ConsensusProposerType::LeaderReputation(heuristic_config) => {
                let backend = Box::new(LibraDBBackend::new(
                    proposers.len(),
                    self.storage.libra_db(),
                ));
                let heuristic = Box::new(ActiveInactiveHeuristic::new(
                    heuristic_config.active_weights,
                    heuristic_config.inactive_weights,
                ));
                Box::new(LeaderReputation::new(proposers, backend, heuristic))
            }
            ConsensusProposerType::RoundProposer(round_proposers) => {
                // Hardcoded to the first proposer
                let default_proposer = proposers.get(0).unwrap();
                Box::new(RoundProposer::new(
                    round_proposers.clone(),
                    *default_proposer,
                ))
            }
        }
    }

    async fn process_epoch_retrieval(
        &mut self,
        request: EpochRetrievalRequest,
        peer_id: AccountAddress,
    ) -> anyhow::Result<()> {
        debug!(
            "[EpochManager] receive {} from {}",
            request,
            peer_id.short_str()
        );
        let proof = self
            .storage
            .libra_db()
            .get_epoch_ending_ledger_infos(request.start_epoch, request.end_epoch)
            .context("[EpochManager] Failed to get epoch proof")?;
        let msg = ConsensusMsg::EpochChangeProof(Box::new(proof));
        self.network_sender.send_to(peer_id, msg).context(format!(
            "[EpochManager] Failed to send epoch proof to {}",
            peer_id
        ))
    }

    async fn process_different_epoch(
        &mut self,
        different_epoch: u64,
        peer_id: AccountAddress,
    ) -> anyhow::Result<()> {
        debug!(
            "[EpochManager] receive message from epoch {} from {}, local epoch: {}",
            different_epoch,
            peer_id,
            self.epoch()
        );
        match different_epoch.cmp(&self.epoch()) {
            // We try to help nodes that have lower epoch than us
            Ordering::Less => {
                self.process_epoch_retrieval(
                    EpochRetrievalRequest {
                        start_epoch: different_epoch,
                        end_epoch: self.epoch(),
                    },
                    peer_id,
                )
                .await
            }
            // We request proof to join higher epoch
            Ordering::Greater => {
                let request = EpochRetrievalRequest {
                    start_epoch: self.epoch(),
                    end_epoch: different_epoch,
                };
                let msg = ConsensusMsg::EpochRetrievalRequest(Box::new(request));
                self.network_sender.send_to(peer_id, msg).context(format!(
                    "[EpochManager] Failed to send epoch retrieval to {}",
                    peer_id
                ))
            }
            Ordering::Equal => {
                bail!("[EpochManager] Same epoch should not come to process_different_epoch");
            }
        }
    }

    async fn start_new_epoch(&mut self, proof: EpochChangeProof) -> anyhow::Result<()> {
        let ledger_info = proof
            .verify(self.epoch_state())
            .context("[EpochManager] Invalid EpochChangeProof")?;
        debug!(
            "Received epoch change to {}",
            ledger_info.ledger_info().epoch() + 1
        );

        // make sure storage is on this ledger_info too, it should be no-op if it's already committed
        self.state_computer
            .sync_to(ledger_info.clone())
            .await
            .context(format!(
                "[EpochManager] State sync to new epoch {}",
                ledger_info
            ))
        // state_computer notifies reconfiguration in another channel
    }

    async fn start_round_manager(&mut self, recovery_data: RecoveryData, epoch_state: EpochState) {
        // Release the previous RoundManager, especially the SafetyRule client
        self.processor = None;
        counters::EPOCH.set(epoch_state.epoch as i64);
        counters::CURRENT_EPOCH_VALIDATORS.set(epoch_state.verifier.len() as i64);
        info!(
            "Starting {} with root {}",
            epoch_state,
            recovery_data.root_block(),
        );
        let last_vote = recovery_data.last_vote();

        info!("Create BlockStore");
        let block_store = Arc::new(BlockStore::new(
            Arc::clone(&self.storage),
            recovery_data,
            Arc::clone(&self.state_computer),
            self.config.max_pruned_blocks_in_mem,
            Arc::clone(&self.time_service),
        ));

        info!("Update SafetyRules");

        let mut safety_rules =
            MetricsSafetyRules::new(self.safety_rules_manager.client(), self.storage.clone());
        safety_rules
            .perform_initialize()
            .expect("Unable to initialize SafetyRules");

        info!("Create ProposalGenerator");
        // txn manager is required both by proposal generator (to pull the proposers)
        // and by event processor (to update their status).
        let proposal_generator = ProposalGenerator::new(
            self.author,
            block_store.clone(),
            self.txn_manager.clone(),
            self.time_service.clone(),
            self.config.max_block_size,
        );

        info!("Create RoundState");
        let round_state =
            self.create_round_state(self.time_service.clone(), self.timeout_sender.clone());

        info!("Create ProposerElection");
        let proposer_election = self.create_proposer_election(&epoch_state);
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            epoch_state.verifier.clone(),
        );

        let mut processor = RoundManager::new(
            epoch_state,
            block_store,
            round_state,
            proposer_election,
            proposal_generator,
            safety_rules,
            network_sender,
            self.txn_manager.clone(),
            self.storage.clone(),
            self.config.sync_only,
        );
        processor.start(last_vote).await;
        self.processor = Some(RoundProcessor::Normal(processor));
        info!("RoundManager started");
    }

    // Depending on what data we can extract from consensusdb, we may or may not have an
    // event processor at startup. If we need to sync up with peers for blocks to construct
    // a valid block store, which is required to construct an event processor, we will take
    // care of the sync up here.
    async fn start_recovery_manager(
        &mut self,
        ledger_recovery_data: LedgerRecoveryData,
        epoch_state: EpochState,
    ) {
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            epoch_state.verifier.clone(),
        );
        self.processor = Some(RoundProcessor::Recovery(RecoveryManager::new(
            epoch_state,
            network_sender,
            self.storage.clone(),
            self.state_computer.clone(),
            ledger_recovery_data.commit_round(),
        )));
        info!("SyncProcessor started");
    }

    pub async fn start_processor(&mut self, payload: OnChainConfigPayload) {
        let validator_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");
        let epoch_state = EpochState {
            epoch: payload.epoch(),
            verifier: (&validator_set).into(),
        };

        match self.storage.start() {
            LivenessStorageData::RecoveryData(initial_data) => {
                self.start_round_manager(initial_data, epoch_state).await
            }
            LivenessStorageData::LedgerRecoveryData(ledger_recovery_data) => {
                self.start_recovery_manager(ledger_recovery_data, epoch_state)
                    .await
            }
        }
    }

    pub async fn process_message(
        &mut self,
        peer_id: AccountAddress,
        consensus_msg: ConsensusMsg,
    ) -> anyhow::Result<()> {
        // we can't verify signatures from a different epoch
        let maybe_unverified_event = self.process_epoch(peer_id, consensus_msg).await?;

        if let Some(unverified_event) = maybe_unverified_event {
            // same epoch -> run well-formedness + signature check
            let verified_event = unverified_event
                .clone()
                .verify(&self.epoch_state().verifier)
                .context("[EpochManager] Verify event")
                .map_err(|err| {
                    error!(
                        SecurityEvent::ConsensusInvalidMessage,
                        StructuredLogEntry::default()
                            .data("from_peer", &peer_id)
                            .data_display("error", &err)
                            .data("event", &unverified_event)
                    );
                    err
                })?;

            // process the verified event
            self.process_event(peer_id, verified_event).await?;
        }
        Ok(())
    }

    async fn process_epoch(
        &mut self,
        peer_id: AccountAddress,
        msg: ConsensusMsg,
    ) -> anyhow::Result<Option<UnverifiedEvent>> {
        match msg {
            ConsensusMsg::ProposalMsg(_) | ConsensusMsg::SyncInfo(_) | ConsensusMsg::VoteMsg(_) => {
                let event: UnverifiedEvent = msg.into();
                if event.epoch() == self.epoch() {
                    return Ok(Some(event));
                } else {
                    monitor!(
                        "process_different_epoch_consensus_msg",
                        self.process_different_epoch(event.epoch(), peer_id).await?
                    );
                }
            }
            ConsensusMsg::EpochChangeProof(proof) => {
                let msg_epoch = proof.epoch()?;
                if msg_epoch == self.epoch() {
                    monitor!("process_epoch_proof", self.start_new_epoch(*proof).await?);
                } else {
                    bail!(
                        "[EpochManager] Unexpected epoch proof from epoch {}, local epoch {}",
                        msg_epoch,
                        self.epoch()
                    );
                }
            }
            ConsensusMsg::EpochRetrievalRequest(request) => {
                ensure!(
                    request.end_epoch <= self.epoch(),
                    "[EpochManager] Received EpochRetrievalRequest beyond what we have locally"
                );
                monitor!(
                    "process_epoch_retrieval",
                    self.process_epoch_retrieval(*request, peer_id).await?
                );
            }
            _ => {
                bail!("[EpochManager] Unexpected messages: {:?}", msg);
            }
        }
        Ok(None)
    }

    async fn process_event(
        &mut self,
        peer_id: AccountAddress,
        event: VerifiedEvent,
    ) -> anyhow::Result<()> {
        match self.processor_mut() {
            RoundProcessor::Recovery(p) => {
                let recovery_data = match event {
                    VerifiedEvent::ProposalMsg(proposal) => p.process_proposal_msg(*proposal).await,
                    VerifiedEvent::VoteMsg(vote) => p.process_vote_msg(*vote).await,
                    VerifiedEvent::SyncInfo(sync_info) => p.sync_up(&sync_info, peer_id).await,
                }?;
                let epoch_state = p.epoch_state().clone();
                info!("Recovered from SyncProcessor");
                self.start_round_manager(recovery_data, epoch_state).await;
                Ok(())
            }
            RoundProcessor::Normal(p) => match event {
                VerifiedEvent::ProposalMsg(proposal) => {
                    monitor!("process_proposal", p.process_proposal_msg(*proposal).await)
                }
                VerifiedEvent::VoteMsg(vote) => {
                    monitor!("process_vote", p.process_vote_msg(*vote).await)
                }
                VerifiedEvent::SyncInfo(sync_info) => monitor!(
                    "process_sync_info",
                    p.process_sync_info_msg(*sync_info, peer_id).await
                ),
            },
        }
    }

    fn processor_mut(&mut self) -> &mut RoundProcessor {
        self.processor
            .as_mut()
            .expect("[EpochManager] not started yet")
    }

    pub async fn process_block_retrieval(
        &mut self,
        request: IncomingBlockRetrievalRequest,
    ) -> anyhow::Result<()> {
        match self.processor_mut() {
            RoundProcessor::Normal(p) => p.process_block_retrieval(request).await,
            _ => bail!("[EpochManager] RoundManager not started yet"),
        }
    }

    pub async fn process_local_timeout(&mut self, round: u64) -> anyhow::Result<()> {
        match self.processor_mut() {
            RoundProcessor::Normal(p) => p.process_local_timeout(round).await,
            _ => unreachable!("RoundManager not started yet"),
        }
    }

    pub async fn start(
        mut self,
        mut round_timeout_sender_rx: channel::Receiver<Round>,
        mut network_receivers: NetworkReceivers,
        mut reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) {
        // initial start of the processor
        if let Some(payload) = reconfig_events.next().await {
            self.start_processor(payload).await;
        }
        loop {
            let result = monitor!(
                "main_loop",
                select! {
                    payload = reconfig_events.select_next_some() => {
                        monitor!("reconfig", self.start_processor(payload).await);
                        Ok(())
                    }
                    msg = network_receivers.consensus_messages.select_next_some() => {
                        monitor!("process_message", self.process_message(msg.0, msg.1).await)
                    }
                    block_retrieval = network_receivers.block_retrieval.select_next_some() => {
                        monitor!("process_block_retrieval", self.process_block_retrieval(block_retrieval).await)
                    }
                    round = round_timeout_sender_rx.select_next_some() => {
                        monitor!("process_local_timeout", self.process_local_timeout(round).await)
                    }
                }
            );
            let round_state = if let RoundProcessor::Normal(p) = self.processor_mut() {
                p.round_state().to_string()
            } else {
                "RoundState: None".into()
            };
            match result {
                Ok(_) => trace!("{}", round_state),
                Err(e) => {
                    counters::ERROR_COUNT.inc();
                    error!("{:?}, {}", e, round_state);
                }
            }
        }
    }
}
