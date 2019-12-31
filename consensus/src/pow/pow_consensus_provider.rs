use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::{
    block_storage_service::make_block_storage_service,
    event_processor::EventProcessor,
    mine_state::{BlockIndex, MineStateManager},
};
use crate::{
    consensus_provider::ConsensusProvider, state_computer::ExecutionProxy,
    txn_manager::MempoolProxy, MineClient,
};
use anyhow::Result;
use async_std::task;
use executor::Executor;
use grpcio::Server;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use libra_types::account_address::AccountAddress;
use miner::server::setup_minerproxy_service;
use network::validator_network::{
    ChainStateNetworkEvents, ChainStateNetworkSender, ConsensusNetworkEvents,
    ConsensusNetworkSender,
};
use state_synchronizer::StateSyncClient;
use std::convert::TryFrom;
use std::sync::Arc;
use storage_client::{StorageRead, StorageWrite};
use tokio::runtime::{self, Handle};
use vm_runtime::MoveVM;

pub struct PowConsensusProvider {
    runtime: tokio::runtime::Runtime,
    event_handle: Option<EventProcessor>,
    miner_proxy: Option<Server>,
    _block_storage_server: Server,
    chain_state_network_sender: Option<ChainStateNetworkSender>,
    chain_state_network_events: Option<ChainStateNetworkEvents>,
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
        read_storage: Arc<dyn StorageRead>,
        write_storage: Arc<dyn StorageWrite>,
        chain_state_network_sender: ChainStateNetworkSender,
        chain_state_network_events: ChainStateNetworkEvents,
    ) -> Self {
        let runtime = runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");

        let txn_manager = Arc::new(MempoolProxy::new(mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client.clone()));

        let peer_id_str = node_config
            .validator_network
            .as_ref()
            .unwrap()
            .peer_id
            .clone();
        let author = AccountAddress::try_from(peer_id_str.clone())
            .expect("Failed to parse peer id of a validator");
        // block store
        let block_store = Arc::new(ConsensusDB::new(&node_config.storage.dir()));

        //BlockStorageService
        let block_storage_server =
            make_block_storage_service(node_config, &Arc::clone(&block_store));

        //Start miner proxy server
        let mine_state = MineStateManager::new(BlockIndex::new(block_store.clone()));
        let miner_rpc_addr = String::from(&node_config.consensus.miner_rpc_address);
        let mut miner_proxy = setup_minerproxy_service(mine_state.clone(), miner_rpc_addr.clone());
        miner_proxy.start();
        for &(ref host, port) in miner_proxy.bind_addrs() {
            info!("listening on {}:{}", host, port);
        }
        // Start miner client.
        if node_config.consensus.miner_client_enable {
            task::spawn(async move {
                let mine_client = MineClient::new(miner_rpc_addr);
                mine_client.start().await
            });
        }

        let self_pri_key = node_config.consensus.take_and_set_key();
        let event_handle = EventProcessor::new(
            network_sender,
            network_events,
            txn_manager,
            state_computer,
            author,
            block_store,
            rollback_flag,
            mine_state,
            read_storage,
            write_storage,
            self_pri_key,
            node_config.storage.dir(),
        );
        //node_config.base().data_dir.clone()
        Self {
            runtime,
            event_handle: Some(event_handle),
            miner_proxy: Some(miner_proxy),
            _block_storage_server: block_storage_server,
            chain_state_network_sender: Some(chain_state_network_sender),
            chain_state_network_events: Some(chain_state_network_events),
        }
    }

    pub fn event_handle(
        &mut self,
        executor: Handle,
        chain_state_network_sender: ChainStateNetworkSender,
        chain_state_network_events: ChainStateNetworkEvents,
    ) {
        match self.event_handle.take() {
            Some(mut handle) => {
                let block_cache_receiver = handle
                    .block_cache_receiver
                    .take()
                    .expect("block_cache_receiver is none.");

                //mint
                handle.mint_manager.borrow_mut().mint(executor.clone());

                //msg
                handle.chain_state_handle(
                    executor.clone(),
                    chain_state_network_sender,
                    chain_state_network_events,
                );
                handle.event_process(executor.clone());

                //save
                handle
                    .chain_manager
                    .borrow_mut()
                    .save_block(block_cache_receiver, executor.clone());

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
        let executor = self.runtime.handle().clone();
        let chain_state_network_sender = self
            .chain_state_network_sender
            .take()
            .expect("chain_state_network_sender is none.");
        let chain_state_network_events = self
            .chain_state_network_events
            .take()
            .expect("chain_state_network_events is none.");
        self.event_handle(
            executor,
            chain_state_network_sender,
            chain_state_network_events,
        );
        info!("PowConsensusProvider start succ.");
        Ok(())
    }

    fn stop(&mut self) {
        //TODO
        // 1. stop mint
        // 2. stop process event
        // Stop Miner proxy
        if let Some(miner_proxy) = self.miner_proxy.take() {
            drop(miner_proxy);
        }
    }
}
