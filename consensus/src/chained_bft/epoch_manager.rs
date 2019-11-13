// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::BlockStore;
use crate::chained_bft::chained_bft_smr::ChainedBftSMRConfig;
use crate::chained_bft::event_processor::EventProcessor;
use crate::chained_bft::liveness::multi_proposer_election::MultiProposer;
use crate::chained_bft::liveness::pacemaker::{ExponentialTimeInterval, Pacemaker};
use crate::chained_bft::liveness::proposal_generator::ProposalGenerator;
use crate::chained_bft::liveness::proposer_election::ProposerElection;
use crate::chained_bft::liveness::rotating_proposer_election::{choose_leader, RotatingProposer};
use crate::chained_bft::network::NetworkSender;
use crate::chained_bft::persistent_storage::{PersistentStorage, RecoveryData};
use crate::counters;
use crate::state_replication::{StateComputer, TxnManager};
use crate::util::time_service::{ClockTimeService, TimeService};
use consensus_types::common::{Payload, Round};
use consensus_types::epoch_retrieval::EpochRetrievalRequest;
use futures::executor::block_on;
use libra_config::config::{ConsensusProposerType, SafetyRulesBackend};
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner, ValidatorVerifier};
use network::proto::ConsensusMsg;
use network::proto::ConsensusMsg_oneof;
use network::validator_network::{ConsensusNetworkSender, Event};
use safety_rules::{InMemoryStorage, OnDiskStorage, SafetyRules};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::sync::Arc;

// Manager the components that shared across epoch and spawn per-epoch EventProcessor with
// epoch-specific input.
pub struct EpochManager<T> {
    epoch: u64,
    config: ChainedBftSMRConfig,
    time_service: Arc<ClockTimeService>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Arc<dyn TxnManager<Payload = T>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    storage: Arc<dyn PersistentStorage<T>>,
    // TODO: remove once we have separate key management structure, and we'll share a slim client
    // across epoch
    signer: Arc<ValidatorSigner>,
}

impl<T: Payload> EpochManager<T> {
    pub fn new(
        epoch: u64,
        config: ChainedBftSMRConfig,
        time_service: Arc<ClockTimeService>,
        self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
        network_sender: ConsensusNetworkSender,
        timeout_sender: channel::Sender<Round>,
        txn_manager: Arc<dyn TxnManager<Payload = T>>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentStorage<T>>,
        signer: Arc<ValidatorSigner>,
    ) -> Self {
        Self {
            epoch,
            config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage,
            signer,
        }
    }

    fn create_pacemaker(
        &self,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
    ) -> Pacemaker {
        // 1.5^6 ~= 11
        // Timeout goes from initial_timeout to initial_timeout*11 in 6 steps
        let time_interval = Box::new(ExponentialTimeInterval::new(
            self.config.pacemaker_initial_timeout,
            1.5,
            6,
        ));
        Pacemaker::new(time_interval, time_service, timeout_sender)
    }

    /// Create a proposer election handler based on proposers
    fn create_proposer_election(
        &self,
        validators: &ValidatorVerifier,
    ) -> Box<dyn ProposerElection<T> + Send + Sync> {
        let proposers = validators.get_ordered_account_addresses();
        match self.config.proposer_type {
            ConsensusProposerType::MultipleOrderedProposers => {
                Box::new(MultiProposer::new(proposers, 2))
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

    pub async fn process_epoch_retrieval(&mut self, start_epoch: u64, peer_id: AccountAddress) {
        let proof = match self.state_computer.get_epoch_proof(start_epoch).await {
            Ok(proof) => proof,
            Err(e) => {
                warn!("Failed to get epoch proof from storage: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::EpochChange(proof.into())),
        };
        if let Err(e) = self.network_sender.send_to(peer_id, msg).await {
            warn!(
                "Failed to send a epoch retrieval to peer {}: {:?}",
                peer_id, e
            );
        };
    }

    pub async fn process_different_epoch(&mut self, different_epoch: u64, peer_id: AccountAddress) {
        match different_epoch.cmp(&self.epoch) {
            // We try to help nodes that have lower epoch than us
            Ordering::Less => self.process_epoch_retrieval(different_epoch, peer_id).await,
            // We request proof to join higher epoch
            Ordering::Greater => {
                let request = EpochRetrievalRequest {
                    start_epoch: self.epoch,
                    target_epoch: different_epoch,
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
                if let Err(e) = self.network_sender.send_to(peer_id, msg).await {
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

    pub fn start_new_epoch(&mut self, ledger_info: LedgerInfoWithSignatures) -> EventProcessor<T> {
        let validators = ledger_info
            .ledger_info()
            .next_validator_set()
            .expect("should have ValidatorSet when start new epoch")
            .into();
        // make sure storage is on this ledger_info too, it should be no-op if it's already committed
        self.state_computer.sync_to_or_bail(ledger_info.clone());
        let initial_data = RecoveryData::new(None, vec![], vec![], ledger_info.ledger_info(), None)
            .expect("should be able to build new epoch RecoveryData");
        self.epoch = initial_data.epoch();
        info!(
            "Start new epoch {} with genesis {}, validators {}",
            self.epoch,
            initial_data.root_block(),
            validators,
        );
        self.start_epoch(self.signer.clone(), Arc::new(validators), initial_data)
    }

    pub fn start_epoch(
        &self,
        signer: Arc<ValidatorSigner>,
        validators: Arc<ValidatorVerifier>,
        initial_data: RecoveryData<T>,
    ) -> EventProcessor<T> {
        counters::EPOCH.set(self.epoch as i64);
        let last_vote = initial_data.last_vote();
        let author = signer.author();
        let safety_rules_storage = match &self.config.safety_rules.backend {
            SafetyRulesBackend::InMemoryStorage => InMemoryStorage::default_storage(),
            SafetyRulesBackend::OnDiskStorage { default, path } => {
                if *default {
                    OnDiskStorage::default_storage(path.clone())
                } else {
                    OnDiskStorage::new_storage(path.clone())
                }
            }
        };
        let safety_rules = SafetyRules::new(safety_rules_storage, signer);

        let block_store = Arc::new(block_on(BlockStore::new(
            Arc::clone(&self.storage),
            initial_data,
            Arc::clone(&self.state_computer),
            self.config.max_pruned_blocks_in_mem,
        )));

        // txn manager is required both by proposal generator (to pull the proposers)
        // and by event processor (to update their status).
        let proposal_generator = ProposalGenerator::new(
            author,
            block_store.clone(),
            Arc::clone(&self.txn_manager),
            self.time_service.clone(),
            self.config.max_block_size,
        );

        let pacemaker =
            self.create_pacemaker(self.time_service.clone(), self.timeout_sender.clone());

        let proposer_election = self.create_proposer_election(&validators);
        let network_sender = NetworkSender::new(
            author,
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
