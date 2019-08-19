// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        common::{Payload, Round},
        event_processor::EventProcessor,
        liveness::{
            pacemaker::{ExponentialTimeInterval, Pacemaker},
            pacemaker_timeout_manager::HighestTimeoutCertificates,
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::RotatingProposer,
        },
        network::{ConsensusNetworkImpl, NetworkReceivers},
        persistent_storage::{PersistentLivenessStorage, PersistentStorage, RecoveryData},
        safety::safety_rules::SafetyRules,
    },
    counters,
    state_replication::{StateComputer, StateMachineReplication, TxnManager},
    util::time_service::{ClockTimeService, TimeService},
};
use channel;
use crypto::ed25519::*;
use failure::prelude::*;
use futures::{
    compat::Future01CompatExt,
    executor::block_on,
    future::{FutureExt, TryFutureExt},
    select,
    stream::StreamExt,
};
use types::validator_signer::ValidatorSigner;

use crate::chained_bft::common::Author;
use config::config::ConsensusConfig;
use logger::prelude::*;
use std::{sync::Arc, time::Duration};
use tokio::runtime::{Runtime, TaskExecutor};

/// Consensus configuration derived from ConsensusConfig
pub struct ChainedBftSMRConfig {
    /// Keep up to this number of committed blocks before cleaning them up from the block store.
    pub max_pruned_blocks_in_mem: usize,
    /// Initial timeout for pacemaker
    pub pacemaker_initial_timeout: Duration,
    /// Contiguous rounds for proposer
    pub contiguous_rounds: u32,
    /// Max block size (number of transactions) that consensus pulls from mempool
    pub max_block_size: u64,
}

impl ChainedBftSMRConfig {
    pub fn from_node_config(cfg: &ConsensusConfig) -> ChainedBftSMRConfig {
        let pacemaker_initial_timeout_ms = cfg.pacemaker_initial_timeout_ms().unwrap_or(1000);
        ChainedBftSMRConfig {
            max_pruned_blocks_in_mem: cfg.max_pruned_blocks_in_mem().unwrap_or(10000) as usize,
            pacemaker_initial_timeout: Duration::from_millis(pacemaker_initial_timeout_ms),
            contiguous_rounds: cfg.contiguous_rounds(),
            max_block_size: cfg.max_block_size(),
        }
    }
}

/// ChainedBFTSMR is the one to generate the components (BlockStore, Proposer, etc.) and start the
/// driver. ChainedBftSMR implements the StateMachineReplication, it is going to be used by
/// ConsensusProvider for the e2e flow.
pub struct ChainedBftSMR<T> {
    author: Author,
    // TODO [Reconfiguration] quorum size is just a function of current validator set.
    quorum_size: usize,
    signer: Option<ValidatorSigner<Ed25519PrivateKey>>,
    proposers: Vec<Author>,
    runtime: Option<Runtime>,
    block_store: Option<Arc<BlockStore<T>>>,
    network: ConsensusNetworkImpl,
    config: ChainedBftSMRConfig,
    storage: Arc<dyn PersistentStorage<T>>,
    initial_data: Option<RecoveryData<T>>,
}

impl<T: Payload> ChainedBftSMR<T> {
    pub fn new(
        author: Author,
        quorum_size: usize,
        signer: ValidatorSigner<Ed25519PrivateKey>,
        proposers: Vec<Author>,
        network: ConsensusNetworkImpl,
        runtime: Runtime,
        config: ChainedBftSMRConfig,
        storage: Arc<dyn PersistentStorage<T>>,
        initial_data: RecoveryData<T>,
    ) -> Self {
        Self {
            author,
            quorum_size,
            signer: Some(signer),
            proposers,
            runtime: Some(runtime),
            block_store: None,
            network,
            config,
            storage,
            initial_data: Some(initial_data),
        }
    }

    #[cfg(test)]
    pub fn block_store(&self) -> Option<Arc<BlockStore<T>>> {
        self.block_store.clone()
    }

    fn create_pacemaker(
        &self,
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
        highest_timeout_certificate: HighestTimeoutCertificates,
    ) -> Pacemaker {
        // 1.5^6 ~= 11
        // Timeout goes from initial_timeout to initial_timeout*11 in 6 steps
        let time_interval = Box::new(ExponentialTimeInterval::new(
            self.config.pacemaker_initial_timeout,
            1.5,
            6,
        ));
        Pacemaker::new(
            persistent_liveness_storage,
            time_interval,
            time_service,
            timeout_sender,
            self.quorum_size,
            highest_timeout_certificate,
        )
    }

    /// Create a proposer election handler based on proposers
    fn create_proposer_election(&self) -> Arc<dyn ProposerElection<T> + Send + Sync> {
        assert!(!self.proposers.is_empty());
        Arc::new(RotatingProposer::new(
            self.proposers.clone(),
            self.config.contiguous_rounds,
        ))
    }

