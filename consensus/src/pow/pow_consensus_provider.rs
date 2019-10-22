use failure::prelude::*;
use config::config::NodeConfig;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, LongestChainInfo, RequestBlock, RespondBlock, BlockRetrievalStatus
    },
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event, RpcError}};
use std::{sync::Arc, thread};
use vm_runtime::MoveVM;
use executor::Executor;
use state_synchronizer::StateSyncClient;
use crate::state_replication::{StateComputer, TxnManager};
use libra_types::transaction::{SignedTransaction, Transaction};
use crate::{
    consensus_provider::ConsensusProvider,
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use consensus_types::{block::Block, quorum_cert::QuorumCert, vote_data::VoteData};
use std::collections::{HashSet, BTreeMap};
use crypto::HashValue;
use futures::{executor::block_on, stream::select, SinkExt, StreamExt, TryStreamExt};
use libra_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use crate::chained_bft::consensusdb::ConsensusDB;
use std::time::Duration;
use rand::{self, Rng};
use tokio::runtime::{self, TaskExecutor};
use logger::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use executor::{StateComputeResult, ExecutedState};
use libra_types::transaction::TransactionStatus;
use libra_types::vm_error::{VMStatus, StatusCode};
use crate::counters;
use std::{convert::{TryFrom, From}, path::PathBuf};
use crypto::hash::CryptoHash;
use libra_types::crypto_proxies::ValidatorSigner;
use libra_types::account_address::AccountAddress;
use channel;
use atomic_refcell::AtomicRefCell;
use config::trusted_peers::ConsensusPeersConfig;
use prost_ext::MessageExt;
use crypto::hash::{GENESIS_BLOCK_ID, PRE_GENESIS_BLOCK_ID};
use futures::{channel::mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use libra_types::validator_set::ValidatorSet;
use libra_types::PeerId;
use std::collections::HashMap;
use bytes::Bytes;
use futures_locks::Mutex;
use {
    futures::{
        compat::Future01CompatExt,
        future::{FutureExt, TryFutureExt},
    },
};

pub struct EventHandle {
    block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
    block_cache_receiver: Option<mpsc::Receiver<Block<Vec<SignedTransaction>>>>,
    block_store: Arc<ConsensusDB>,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
    author: AccountAddress,
    peers: ConsensusPeersConfig,
//    sync_model: Arc<AtomicBool>,

    //    sync_blocks:Arc<Mutex<Vec<Block<Vec<SignedTransaction>>>>>,
    sync_block_sender: mpsc::Sender<BlockRetrievalResponse<Vec<SignedTransaction>>>,
    sync_block_receiver: Option<mpsc::Receiver<BlockRetrievalResponse<Vec<SignedTransaction>>>>,
    sync_signal_sender: mpsc::Sender<(PeerId, u64)>,
    sync_signal_receiver: Option<mpsc::Receiver<(PeerId, u64)>>,
    sync_state_sender: mpsc::Sender<SyncState>,
    sync_state_receiver: Option<mpsc::Receiver<SyncState>>,

    block_chain: Arc<Mutex<BlockChain>>,
    orphan_blocks: Arc<Mutex<HashMap<HashValue, Vec<HashValue>>>>,//key -> parent_block_id, value -> block_id
}

enum SyncState {
    GOON(HashValue),
    END,
}

struct BlockRetrievalResponse<T> {
    pub status: BlockRetrievalStatus,
    pub blocks: Vec<Block<T>>,
}

#[derive(Clone)]
pub struct BlockIndex {
    pub id: HashValue,
    pub parent_block_id: HashValue,
}

#[derive(Clone)]
pub struct BlockChain {
    pub height: u64,
    pub indexs: HashMap<u64, Vec<BlockIndex>>,
    pub hash_to_height: HashMap<HashValue, u64>,
}

impl BlockChain {
    pub fn longest_chain_height(&self) -> u64 {
        self.height
    }

    pub fn find_height_by_block_hash(&self, block_hash: &HashValue) -> Option<&u64> {
        self.hash_to_height.get(block_hash)
    }

    fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.hash_to_height.contains_key(block_hash)
    }

    pub fn connect_block(&mut self, block_index: BlockIndex) -> bool {
        let parent_height = self.find_height_by_block_hash(&block_index.parent_block_id);
        match parent_height {
            Some(ph) => {
                let height = ph.clone();
                let current_block_id = block_index.id.clone();
                if self.height == height {
                    self.height = self.height + 1;
                    self.indexs.insert(self.height, vec![block_index]);
                } else {
                    self.indexs.get_mut(&(height + 1)).unwrap().push(block_index);
                }

                self.hash_to_height.insert(current_block_id, self.height);
                true
            }
            None => {
                false
            }
        }
    }
}

impl EventHandle {
    pub fn new(network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               author: AccountAddress,
               peers: ConsensusPeersConfig,
               storage_dir: PathBuf, sync_model: AtomicBool) -> Self {
        let (block_cache_sender, block_cache_receiver) = mpsc::channel(10);

        //sync
//        let sync_blocks = Arc::new(Mutex::new(vec![]));
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(10);
        let (sync_state_sender, sync_state_receiver) = mpsc::channel(10);

        //longest chain
        let genesis_block_index = BlockIndex { id: *GENESIS_BLOCK_ID, parent_block_id: *PRE_GENESIS_BLOCK_ID };
        let genesis_height = 0;
        let mut index_map = HashMap::new();
        index_map.insert(genesis_height, vec![genesis_block_index]);
        let indexs = index_map;
        let mut hash_to_height = HashMap::new();
        hash_to_height.insert(*GENESIS_BLOCK_ID, 0);
        let init_block_chain = BlockChain { height: genesis_height, indexs, hash_to_height };
        let block_chain = Arc::new(Mutex::new(init_block_chain));
        let mint_block_vec = Arc::new(AtomicRefCell::new(vec![*GENESIS_BLOCK_ID]));
        let block_store = Arc::new(ConsensusDB::new(storage_dir));
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let orphan_blocks = Arc::new(Mutex::new(HashMap::new()));
        EventHandle {
            block_cache_sender,
            block_cache_receiver: Some(block_cache_receiver),
            block_store,
            network_sender,
            network_events: Some(network_events),
            txn_manager,
            state_computer,
            self_sender,
            self_receiver: Some(self_receiver),
            author,
            peers,
            sync_block_sender,
            sync_block_receiver: Some(sync_block_receiver),
            sync_signal_sender,
            sync_signal_receiver: Some(sync_signal_receiver),
            sync_state_sender,
            sync_state_receiver:Some(sync_state_receiver),
            block_chain,
            orphan_blocks,
        }
    }

    async fn process_new_block_msg(block_chain: Arc<Mutex<BlockChain>>, db: Arc<ConsensusDB>, new_block: BlockProto, orphan_blocks: Arc<Mutex<HashMap<HashValue, Vec<HashValue>>>>) {
        let block = Block::try_from(new_block).expect("parse block pb err.");
        let mut blocks: Vec<Block<Vec<SignedTransaction>>> = Vec::new();
        blocks.push(block.clone());
        db.save_blocks_and_quorum_certificates(blocks, Vec::new());
        let mut chain_lock = block_chain.lock().compat().await.unwrap();
        let block_index = BlockIndex { id: block.id(), parent_block_id: block.parent_id() };
        let flag = chain_lock.connect_block(block_index.clone());
        if !flag {
            let mut write_lock = orphan_blocks.lock().compat().await.unwrap();
            if write_lock.contains_key(&block_index.parent_block_id) {
                write_lock.get_mut(&block_index.parent_block_id).unwrap().push(block_index.id);
            } else {
                write_lock.insert(block_index.parent_block_id, vec![block_index.id]);
            }
        }

        //TODO:send other peers

        drop(chain_lock);
    }

    pub fn event_process(&mut self, executor: TaskExecutor) {
        let network_events = self
            .network_events
            .take()
            .expect("[consensus] Failed to start; network_events stream is already taken")
            .map_err(Into::<failure::Error>::into);

        let own_msgs = self
            .self_receiver
            .take()
            .expect("[consensus]: self receiver is already taken");

        let mut all_events = select(network_events, own_msgs);
        let block_db = self.block_store.clone();
        let block_chain = self.block_chain.clone();
        let consensus_peers = self.peers.clone();
        let mut network_sender = self.network_sender.clone();
        let self_peer_id = self.author;
        let mut self_sender = self.self_sender.clone();
//        let sync_model = self.sync_model.clone();
        let orphan_blocks = self.orphan_blocks.clone();
        let mut sync_signal_sender = self.sync_signal_sender.clone();
        let mut sync_block_sender = self.sync_block_sender.clone();

        let fut = async move {
            while let Some(Ok(message)) = all_events.next().await {
                match message {
                    Event::Message((peer_id, msg)) => {
                        let msg = match msg.message {
                            Some(msg) => msg,
                            None => {
                                warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                continue;
                            }
                        };

                        match msg.clone() {
                            ConsensusMsg_oneof::NewBlock(new_block) => Self::process_new_block_msg(block_chain.clone(), block_db.clone(), new_block, orphan_blocks.clone()).await,
                            ConsensusMsg_oneof::ChainInfo(chain_info) => {
                                let height = block_chain.clone().lock().compat().await.unwrap().longest_chain_height();
                                if height > chain_info.height {
                                    let chain_info_msg = Self::chain_info_msg(height);
                                    Self::send_consensus_msg(peer_id, &mut network_sender.clone(), self_peer_id.clone(), &mut self_sender.clone(), chain_info_msg).await;
                                } else {
                                    if height < chain_info.height {
                                        if let Err(err) = sync_signal_sender.clone().send((peer_id, chain_info.height)).await {
                                            error!("send sync signal err: {:?}", err);
                                        }
                                    }
                                };
                                ()
                            }
                            ConsensusMsg_oneof::RequestBlock(req_block) => {
                                let mut blocks = vec![];
                                let mut latest_block = if req_block.block_id.len() > 0 {
                                    Some(HashValue::from_slice(req_block.block_id.as_ref()).unwrap())
                                } else {None};
                                let block_chain_lock = block_chain.clone().lock().compat().await.unwrap();
                                let mut exist_flag = false;
                                for i in 0 .. req_block.num_blocks {
                                    let hash = match latest_block {
                                        Some(child_hash) => {
                                            let child = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&child_hash);
                                            match child {
                                                Some(c) => {
                                                    c.parent_id()
                                                }
                                                None => {
                                                    exist_flag = true;
                                                    break;
                                                }
                                            }
                                        },
                                        None => {
                                            block_chain_lock.indexs.get(&block_chain_lock.longest_chain_height()).unwrap()[0].id
                                        }
                                    };
                                    if hash == *PRE_GENESIS_BLOCK_ID{
                                        break;
                                    }

                                    let block = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&hash).expect("block not exist.");
                                    blocks.push(block.into());

                                    if hash == *GENESIS_BLOCK_ID {
                                        break;
                                    }
                                }

                                let status = if exist_flag {
                                    //BlockRetrievalStatus::IDNOTFOUND
                                    1
                                } else {
                                    if (blocks.len() as u64) == req_block.num_blocks {
                                        //BlockRetrievalStatus::SUCCEEDED
                                        0
                                    } else {
                                        //BlockRetrievalStatus::NOTENOUGHBLOCKS
                                        2
                                    }
                                };

                                let resp_block = RespondBlock {status, blocks};
                                let resp_block_msg = ConsensusMsg {
                                    message: Some(ConsensusMsg_oneof::RespondBlock(resp_block)),
                                };

                                Self::send_consensus_msg(peer_id, &mut network_sender.clone(), self_peer_id.clone(), &mut self_sender.clone(), resp_block_msg).await;
                            }
                            ConsensusMsg_oneof::RespondBlock(res_block) => {
                                let mut blocks = vec![];
                                let status = res_block.status();
                                for block in res_block.blocks.into_iter() {
                                    match Block::try_from(block) {
                                        Ok(block) => {
                                            blocks.push(block);
                                        }
                                        Err(e) => error!("Failed to deserialize block because of {:?}", e),
                                    };
                                }
                                let response = BlockRetrievalResponse { status, blocks };

                                if let Err(err) = sync_block_sender.send(response).await {
                                    error!("send sync block err: {:?}", err);
                                };
                                ()
                            }
                            _ => {
                                warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                continue;
                            }
                        }
                    }
                    Event::RpcRequest((peer_id, msg, callback)) => {
                        match msg.message {
                            Some(RequestBlock(request)) => {
                                Self::process_sync_block_msg().await;
                                warn!("RequestBlock : {:?}", request);
                            }
                            _ => {
                                warn!("Unexpected RPC from {}: {:?}", peer_id, msg);
                                continue;
                            }
                        };
                    }
                    Event::NewPeer(peer_id) => {
//                        let msg = Self::chain_info_msg(block_chain.clone().lock().unwrap().height as u64);
//                        Self::broadcast_consensus_msg(consensus_peers.clone(), &mut network_sender, self_peer_id, &mut self_sender, msg).await;
                        debug!("Peer {} connected", peer_id);
                    }
                    Event::LostPeer(peer_id) => {
                        debug!("Peer {} disconnected", peer_id);
                    }
                }
            }
        };
        executor.spawn(fut);
    }

    fn process_orphan_blocks(&self) {
        //TODO:处理孤块
    }

    fn chain_info_msg(height: u64) -> ConsensusMsg {
        let mut info = LongestChainInfo::default();
        info.height = height;
        ConsensusMsg {
            message: Some(ConsensusMsg_oneof::ChainInfo(info)),
        }
    }

    async fn broadcast_consensus_msg(consensus_peers_config: ConsensusPeersConfig, network_sender: &mut ConsensusNetworkSender,
                                     self_peer_id: PeerId, self_sender: &mut channel::Sender<failure::Result<Event<ConsensusMsg>>>, msg: ConsensusMsg) {
        let event_msg = Ok(Event::Message((self_peer_id, msg.clone())));
        if let Err(err) = self_sender.send(event_msg).await {
            error!("Error delivering a self proposal: {:?}", err);
        }
        let msg_raw = msg.to_bytes().unwrap();
        for peer in consensus_peers_config.get_validator_verifier().get_ordered_account_addresses() {
            if let Err(err) = network_sender.send_bytes(peer, msg_raw.clone()).await {
                error!(
                    "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
                    peer, err, msg
                );
            }
        }
    }

    async fn send_consensus_msg(send_peer_id: PeerId, network_sender: &mut ConsensusNetworkSender, self_peer_id: PeerId,
                                self_sender: &mut channel::Sender<failure::Result<Event<ConsensusMsg>>>, msg: ConsensusMsg) {
        if send_peer_id == self_peer_id {
            let event_msg = Ok(Event::Message((self_peer_id, msg.clone())));
            if let Err(err) = self_sender.send(event_msg).await {
                error!("Error delivering a self proposal: {:?}", err);
            }
        } else {
            let msg_raw = msg.to_bytes().unwrap();
            if let Err(err) = network_sender.send_bytes(send_peer_id, msg_raw.clone()).await {
                error!(
                    "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
                    send_peer_id, err, msg
                );
            }
        }
    }

    fn sync_block_req(hash:Option<HashValue>) -> ConsensusMsg {
        let num_blocks= 10;
        let req = match hash {
            None => RequestBlock { block_id: vec![], num_blocks },
            Some(h) => {
                RequestBlock { block_id: h.to_vec(), num_blocks }
            }
        };
        ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RequestBlock(req)),
        }
    }

    fn sync_block_msg(&mut self, executor: TaskExecutor) {

        let sync_block_chain = self.block_chain.clone();
        let mut sync_block_receiver = self.sync_block_receiver.take().expect("sync_block_receiver is none.");
        let mut sync_signal_receiver = self.sync_signal_receiver.take().expect("sync_signal_receiver is none.");
        let mut sync_network_sender = self.network_sender.clone();
        let mut sync_state_sender = self.sync_state_sender.clone();
        let mut sync_state_receiver = self.sync_state_receiver.take().expect("sync_state_receiver is none.");
        let mut sync_block_cache_sender = self.block_cache_sender.clone();
        let self_peer_id = self.author.clone();
        let mut sync_self_sender = self.self_sender.clone();

        let sync_fut = async move {
            loop {
                ::futures::select! {
                    (peer_id, height) = sync_signal_receiver.select_next_some() => {
                        //TODO:同步数据[timeout]
                        // 1. 从最新块，倒过来同步
                        let sync_block_req_msg = Self::sync_block_req(None);

                        // 3. 直到本地有相同的
                        // 4. 回滚旧的block
                        // 5. 应用新的block
                        // 6. 从最新块开始挖矿

                        Self::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                        while let Ok(Some(state)) = sync_state_receiver.try_next() {
                            match state {
                                SyncState::END => {
                                    break;
                                },
                                SyncState::GOON(hash) => {
                                    let sync_block_req_msg = Self::sync_block_req(Some(hash));
                                    Self::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                                }
                            }

                        }
                    }
                    sync_block_resp = sync_block_receiver.select_next_some() => {
                        // 2. 把同步的数据暂存起来
                        let status = sync_block_resp.status;
                        let blocks = sync_block_resp.blocks;

                        let mut flag = false;
                        let mut end_block = None;
                        if blocks.len() > 0 {
                            let sync_block_chain_lock = sync_block_chain.lock().compat().await.unwrap();
                            for block in blocks {
                                let hash = block.hash();
                                if sync_block_chain_lock.block_exist(&hash) {
                                    flag = true;
                                    break;
                                } else {
                                    end_block = Some(hash);
                                    sync_block_cache_sender.send(block).await;
                                }
                            }
                        }

                        let state = match status {
                            BlockRetrievalStatus::Succeeded => {
                                if flag {
                                    SyncState::END
                                } else {
                                    SyncState::GOON(end_block.unwrap())
                                }
                            }
                            BlockRetrievalStatus::IdNotFound => {
                                //end
                                SyncState::END
                            }
                            BlockRetrievalStatus::NotEnoughBlocks => {
                                SyncState::END
                            }
                        };
                        if let Err(err) = sync_state_sender.clone().send(state).await {
                            error!("send sync block err: {:?}", err);
                        }
                    }
                    complete => {
                        break;
                    }
                }

            }
        };
        executor.spawn(sync_fut);
    }

    pub fn mint(&self, executor: TaskExecutor) {
        let mint_txn_manager = self.txn_manager.clone();
        let block_chain = self.block_chain.clone();
        let mut block_cache_sender = self.block_cache_sender.clone();
        let mint_state_computer = self.state_computer.clone();
        let mint_peers = self.peers.clone();
        let mut mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let mint_fut = async move {
            let block_chain_clone = block_chain.clone();
            loop {
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
//                        if txns.len() > 0 {
                        // build block
                        let block_id = HashValue::random();
                        let block_read_lock = block_chain_clone.lock().compat().await.unwrap();
                        let height = &block_read_lock.longest_chain_height();
                        let parent_block_id = block_read_lock.indexs.get(height).unwrap()[0].id;
                        let state_compute_result = block_on(mint_state_computer.compute(parent_block_id, block_id, &txns)).unwrap();

                        let info = LedgerInfo::new(state_compute_result.version(), state_compute_result.root_hash(), HashValue::random(), block_id, 0, u64::max_value(), None);
                        let ledger_hash = info.hash();
                        let info_sign = LedgerInfoWithSignatures::new(info, BTreeMap::new());

                        let quorum_cert = QuorumCert::new(
                            VoteData::new(block_id, ledger_hash, 0, parent_block_id, 0),
                            info_sign.clone(),
                        );
                        let block = Block::new_internal(txns, 0, 0, 0, quorum_cert, &ValidatorSigner::from_int(1));

                        // insert into block_cache_sender
                        debug!("block height: {:?}, latest block hash: {:?}, new block hash: {:?}", height, parent_block_id, block_id);
                        if let Err(err) = block_cache_sender.send(block).await {
                            error!("send new block err: {:?}", err);
                        }
                        drop(block_read_lock);

//                        if len == tmp_len {
//                            // commit
//                            let info = LedgerInfo::new(state_compute_result.version(), state_compute_result.root_hash(), HashValue::random(), block_id, 0, u64::max_value(), None);
//                            let ledger_hash = info.hash();
//                            let info_sign = LedgerInfoWithSignatures::new(info, BTreeMap::new());
//
//                            let quorum_cert = QuorumCert::new(
//                                VoteData::new(block_id, ledger_hash, 0, parent_block_id, 0),
//                                info_sign.clone(),
//                            );
//                            let block = Block::new_internal(txns.clone(), 0, 0, 0, quorum_cert, &ValidatorSigner::from_int(1));
//                            let msg = ConsensusMsg {
//                                message: Some(ConsensusMsg_oneof::NewBlock(block.into())),
//                            };
//                            //TODO:1. send block
////                            let msg = Ok(Event::Message((mint_author, msg)));
////                            if let Err(err) = block_cache_sender.send(msg).await {
//////                                error!("Error delivering a self proposal: {:?}", err);
//////                            }
//
//                            let msg_raw = msg.to_bytes().unwrap();
//                            for peer in mint_peers.get_validator_verifier().get_ordered_account_addresses() {
//                                println!("-------4444-------");
//                                if let Err(err) = mint_network_sender.send_bytes(peer, msg_raw.clone()).await {
//                                    error!(
//                                        "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
//                                        peer, err, msg
//                                    );
//                                }
//                            }
//
//                            // 2. commit block
//                            block_on(mint_state_computer.commit(info_sign)).unwrap();
//
//                            let keep = TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED));
//                            //let state_compute = StateComputeResult { executed_state: ExecutedState::state_for_genesis(), compute_status: vec![keep] };
//
//                            // 3. remove from mem pool
//                            mint_txn_manager.commit_txns(&txns, &state_compute_result, u64::max_value());
//                        }
//                        }
                    }
                    _ => {}
                }

                let mut r = rand::thread_rng();
                r.gen::<i32>();
                let sleep_time = r.gen_range(30, 60);
                thread::sleep(Duration::from_secs(sleep_time));
            }
        };
        executor.spawn(mint_fut);
    }
}

