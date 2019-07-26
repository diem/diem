// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        common::{Payload, Round},
        consensus_types::{
            proposal_info::{ProposalInfo, ProposerInfo},
            timeout_msg::TimeoutMsg,
        },
        event_processor::{EventProcessor, ProcessProposalResult},
        liveness::{
            local_pacemaker::{ExponentialTimeInterval, LocalPacemaker},
            pacemaker::{NewRoundEvent, Pacemaker},
            pacemaker_timeout_manager::HighestTimeoutCertificates,
            proposal_generator::ProposalGenerator,
            proposer_election::ProposerElection,
            rotating_proposer_election::RotatingProposer,
        },
        network::{
            BlockRetrievalRequest, ChunkRetrievalRequest, ConsensusNetworkImpl, NetworkReceivers,
        },
        persistent_storage::{PersistentLivenessStorage, PersistentStorage, RecoveryData},
        safety::{safety_rules::SafetyRules, vote_msg::VoteMsg},
    },
    counters,
    state_replication::{StateComputer, StateMachineReplication, TxnManager},
    state_synchronizer::SyncStatus,
    util::time_service::{ClockTimeService, TimeService},
};
use channel;
use failure::prelude::*;
use futures::{
    compat::Future01CompatExt,
    executor::block_on,
    future::{FutureExt, TryFutureExt},
    stream::StreamExt,
};
use nextgen_crypto::ed25519::*;
use types::validator_signer::ValidatorSigner;

use crate::chained_bft::consensus_types::sync_info::SyncInfo;
use config::config::ConsensusConfig;
use logger::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::{Runtime, TaskExecutor};

type ConcurrentEventProcessor<T, P> = Arc<futures_locks::RwLock<EventProcessor<T, P>>>;

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

/// ChainedBFTSmr is the one to generate the components (BLockStore, Proposer, etc.) and start the
/// driver. ChainedBftSMR implements the StateMachineReplication, it is going to be used by
/// ConsensusProvider for the e2e flow.
pub struct ChainedBftSMR<T, P> {
    author: P,
    // TODO [Reconfiguration] quorum size is just a function of current validator set.
    quorum_size: usize,
    signer: Option<ValidatorSigner<Ed25519PrivateKey>>,
    proposers: Vec<P>,
    runtime: Option<Runtime>,
    block_store: Option<Arc<BlockStore<T>>>,
    network: ConsensusNetworkImpl,
    config: ChainedBftSMRConfig,
    storage: Arc<dyn PersistentStorage<T>>,
    initial_data: Option<RecoveryData<T>>,
}

#[allow(dead_code)]
impl<T: Payload, P: ProposerInfo> ChainedBftSMR<T, P> {
    pub fn new(
        author: P,
        quorum_size: usize,
        signer: ValidatorSigner<Ed25519PrivateKey>,
        proposers: Vec<P>,
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

    pub fn block_store(&self) -> Option<Arc<BlockStore<T>>> {
        self.block_store.clone()
    }

    fn create_pacemaker(
        &self,
        executor: TaskExecutor,
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
        highest_committed_round: Round,
        highest_certified_round: Round,
        highest_timeout_certificates: HighestTimeoutCertificates,
        time_service: Arc<dyn TimeService>,
        new_round_events_sender: channel::Sender<NewRoundEvent>,
        external_timeout_sender: channel::Sender<Round>,
    ) -> Arc<dyn Pacemaker> {
        // 1.5^6 ~= 11
        // Timeout goes from initial_timeout to initial_timeout*11 in 6 steps
        let time_interval = Box::new(ExponentialTimeInterval::new(
            self.config.pacemaker_initial_timeout,
            1.5,
            6,
        ));
        Arc::new(LocalPacemaker::new(
            executor,
            persistent_liveness_storage,
            time_interval,
            highest_committed_round,
            highest_certified_round,
            time_service,
            new_round_events_sender,
            external_timeout_sender,
            self.quorum_size,
            highest_timeout_certificates,
        ))
    }

    /// Create a proposer election handler based on proposers
    fn create_proposer_election(
        &self,
        winning_proposals_sender: channel::Sender<ProposalInfo<T, P>>,
    ) -> Arc<dyn ProposerElection<T, P> + Send + Sync> {
        assert!(!self.proposers.is_empty());
        Arc::new(RotatingProposer::new(
            self.proposers.clone(),
            self.config.contiguous_rounds,
            winning_proposals_sender,
        ))
    }

    async fn process_new_round_events(
        mut receiver: channel::Receiver<NewRoundEvent>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(new_round_event) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            guard.process_new_round_event(new_round_event).await;
        }
    }

