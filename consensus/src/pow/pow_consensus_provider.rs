use crate::chained_bft::consensusdb::ConsensusDB;
use crate::counters;
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
use consensus_types::block_retrieval::BlockRetrievalResponse;
use consensus_types::payload_ext::BlockPayloadExt;
use executor::Executor;
use futures::channel::mpsc;
use grpcio::Server;
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use libra_types::account_address::AccountAddress;
use libra_types::PeerId;
use miner::server::setup_minerproxy_service;
use network::proto::ConsensusMsg;
use network::validator_network::{
    ChainStateNetworkEvents, ChainStateNetworkSender, ConsensusNetworkEvents,
    ConsensusNetworkSender, Event,
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
    mint_key: Option<Ed25519PrivateKey>,
    event_handle_network_events: Option<ConsensusNetworkEvents>,
    event_handle_receiver: Option<channel::Receiver<Result<Event<ConsensusMsg>>>>,
    sync_block_receiver: Option<mpsc::Receiver<(PeerId, BlockRetrievalResponse<BlockPayloadExt>)>>,
    sync_signal_receiver: Option<mpsc::Receiver<(PeerId, (u64, HashValue))>>,
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
        let (event_handle_sender, event_handle_receiver) =
            channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(1024);
        let event_handle = EventProcessor::new(
            network_sender,
            txn_manager,
            state_computer,
            author,
            block_store,
            rollback_flag,
            mine_state,
            read_storage,
            write_storage,
            event_handle_sender,
            sync_block_sender,
            sync_signal_sender,
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
            mint_key: Some(self_pri_key),
            event_handle_network_events: Some(network_events),
            event_handle_receiver: Some(event_handle_receiver),
            sync_block_receiver: Some(sync_block_receiver),
            sync_signal_receiver: Some(sync_signal_receiver),
        }
    }

    pub fn event_handle(
        &mut self,
        executor: Handle,
        chain_state_network_sender: ChainStateNetworkSender,
        chain_state_network_events: ChainStateNetworkEvents,
        self_key: Ed25519PrivateKey,
        event_handle_network_events: ConsensusNetworkEvents,
        event_handle_receiver: channel::Receiver<Result<Event<ConsensusMsg>>>,
        sync_block_receiver: mpsc::Receiver<(PeerId, BlockRetrievalResponse<BlockPayloadExt>)>,
        sync_signal_receiver: mpsc::Receiver<(PeerId, (u64, HashValue))>,
    ) {
        match self.event_handle.take() {
            Some(mut handle) => {
                let block_cache_receiver = handle
                    .block_cache_receiver
                    .take()
                    .expect("block_cache_receiver is none.");

                //mint
                handle
                    .mint_manager
                    .borrow()
                    .mint(executor.clone(), self_key);

                //msg
                handle.chain_state_handle(
                    executor.clone(),
                    chain_state_network_sender,
                    chain_state_network_events,
                );
                handle.event_process(
                    executor.clone(),
                    event_handle_network_events,
                    event_handle_receiver,
                );

                //save
                handle
                    .chain_manager
                    .borrow()
                    .save_block(block_cache_receiver, executor.clone());

                //sync
                handle.sync_manager.borrow().sync_block_msg(
                    executor.clone(),
                    sync_block_receiver,
                    sync_signal_receiver,
                );

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
        let mint_key = self.mint_key.take().expect("self_key is none.");
        let event_handle_network_events = self
            .event_handle_network_events
            .take()
            .expect("[consensus] Failed to start; network_events stream is already taken");
        let event_handle_receiver = self
            .event_handle_receiver
            .take()
            .expect("[consensus]: self receiver is already taken");
        let sync_block_receiver = self
            .sync_block_receiver
            .take()
            .expect("sync_block_receiver is none.");
        let sync_signal_receiver = self
            .sync_signal_receiver
            .take()
            .expect("sync_signal_receiver is none.");

        self.event_handle(
            executor,
            chain_state_network_sender,
            chain_state_network_events,
            mint_key,
            event_handle_network_events,
            event_handle_receiver,
            sync_block_receiver,
            sync_signal_receiver,
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
