// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        event_processor::{EventProcessor, SyncProcessor, UnverifiedEvent, VerifiedEvent},
        liveness::{
            leader_reputation::{ActiveInactiveHeuristic, LeaderReputation, LibraDBBackend},
            multi_proposer_election::MultiProposer,
            pacemaker::{ExponentialTimeInterval, Pacemaker},
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::{choose_leader, RotatingProposer},
        },
        network::{IncomingBlockRetrievalRequest, NetworkSender},
        network_interface::{ConsensusMsg, ConsensusNetworkSender},
        persistent_liveness_storage::{
            LedgerRecoveryData, PersistentLivenessStorage, RecoveryData,
        },
    },
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::{ClockTimeService, TimeService},
};
use anyhow::anyhow;
use consensus_types::{
    common::{Author, Payload, Round},
    epoch_retrieval::EpochRetrievalRequest,
};
use libra_config::config::{ConsensusConfig, ConsensusProposerType};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    epoch_info::EpochInfo,
    validator_change::{ValidatorChangeProof, VerifierType},
};
use network::protocols::network::Event;
use safety_rules::SafetyRulesManager;
use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
    time::Duration,
};

/// The enum contains two processor
/// SyncProcessor is used to process events in order to sync up with peer if we can't recover from local consensusdb
/// EventProcessor is used for normal event handling.
/// We suppress clippy warning here because we expect most of the time we will have EventProcessor
#[allow(clippy::large_enum_variant)]
pub enum Processor<T> {
    SyncProcessor(SyncProcessor<T>),
    EventProcessor(EventProcessor<T>),
}

#[allow(clippy::large_enum_variant)]
pub enum LivenessStorageData<T> {
    RecoveryData(RecoveryData<T>),
    LedgerRecoveryData(LedgerRecoveryData),
}

impl<T: Payload> LivenessStorageData<T> {
    pub fn expect_recovery_data(self, msg: &str) -> RecoveryData<T> {
        match self {
            LivenessStorageData::RecoveryData(data) => data,
            LivenessStorageData::LedgerRecoveryData(_) => panic!("{}", msg),
        }
    }
}

// Manager the components that shared across epoch and spawn per-epoch EventProcessor with
// epoch-specific input.
pub struct EpochManager<T> {
    author: Author,
    config: ConsensusConfig,
    time_service: Arc<ClockTimeService>,
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg<T>>>>,
    network_sender: ConsensusNetworkSender<T>,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    storage: Arc<dyn PersistentLivenessStorage<T>>,
    safety_rules_manager: SafetyRulesManager<T>,
    processor: Option<Processor<T>>,
    block_store: Arc<Mutex<Option<Arc<BlockStore<T>>>>>,
}