pub struct PowConsensusProvider {
    runtime: tokio::runtime::Runtime,
    event_handle: Option<EventHandle>,
}

impl PowConsensusProvider {
    pub fn new(node_config: &mut NodeConfig,
               network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               mempool_client: Arc<MempoolClient>,
               executor: Arc<Executor<MoveVM>>,
               synchronizer_client: Arc<StateSyncClient>) -> Self {
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
        let author =
            AccountAddress::try_from(peer_id_str.clone()).expect("Failed to parse peer id of a validator");

        let peers_config = node_config
            .consensus
            .consensus_peers.clone();

        let sync_flag = if peers_config.peers.len() == 0 || peers_config.peers.contains_key(&peer_id_str) {
            AtomicBool::new(false)
        } else {
            AtomicBool::new(true)
        };

        let event_handle = EventHandle::new(network_sender, network_events, txn_manager, state_computer, author, peers_config, node_config.get_storage_dir(), sync_flag);
        Self {
            runtime,
            event_handle: Some(event_handle),
        }
    }

    pub fn event_handle(&mut self, executor: TaskExecutor) {
        match self.event_handle.take() {
            Some(mut handle) => {
                //mint
                handle.mint(executor.clone());

                //msg
                handle.event_process(executor.clone());

                //sync
                handle.sync_block_msg(executor);
            }
            _ => {}
        }
    }
}

impl ConsensusProvider for PowConsensusProvider {
    fn start(&mut self) -> Result<()> {
        let executor = self.runtime.executor();
        self.event_handle(executor);
        Ok(())
    }

    fn stop(&mut self) {
        //TODO
        // 1. stop mint
        // 2. stop process event
        unimplemented!()
    }
}