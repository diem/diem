// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        event_processor::EventProcessor,
        liveness::{
            multi_proposer_election::MultiProposer,
            pacemaker::{ExponentialTimeInterval, Pacemaker},
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::{choose_leader, RotatingProposer},
        },
        network::NetworkSender,
        persistent_liveness_storage::{PersistentLivenessStorage, RecoveryData},
    },
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::{ClockTimeService, TimeService},
};
use consensus_types::{
    common::{Author, Payload, Round},
    epoch_retrieval::EpochRetrievalRequest,
};
use futures::executor::block_on;
use libra_config::config::{ConsensusConfig, ConsensusProposerType};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{EpochInfo, LedgerInfoWithSignatures, ValidatorVerifier},
};
use network::{
    proto::{ConsensusMsg, ConsensusMsg_oneof},
    validator_network::{ConsensusNetworkSender, Event},
};
use safety_rules::SafetyRulesManager;
use std::{
    cmp::Ordering,
    convert::TryInto,
    sync::{Arc, RwLock},
    time::Duration,
};

// Manager the components that shared across epoch and spawn per-epoch EventProcessor with
// epoch-specific input.
pub struct EpochManager<T> {
    author: Author,
    epoch_info: Arc<RwLock<EpochInfo>>,
    config: ConsensusConfig,
    time_service: Arc<ClockTimeService>,
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    storage: Arc<dyn PersistentLivenessStorage<T>>,
    safety_rules_manager: SafetyRulesManager<T>,
}

impl<T: Payload> EpochManager<T> {
    pub fn new(
        author: Author,
        epoch_info: Arc<RwLock<EpochInfo>>,
        config: ConsensusConfig,
        time_service: Arc<ClockTimeService>,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
        network_sender: ConsensusNetworkSender,
        timeout_sender: channel::Sender<Round>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentLivenessStorage<T>>,
        safety_rules_manager: SafetyRulesManager<T>,
    ) -> Self {
        Self {
            author,
            epoch_info,
            config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage,
            safety_rules_manager,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch_info.read().unwrap().epoch
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
        epoch: u64,
        validators: &ValidatorVerifier,
    ) -> Box<dyn ProposerElection<T> + Send + Sync> {
        let proposers = validators
            .get_ordered_account_addresses_iter()
            .collect::<Vec<_>>();
        match self.config.proposer_type {
            ConsensusProposerType::MultipleOrderedProposers => {
                Box::new(MultiProposer::new(epoch, proposers, 2))
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
        }
    }

    pub async fn process_epoch_retrieval(
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
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::EpochChange(proof.into())),
        };
        if let Err(e) = self.network_sender.send_to(peer_id, msg) {
            warn!(
                "Failed to send a epoch retrieval to peer {}: {:?}",
                peer_id, e
            );
        };
    }

    pub async fn process_different_epoch(&mut self, different_epoch: u64, peer_id: AccountAddress) {
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
                let msg = match request.try_into() {
                    Ok(bytes) => ConsensusMsg {
                        message: Some(ConsensusMsg_oneof::RequestEpoch(bytes)),
                    },
                    Err(e) => {
                        warn!("Fail to serialize EpochRetrievalRequest: {:?}", e);
                        return;
                    }
                };
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

    pub async fn start_new_epoch(
        &mut self,
        ledger_info: LedgerInfoWithSignatures,
    ) -> EventProcessor<T> {
        // make sure storage is on this ledger_info too, it should be no-op if it's already committed
        if let Err(e) = self.state_computer.sync_to(ledger_info.clone()).await {
            error!("State sync to new epoch {} failed with {:?}, we'll try to start from current libradb", ledger_info, e);
        }
        let initial_data = self.storage.start().await;
        *self.epoch_info.write().unwrap() = EpochInfo {
            epoch: initial_data.epoch(),
            verifier: initial_data.validators(),
        };
        self.start_epoch(initial_data)
    }

    pub fn start_epoch(&mut self, initial_data: RecoveryData<T>) -> EventProcessor<T> {
        let validators = initial_data.validators();
        let epoch = self.epoch();
        counters::EPOCH.set(epoch as i64);
        counters::CURRENT_EPOCH_VALIDATORS.set(validators.len() as i64);
        counters::CURRENT_EPOCH_QUORUM_SIZE.set(validators.quorum_voting_power() as i64);
        info!(
            "Start EventProcessor with epoch {} with genesis {}, validators {}",
            epoch,
            initial_data.root_block(),
            validators,
        );
        block_on(
            self.network_sender
                .update_eligible_nodes(initial_data.validator_keys()),
        )
        .expect("Unable to update network's eligible peers");
        let last_vote = initial_data.last_vote();

        let block_store = Arc::new(BlockStore::new(
            Arc::clone(&self.storage),
            initial_data,
            Arc::clone(&self.state_computer),
            self.config.max_pruned_blocks_in_mem,
        ));

        let mut safety_rules = self.safety_rules_manager.client();
        safety_rules
            .start_new_epoch(block_store.highest_quorum_cert().as_ref())
            .expect("Unable to transition SafetyRules to the new epoch");

        // txn manager is required both by proposal generator (to pull the proposers)
        // and by event processor (to update their status).
        let proposal_generator = ProposalGenerator::new(
            self.author,
            block_store.clone(),
            self.txn_manager.clone(),
            self.time_service.clone(),
            self.config.max_block_size,
        );

        let pacemaker =
            self.create_pacemaker(self.time_service.clone(), self.timeout_sender.clone());

        let proposer_election = self.create_proposer_election(epoch, &validators);
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            validators.clone(),
        );

        EventProcessor::new(
            block_store,
            last_vote,
            pacemaker,
            proposer_election,
            proposal_generator,
            safety_rules,
            self.txn_manager.clone(),
            network_sender,
            self.storage.clone(),
            self.time_service.clone(),
            validators,
        )
    }
}