impl<T: Payload> EpochManager<T> {
    pub fn new(
        author: Author,
        config: ConsensusConfig,
        time_service: Arc<ClockTimeService>,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg<T>>>>,
        network_sender: ConsensusNetworkSender<T>,
        timeout_sender: channel::Sender<Round>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentLivenessStorage<T>>,
        safety_rules_manager: SafetyRulesManager<T>,
    ) -> Self {
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
            // TODO: this is for testing, remove
            block_store: Arc::new(Mutex::new(None)),
        }
    }

    fn epoch_info(&self) -> &EpochInfo {
        match self
            .processor
            .as_ref()
            .expect("EpochManager not started yet")
        {
            Processor::EventProcessor(p) => p.epoch_info(),
            Processor::SyncProcessor(p) => p.epoch_info(),
        }
    }

    fn epoch(&self) -> u64 {
        self.epoch_info().epoch
    }

    fn create_pacemaker(
        &self,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
    ) -> Pacemaker {
        // 1.5^6 ~= 11
        // Timeout goes from initial_timeout to initial_timeout*11 in 6 steps
        let time_interval = Box::new(ExponentialTimeInterval::new(
            Duration::from_millis(self.config.pacemaker_initial_timeout_ms),
            1.5,
            6,
        ));
        Pacemaker::new(time_interval, time_service, timeout_sender)
    }

    /// Create a proposer election handler based on proposers
    fn create_proposer_election(
        &self,
        epoch_info: &EpochInfo,
    ) -> Box<dyn ProposerElection<T> + Send + Sync> {
        let proposers = epoch_info
            .verifier
            .get_ordered_account_addresses_iter()
            .collect::<Vec<_>>();
        match self.config.proposer_type {
            ConsensusProposerType::MultipleOrderedProposers => {
                Box::new(MultiProposer::new(epoch_info.epoch, proposers, 2))
            }
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
        }
    }

    async fn process_epoch_retrieval(
        &mut self,
        request: EpochRetrievalRequest,
        peer_id: AccountAddress,
    ) {
        let proof = match self
            .state_computer
            .get_epoch_proof(request.start_epoch, request.end_epoch)
            .await
        {
            Ok(proof) => proof,
            Err(e) => {
                warn!("Failed to get epoch proof from storage: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg::ValidatorChangeProof::<T>(Box::new(proof));
        if let Err(e) = self.network_sender.send_to(peer_id, msg) {
            warn!(
                "Failed to send a epoch retrieval to peer {}: {:?}",
                peer_id, e
            );
        };
    }

    async fn process_different_epoch(&mut self, different_epoch: u64, peer_id: AccountAddress) {
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
                let msg = ConsensusMsg::EpochRetrievalRequest::<T>(Box::new(request));
                if let Err(e) = self.network_sender.send_to(peer_id, msg) {
                    warn!(
                        "Failed to send a epoch retrieval to peer {}: {:?}",
                        peer_id, e
                    );
                }
            }
            Ordering::Equal => {
                warn!("Same epoch should not come to process_different_epoch");
            }
        }
    }

    async fn start_new_epoch(&mut self, proof: ValidatorChangeProof) {
        let verifier = VerifierType::TrustedVerifier(self.epoch_info().clone());
        let ledger_info = match proof.verify(&verifier) {
            Ok(ledger_info) => ledger_info,
            Err(e) => {
                error!("Invalid ValidatorChangeProof: {:?}", e);
                return;
            }
        };
        debug!(
            "Received epoch change to {}",
            ledger_info.ledger_info().epoch() + 1
        );

        // make sure storage is on this ledger_info too, it should be no-op if it's already committed
        if let Err(e) = self.state_computer.sync_to(ledger_info.clone()).await {
            error!("State sync to new epoch {} failed with {:?}, we'll try to start from current libradb", ledger_info, e);
        }
        // state_computer notifies reconfiguration in another channel
    }

    async fn start_event_processor(
        &mut self,
        recovery_data: RecoveryData<T>,
        epoch_info: EpochInfo,
    ) {
        // Release the previous EventProcessor, especially the SafetyRule client
        self.processor = None;
        counters::EPOCH.set(epoch_info.epoch as i64);
        counters::CURRENT_EPOCH_VALIDATORS.set(epoch_info.verifier.len() as i64);
        counters::CURRENT_EPOCH_QUORUM_SIZE.set(epoch_info.verifier.quorum_voting_power() as i64);
        info!(
            "Starting {} with genesis {}",
            epoch_info,
            recovery_data.root_block(),
        );
        info!("Update Network about new validators");
        self.network_sender
            .update_eligible_nodes(recovery_data.validator_keys())
            .await
            .expect("Unable to update network's eligible peers");
        let last_vote = recovery_data.last_vote();

        info!("Create BlockStore");
        let block_store = Arc::new(BlockStore::new(
            Arc::clone(&self.storage),
            recovery_data,
            Arc::clone(&self.state_computer),
            self.config.max_pruned_blocks_in_mem,
        ));
        *self.block_store.lock().unwrap() = Some(block_store.clone());

        info!("Update SafetyRules");

        let mut safety_rules = self.safety_rules_manager.client();
        safety_rules
            .start_new_epoch(block_store.highest_quorum_cert().as_ref())
            .expect("Unable to transition SafetyRules to the new epoch");

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

        info!("Create Pacemaker");
        let pacemaker =
            self.create_pacemaker(self.time_service.clone(), self.timeout_sender.clone());

        info!("Create ProposerElection");
        let proposer_election = self.create_proposer_election(&epoch_info);
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            epoch_info.verifier.clone(),
        );

        let mut processor = EventProcessor::new(
            epoch_info,
            block_store,
            last_vote,
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules,
            network_sender,
            self.txn_manager.clone(),
            self.storage.clone(),
            self.time_service.clone(),
        );
        processor.start().await;
        self.processor = Some(Processor::EventProcessor(processor));
        info!("EventProcessor started");
    }

    // Depending on what data we can extract from consensusdb, we may or may not have an
    // event processor at startup. If we need to sync up with peers for blocks to construct
    // a valid block store, which is required to construct an event processor, we will take
    // care of the sync up here.
    async fn start_sync_processor(
        &mut self,
        ledger_recovery_data: LedgerRecoveryData,
        epoch_info: EpochInfo,
    ) {
        self.network_sender
            .update_eligible_nodes(ledger_recovery_data.validator_keys())
            .await
            .expect("Unable to update network's eligible peers");
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            epoch_info.verifier.clone(),
        );
        self.processor = Some(Processor::SyncProcessor(SyncProcessor::new(
            epoch_info,
            network_sender,
            self.storage.clone(),
            self.state_computer.clone(),
            ledger_recovery_data,
        )));
        info!("SyncProcessor started");
    }

    pub async fn start_processor(&mut self, epoch_info: EpochInfo) {
        match self.storage.start() {
            LivenessStorageData::RecoveryData(initial_data) => {
                self.start_event_processor(initial_data, epoch_info).await
            }
            LivenessStorageData::LedgerRecoveryData(ledger_recovery_data) => {
                self.start_sync_processor(ledger_recovery_data, epoch_info)
                    .await
            }
        }
    }

    pub async fn process_message(
        &mut self,
        peer_id: AccountAddress,
        consensus_msg: ConsensusMsg<T>,
    ) {
        if let Some(event) = self.process_epoch(peer_id, consensus_msg).await {
            match event.verify(&self.epoch_info().verifier) {
                Ok(event) => self.process_event(peer_id, event).await,
                Err(err) => warn!("Message failed verification: {:?}", err),
            }
        }
    }

    async fn process_epoch(
        &mut self,
        peer_id: AccountAddress,
        msg: ConsensusMsg<T>,
    ) -> Option<UnverifiedEvent<T>> {
        match msg {
            ConsensusMsg::ProposalMsg(_) | ConsensusMsg::SyncInfo(_) | ConsensusMsg::VoteMsg(_) => {
                let event: UnverifiedEvent<T> = msg.into();
                if event.epoch() == self.epoch() {
                    return Some(event);
                } else {
                    self.process_different_epoch(event.epoch(), peer_id).await;
                }
            }
            ConsensusMsg::ValidatorChangeProof(proof) => {
                let msg_epoch = proof.epoch().map_err(|e| warn!("{:?}", e)).ok()?;
                if msg_epoch == self.epoch() {
                    self.start_new_epoch(*proof).await
                } else {
                    self.process_different_epoch(msg_epoch, peer_id).await
                }
            }
            ConsensusMsg::EpochRetrievalRequest(request) => {
                if request.end_epoch <= self.epoch() {
                    self.process_epoch_retrieval(*request, peer_id).await
                } else {
                    warn!("Received EpochRetrievalRequest beyond what we have locally");
                }
            }
            _ => {
                warn!("Unexpected messages: {:?}", msg);
            }
        }
        None
    }

    async fn process_event(&mut self, peer_id: AccountAddress, event: VerifiedEvent<T>) {
        match self.processor_mut() {
            Processor::SyncProcessor(p) => {
                let result = match event {
                    VerifiedEvent::ProposalMsg(proposal) => p.process_proposal_msg(*proposal).await,
                    VerifiedEvent::VoteMsg(vote) => p.process_vote(*vote).await,
                    _ => Err(anyhow!("Unexpected VerifiedEvent during startup")),
                };
                let epoch_info = p.epoch_info().clone();
                match result {
                    Ok(data) => {
                        info!("Recovered from SyncProcessor");
                        self.start_event_processor(data, epoch_info).await
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
            Processor::EventProcessor(p) => match event {
                VerifiedEvent::ProposalMsg(proposal) => p.process_proposal_msg(*proposal).await,
                VerifiedEvent::VoteMsg(vote) => p.process_vote(*vote).await,
                VerifiedEvent::SyncInfo(sync_info) => {
                    p.process_sync_info_msg(*sync_info, peer_id).await
                }
            },
        }
    }

    fn processor_mut(&mut self) -> &mut Processor<T> {
        self.processor
            .as_mut()
            .expect("EpochManager not started yet")
    }

    pub async fn process_block_retrieval(&mut self, request: IncomingBlockRetrievalRequest) {
        match self.processor_mut() {
            Processor::EventProcessor(p) => p.process_block_retrieval(request).await,
            _ => warn!("EventProcessor not started yet"),
        }
    }

    pub async fn process_local_timeout(&mut self, round: u64) {
        match self.processor_mut() {
            Processor::EventProcessor(p) => p.process_local_timeout(round).await,
            _ => unreachable!("EventProcessor not started yet"),
        }
    }

    pub fn block_store(&self) -> Arc<Mutex<Option<Arc<BlockStore<T>>>>> {
        self.block_store.clone()
    }
}