    async fn process_proposals(
        executor: TaskExecutor,
        mut receiver: channel::Receiver<ProposalInfo<T, P>>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(proposal_info) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            match guard.process_proposal(proposal_info).await {
                ProcessProposalResult::Done => (),
                // Spawn a new task that would start retrieving the missing
                // blocks in the background.
                ProcessProposalResult::NeedFetch(deadline, proposal) => executor.spawn(
                    Self::fetch_and_process_proposal(
                        Arc::clone(&event_processor),
                        deadline,
                        proposal,
                    )
                    .boxed()
                    .unit_error()
                    .compat(),
                ),
                // Spawn a new task that would start state synchronization
                // in the background.
                ProcessProposalResult::NeedSync(deadline, proposal) => executor.spawn(
                    Self::sync_and_process_proposal(
                        Arc::clone(&event_processor),
                        deadline,
                        proposal,
                    )
                    .boxed()
                    .unit_error()
                    .compat(),
                ),
            }
        }
    }

    async fn fetch_and_process_proposal(
        event_processor: ConcurrentEventProcessor<T, P>,
        deadline: Instant,
        proposal: ProposalInfo<T, P>,
    ) {
        let guard = event_processor.read().compat().await.unwrap();
        guard.fetch_and_process_proposal(deadline, proposal).await
    }

    async fn sync_and_process_proposal(
        event_processor: ConcurrentEventProcessor<T, P>,
        deadline: Instant,
        proposal: ProposalInfo<T, P>,
    ) {
        let mut guard = event_processor.write().compat().await.unwrap();
        guard.sync_and_process_proposal(deadline, proposal).await
    }

    async fn process_winning_proposals(
        mut receiver: channel::Receiver<ProposalInfo<T, P>>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(proposal_info) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            guard.process_winning_proposal(proposal_info).await;
        }
    }

    async fn process_votes(
        mut receiver: channel::Receiver<VoteMsg>,
        event_processor: ConcurrentEventProcessor<T, P>,
        quorum_size: usize,
    ) {
        while let Some(vote) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            guard.process_vote(vote, quorum_size).await;
        }
    }

    async fn process_timeout_msg(
        mut receiver: channel::Receiver<TimeoutMsg>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(timeout_msg) = receiver.next().await {
            let mut guard = event_processor.write().compat().await.unwrap();
            guard.process_timeout_msg(timeout_msg).await;
        }
    }

    async fn process_sync_info_msgs(
        mut receiver: channel::Receiver<SyncInfo>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(sync_info) = receiver.next().await {
            let mut guard = event_processor.write().compat().await.unwrap();
            guard.process_sync_info_msg(sync_info).await;
        }
    }

    async fn process_outgoing_pacemaker_timeouts(
        mut receiver: channel::Receiver<Round>,
        event_processor: ConcurrentEventProcessor<T, P>,
        mut network: ConsensusNetworkImpl,
    ) {
        while let Some(round) = receiver.next().await {
            // Update the last voted round and generate the timeout message
            let guard = event_processor.read().compat().await.unwrap();
            let timeout_msg = guard.process_outgoing_pacemaker_timeout(round).await;
            match timeout_msg {
                Some(timeout_msg) => {
                    network.broadcast_timeout_msg(timeout_msg).await;
                }
                None => {
                    info!("Broadcast not sent as the processing of the timeout failed.  Will retry again on the next timeout.");
                }
            }
        }
    }

    async fn process_block_retrievals(
        mut receiver: channel::Receiver<BlockRetrievalRequest<T>>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(request) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            guard.process_block_retrieval(request).await;
        }
    }

    async fn process_chunk_retrievals(
        mut receiver: channel::Receiver<ChunkRetrievalRequest>,
        event_processor: ConcurrentEventProcessor<T, P>,
    ) {
        while let Some(request) = receiver.next().await {
            let guard = event_processor.read().compat().await.unwrap();
            guard.process_chunk_retrieval(request).await;
        }
    }

    fn start_event_processing(
        &self,
        event_processor: ConcurrentEventProcessor<T, P>,
        executor: TaskExecutor,
        new_round_events_receiver: channel::Receiver<NewRoundEvent>,
        winning_proposals_receiver: channel::Receiver<ProposalInfo<T, P>>,
        network_receivers: NetworkReceivers<T, P>,
        pacemaker_timeout_sender_rx: channel::Receiver<Round>,
    ) {
        executor.spawn(
            Self::process_new_round_events(new_round_events_receiver, event_processor.clone())
                .boxed()
                .unit_error()
                .compat(),
        );

        executor.spawn(
            Self::process_proposals(
                executor.clone(),
                network_receivers.proposals,
                event_processor.clone(),
            )
            .boxed()
            .unit_error()
            .compat(),
        );

        executor.spawn(
            Self::process_winning_proposals(winning_proposals_receiver, event_processor.clone())
                .boxed()
                .unit_error()
                .compat(),
        );

        executor.spawn(
            Self::process_block_retrievals(
                network_receivers.block_retrieval,
                event_processor.clone(),
            )
            .boxed()
            .unit_error()
            .compat(),
        );

        executor.spawn(
            Self::process_chunk_retrievals(
                network_receivers.chunk_retrieval,
                event_processor.clone(),
            )
            .boxed()
            .unit_error()
            .compat(),
        );

        executor.spawn(
            Self::process_votes(
                network_receivers.votes,
                event_processor.clone(),
                self.quorum_size,
            )
            .boxed()
            .unit_error()
            .compat(),
        );

        executor.spawn(
            Self::process_timeout_msg(network_receivers.timeout_msgs, event_processor.clone())
                .boxed()
                .unit_error()
                .compat(),
        );

        executor.spawn(
            Self::process_outgoing_pacemaker_timeouts(
                pacemaker_timeout_sender_rx,
                event_processor.clone(),
                self.network.clone(),
            )
            .boxed()
            .unit_error()
            .compat(),
        );

        executor.spawn(
            Self::process_sync_info_msgs(network_receivers.sync_info_msgs, event_processor.clone())
                .boxed()
                .unit_error()
                .compat(),
        );
    }
}