    fn start_event_processing(
        &self,
        executor: TaskExecutor,
        mut event_processor: EventProcessor<T>,
        mut network_receivers: NetworkReceivers<T>,
        mut pacemaker_timeout_sender_rx: channel::Receiver<Round>,
    ) {
        let quorum_size = self.quorum_size;
        let fut = async move {
            event_processor.start().await;
            loop {
                select! {
                    proposal_msg = network_receivers.proposals.select_next_some() => {
                        event_processor.process_proposal_msg(proposal_msg).await;
                    }
                    block_retrieval = network_receivers.block_retrieval.select_next_some() => {
                        event_processor.process_block_retrieval(block_retrieval).await;
                    }
                    vote_msg = network_receivers.votes.select_next_some() => {
                        event_processor.process_vote(vote_msg, quorum_size).await;
                    }
                    remote_timeout_msg = network_receivers.timeout_msgs.select_next_some() => {
                        event_processor.process_remote_timeout_msg(remote_timeout_msg, quorum_size).await;
                    }
                    local_timeout_round = pacemaker_timeout_sender_rx.select_next_some() => {
                        event_processor.process_local_timeout(local_timeout_round).await;
                    }
                    sync_info_msg = network_receivers.sync_info_msgs.select_next_some() => {
                        event_processor.process_sync_info_msg(sync_info_msg.0, sync_info_msg.1).await;
                    }
                    complete => {
                        break;
                    }
                }
            }
        };
        executor.spawn(fut.boxed().unit_error().compat());
    }
}

impl<T: Payload> StateMachineReplication for ChainedBftSMR<T> {
    type Payload = T;

    fn start(
        &mut self,
        txn_manager: Arc<dyn TxnManager<Payload = Self::Payload>>,
        state_computer: Arc<dyn StateComputer<Payload = Self::Payload>>,
    ) -> Result<()> {
        let executor = self
            .runtime
            .as_mut()
            .expect("Consensus start: No valid runtime found!")
            .executor();
        let time_service = Arc::new(ClockTimeService::new(executor.clone()));

        // We first start the network and retrieve the network receivers (this function needs a
        // mutable reference).
        // Must do it here before giving the clones of network to other components.
        let network_receivers = self.network.start(&executor);
        let initial_data = self
            .initial_data
            .take()
            .expect("already started, initial data is None");
        let consensus_state = initial_data.state();
        let highest_timeout_certificates = initial_data.highest_timeout_certificates().clone();
        if initial_data.need_sync() {
            loop {
                // make sure we sync to the root state in case we're not
                let status = block_on(state_computer.sync_to(initial_data.root_ledger_info()));
                match status {
                    Ok(true) => break,
                    Ok(false) => panic!(
                    "state synchronizer failure, this validator will be killed as it can not \
                 recover from this error.  After the validator is restarted, synchronization will \
                 be retried.",
                ),
                    Err(e) => panic!(
                    "state synchronizer failure: {:?}, this validator will be killed as it can not \
                 recover from this error.  After the validator is restarted, synchronization will \
                 be retried.",
                    e
                ),
                }
            }
        }

        // the signer is only stored in the SMR to be provided here
        let opt_signer = std::mem::replace(&mut self.signer, None);
        if let Some(signer) = opt_signer {
            let block_store = Arc::new(block_on(BlockStore::new(
                Arc::clone(&self.storage),
                initial_data,
                signer,
                Arc::clone(&state_computer),
                true,
                self.config.max_pruned_blocks_in_mem,
            )));

            self.block_store = Some(Arc::clone(&block_store));

            // txn manager is required both by proposal generator (to pull the proposers)
            // and by event processor (to update their status).
            let proposal_generator = ProposalGenerator::new(
                block_store.clone(),
                Arc::clone(&txn_manager),
                time_service.clone(),
                self.config.max_block_size,
                true,
            );

            let safety_rules = SafetyRules::new(consensus_state);

            let (timeout_sender, timeout_receiver) =
                channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
            let pacemaker = self.create_pacemaker(
                self.storage.persistent_liveness_storage(),
                time_service.clone(),
                timeout_sender,
                highest_timeout_certificates,
            );

            let proposer_election = self.create_proposer_election();
            let event_processor = EventProcessor::new(
                self.author,
                Arc::clone(&block_store),
                pacemaker,
                Arc::clone(&proposer_election),
                proposal_generator,
                safety_rules,
                state_computer,
                txn_manager,
                self.network.clone(),
                Arc::clone(&self.storage),
                time_service.clone(),
                true,
            );

            self.start_event_processing(
                executor,
                event_processor,
                network_receivers,
                timeout_receiver,
            );
        } else {
            panic!("start called twice on the same Chained BFT SMR!");
        }

        debug!("Chained BFT SMR started.");
        Ok(())
    }

    /// Stop is synchronous: waits for all the worker threads to terminate.
    fn stop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            block_on(rt.shutdown_now().compat()).unwrap();
            debug!("Chained BFT SMR stopped.")
        }
    }
}
