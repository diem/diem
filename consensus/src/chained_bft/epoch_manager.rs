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
use crate::state_replication::{StateComputer, TxnManager};
use crate::util::time_service::{ClockTimeService, TimeService};
use consensus_types::common::{Payload, Round};
use futures::executor::block_on;
use libra_config::config::ConsensusProposerType;
use libra_types::crypto_proxies::{ValidatorSigner, ValidatorVerifier};
use network::proto::ConsensusMsg;
use network::validator_network::{ConsensusNetworkSender, Event};
use safety_rules::SafetyRules;
use std::sync::Arc;

// Manager the components that shared across epoch and spawn per-epoch EventProcessor with
// epoch-specific input.
pub struct EpochManager<T> {
    #[allow(dead_code)]
    epoch: u64,
    config: ChainedBftSMRConfig,
    time_service: Arc<ClockTimeService>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Arc<dyn TxnManager<Payload = T>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    storage: Arc<dyn PersistentStorage<T>>,
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

    pub fn start_epoch(
        &self,
        signer: ValidatorSigner,
        validators: Arc<ValidatorVerifier>,
        initial_data: RecoveryData<T>,
    ) -> EventProcessor<T> {
        let last_vote = initial_data.last_vote();
        let author = signer.author();
        let safety_rules = SafetyRules::new(initial_data.state(), signer);

        let block_store = Arc::new(block_on(BlockStore::new(
            Arc::clone(&self.storage),
            initial_data,
            author,
            Arc::clone(&self.state_computer),
            true,
            self.config.max_pruned_blocks_in_mem,
        )));

        // txn manager is required both by proposal generator (to pull the proposers)
        // and by event processor (to update their status).
        let proposal_generator = ProposalGenerator::new(
            block_store.clone(),
            Arc::clone(&self.txn_manager),
            self.time_service.clone(),
            self.config.max_block_size,
            true,
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
            true,
            validators,
        )
    }
}