impl<T: Payload, P: ProposerInfo> StateMachineReplication for ChainedBftSMR<T, P> {
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
                    Ok(SyncStatus::Finished) => break,
                    Ok(SyncStatus::DownloadFailed) => {
                        warn!("DownloadFailed, we may not establish connection with peers yet, sleep and retry");
                        // we can remove this when we start to handle NewPeer/LostPeer events.
                        thread::sleep(Duration::from_secs(2));
                    }
                    Ok(e) => panic!(
                    "state synchronizer failure: {:?}, this validator will be killed as it can not \
                 recover from this error.  After the validator is restarted, synchronization will \
                 be retried.",
                    e
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

            let safety_rules = Arc::new(RwLock::new(SafetyRules::new(
                block_store.clone(),
                consensus_state,
            )));

            let (external_timeout_sender, external_timeout_receiver) =
                channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
            let (new_round_events_sender, new_round_events_receiver) =
                channel::new(1_024, &counters::PENDING_NEW_ROUND_EVENTS);
            let pacemaker = self.create_pacemaker(
                executor.clone(),
                self.storage.persistent_liveness_storage(),
                safety_rules.read().unwrap().last_committed_round(),
                block_store.highest_certified_block().round(),
                highest_timeout_certificates,
                time_service.clone(),
                new_round_events_sender,
                external_timeout_sender,
            );

            let (winning_proposals_sender, winning_proposals_receiver) =
                channel::new(1_024, &counters::PENDING_WINNING_PROPOSALS);
            let proposer_election = self.create_proposer_election(winning_proposals_sender);
            let event_processor = Arc::new(futures_locks::RwLock::new(EventProcessor::new(
                self.author,
                Arc::clone(&block_store),
                Arc::clone(&pacemaker),
                Arc::clone(&proposer_election),
                proposal_generator,
                safety_rules,
                state_computer,
                txn_manager,
                self.network.clone(),
                Arc::clone(&self.storage),
                time_service.clone(),
                true,
            )));

            self.start_event_processing(
                event_processor,
                executor.clone(),
                new_round_events_receiver,
                winning_proposals_receiver,
                network_receivers,
                external_timeout_receiver,
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
