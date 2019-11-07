use crate::pow::event_processor::EventProcessor;
use crate::{
    consensus_provider::ConsensusProvider, state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use config::config::NodeConfig;
use executor::Executor;
use failure::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use libra_types::account_address::AccountAddress;
use logger::prelude::*;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use state_synchronizer::StateSyncClient;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::runtime::{self, TaskExecutor};
use vm_runtime::MoveVM;

pub struct PowConsensusProvider {
    runtime: tokio::runtime::Runtime,
    event_handle: Option<EventProcessor>,
}

impl PowConsensusProvider {
    pub fn new(
        node_config: &mut NodeConfig,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        mempool_client: Arc<MempoolClient>,
        executor: Arc<Executor<MoveVM>>,
        synchronizer_client: Arc<StateSyncClient>,
        rollback_flag: bool,
    ) -> Self {
        let runtime = runtime::Builder::new()
            .name_prefix("pow-consensus-")
            .build()
            .expect("Failed to create Tokio runtime!");

        let txn_manager = Arc::new(MempoolProxy::new(mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client.clone()));

        let peer_id_str = node_config
            .get_validator_network_config()
            .unwrap()
            .peer_id
            .clone();
        let author = AccountAddress::try_from(peer_id_str.clone())
            .expect("Failed to parse peer id of a validator");

        let genesis_transaction = node_config
            .get_genesis_transaction()
            .expect("failed to load genesis transaction!");

        let event_handle = EventProcessor::new(
            network_sender,
            network_events,
            txn_manager,
            state_computer,
            author,
            node_config.get_storage_dir(),
            genesis_transaction,
            rollback_flag,
        );
        Self {
            runtime,
            event_handle: Some(event_handle),
        }
    }

    pub fn event_handle(&mut self, executor: TaskExecutor) {
        match self.event_handle.take() {
            Some(mut handle) => {
                //mint
                handle.mint_manager.mint(executor.clone());

                //msg
                handle.event_process(executor.clone());

                //save
                handle
                    .chain_manager
                    .borrow_mut()
                    .save_block(executor.clone());

                //sync
                handle
                    .sync_manager
                    .borrow_mut()
                    .sync_block_msg(executor.clone());

                //TODO:orphan
            }
            _ => {}
        }
    }
}

impl ConsensusProvider for PowConsensusProvider {
    fn start(&mut self) -> Result<()> {
        let executor = self.runtime.executor();
        self.event_handle(executor);
        info!("PowConsensusProvider start succ.");
        Ok(())
    }

    fn stop(&mut self) {
        //TODO
        // 1. stop mint
        // 2. stop process event
        unimplemented!()
    }
}
