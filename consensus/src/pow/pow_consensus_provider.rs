use failure::prelude::*;
use config::config::NodeConfig;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, LongestChainInfo, RequestBlock, RespondBlock, BlockRetrievalStatus,
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
use std::time::{Duration, Instant};
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
use futures_locks::{Mutex, RwLock};
use {
    futures::{
        compat::Future01CompatExt,
        future::{self, FutureExt, TryFutureExt},
    },
};
use std::thread::sleep;
use tokio::timer::Interval;
use std::convert::TryInto;
use cuckoo::consensus::{PowCuckoo, PowService, Proof};
use network::validator_network::PowContext;
use crate::chained_bft::consensusdb::BlockIndex;

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
    sync_block_sender: mpsc::Sender<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>,
    sync_block_receiver: Option<mpsc::Receiver<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>>,
    sync_signal_sender: mpsc::Sender<(PeerId, u64)>,
    sync_signal_receiver: Option<mpsc::Receiver<(PeerId, u64)>>,
    sync_state_sender: mpsc::Sender<SyncState>,
    sync_state_receiver: Option<mpsc::Receiver<SyncState>>,
    sync_height_sender: mpsc::Sender<()>,
    sync_height_receiver: Option<mpsc::Receiver<()>>,

    block_chain: Arc<RwLock<BlockChain>>,
    orphan_blocks: Arc<Mutex<HashMap<HashValue, Vec<HashValue>>>>,//key -> parent_block_id, value -> block_id
    genesis_txn: Vec<SignedTransaction>,
    pow_srv: Arc<PowService>,
    rollback_flag: bool,
}

enum SyncState {
    GOON((PeerId, HashValue)),
    END,
}

struct BlockRetrievalResponse<T> {
    pub status: BlockRetrievalStatus,
    pub blocks: Vec<Block<T>>,
}

#[derive(Clone)]
pub struct BlockChain {
    pub height: u64,
    pub indexs: HashMap<u64, Vec<BlockIndex>>,
    pub hash_height_index: HashMap<HashValue, (u64, usize)>,
    pub main_chain: AtomicRefCell<HashMap<u64, BlockIndex>>,
}

impl BlockChain {
    pub fn longest_chain_height(&self) -> u64 {
        self.height
    }

    pub fn find_height_index_by_block_hash(&self, block_hash: &HashValue) -> Option<&(u64, usize)> {
        self.hash_height_index.get(block_hash)
    }

    pub fn print_block_chain_root(&self, peer_id:PeerId) {
        let height = ((self.hash_height_index.len() - 1) as u64);
        for index in 0..height {
            println!("----->{:?}------>{}----->{:?}", peer_id, height, self.main_chain.borrow().get(&index).expect("eeeee"));
        }
    }

    fn find_index_by_block_hash(&self, block_hash: &HashValue) -> Option<&BlockIndex> {
        match self.find_height_index_by_block_hash(block_hash) {
            Some((height, index)) => {
                let tmp_index = self.indexs.get(height).expect("block hash not exist.");
                tmp_index.get(index.clone())
            }
            None => None
        }
    }

    fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.hash_height_index.contains_key(block_hash)
    }

    fn root_hash(&self) -> &HashValue {
        &self.indexs.get(&self.longest_chain_height()).expect("get root hash err.")[0].id
    }

    pub fn connect_block(&mut self, block_index: BlockIndex) -> (bool, Option<HashValue>) {
        let parent = self.find_height_index_by_block_hash(&block_index.parent_block_id);
        match parent {
            Some((parent_height, _parent_index)) => {
                let height = parent_height.clone();
                let current_block_id = block_index.id.clone();
                let (old, current_index) = if self.height == height {
                    let old_root_hash = self.root_hash().clone();
                    self.height = self.height + 1;
                    self.indexs.insert(self.height, vec![block_index.clone()]);
                    (Some(old_root_hash), 0)
                } else {
                    let tmp_indexs = self.indexs.get_mut(&(height + 1)).expect("height block index not exist.");
                    let current_index = tmp_indexs.len();
                    tmp_indexs.push(block_index);
                    (None, current_index)
                };

                self.hash_height_index.insert(current_block_id, ({height + 1}, current_index));
                (false, old)
            }
            None => {
                (true, None)
            }
        }
    }

    pub fn update_main_chain(&self, height: &u64, block_index: &BlockIndex) {
        self.main_chain.borrow_mut().insert(height.clone(), block_index.clone());
    }

    pub fn find_height_and_block_index(&self, hash: &HashValue) -> (&u64, &BlockIndex) {
        let (index, _) = self.find_height_index_by_block_hash(hash).expect("block height not exist.");
        let block_index = self.find_index_by_block_hash(hash).expect("block index not exist.");
        (index, block_index)
    }

    pub fn find_ancestor_until_main_chain(&self, hash: &HashValue) -> Option<(Vec<HashValue>, BlockIndex)> {
        let mut ancestors = vec![];
        let mut latest_hash = hash;
        let mut block_index = None;
        for i in 0..100 {
            let (height, index) = match self.find_height_index_by_block_hash(latest_hash) {
                Some(h_i) => h_i,
                None => return None
            };

            let tmp_index = self.indexs.get(height).expect("block hash not exist.");
            let tmp_block_index = tmp_index.get(index.clone());

            match tmp_block_index {
                Some(b_i) => {
                    let current_id = b_i.id;
                    latest_hash = &b_i.parent_block_id;
                    block_index = Some(b_i.clone());

                    if self.main_chain.borrow().get(height).expect("get block index from main chain err.").clone().id == current_id {
                        break;
                    } else {
                        ancestors.push(current_id);
                    }
                },
                None => return None,
            }
        }

        ancestors.reverse();
        Some((ancestors, block_index.expect("block_index is none.")))
    }

    fn find_ancestor(&self, first_hash: &HashValue, second_hash: &HashValue) -> Option<(Vec<&HashValue>, Vec<&HashValue>)> {
        if first_hash != second_hash {
            let first_index = self.find_index_by_block_hash(first_hash);
            match first_index {
                Some(block_index_1) => {
                    let second_index = self.find_index_by_block_hash(second_hash);
                    match second_index {
                        Some(block_index_2) => {
                            if block_index_1.parent_block_id != block_index_2.parent_block_id {
                                let mut first_ancestors = vec![];
                                let mut second_ancestors = vec![];
                                first_ancestors.push(&block_index_1.parent_block_id);
                                second_ancestors.push(&block_index_2.parent_block_id);

                                let ancestors = self.find_ancestor(&block_index_1.parent_block_id, &block_index_2.parent_block_id);
                                match ancestors {
                                    Some((f, s)) => {
                                        first_ancestors.append(&mut f.clone());
                                        second_ancestors.append(&mut s.clone());
                                    }
                                    None => {}
                                }

                                return Some((first_ancestors, second_ancestors));
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
        return None;
    }
}

impl EventHandle {
    pub fn new(network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               author: AccountAddress, peers: ConsensusPeersConfig, storage_dir: PathBuf,
               sync_model: AtomicBool, genesis_txn: Transaction, rollback_flag: bool) -> Self {
        let (block_cache_sender, block_cache_receiver) = mpsc::channel(10);

        //sync
//        let sync_blocks = Arc::new(Mutex::new(vec![]));
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(1024);
        let (sync_state_sender, sync_state_receiver) = mpsc::channel(10);
        let (sync_height_sender, sync_height_receiver) = mpsc::channel(10);

        //longest chain
        let genesis_block_index = BlockIndex { id: *GENESIS_BLOCK_ID, parent_block_id: *PRE_GENESIS_BLOCK_ID };
        let genesis_height = 0;
        let mut index_map = HashMap::new();
        index_map.insert(genesis_height, vec![genesis_block_index.clone()]);
        let indexs = index_map;
        let mut hash_height_index = HashMap::new();
        hash_height_index.insert(*GENESIS_BLOCK_ID, (genesis_height, 0));
        let mut main_chain = AtomicRefCell::new(HashMap::new());
        main_chain.borrow_mut().insert(genesis_height, genesis_block_index);
        let init_block_chain = BlockChain { height: genesis_height, indexs, hash_height_index, main_chain };
        let block_chain = Arc::new(RwLock::new(init_block_chain));
        let mint_block_vec = Arc::new(AtomicRefCell::new(vec![*GENESIS_BLOCK_ID]));
        let block_store = Arc::new(ConsensusDB::new(storage_dir));
        //TODO:init block index
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let orphan_blocks = Arc::new(Mutex::new(HashMap::new()));
        let pow_srv = Arc::new(PowCuckoo::new(6, 8));

        let genesis_txn_vec = match genesis_txn {
            Transaction::UserTransaction(signed_txn) => {
                vec![signed_txn].to_vec()
            },
            _ => {vec![]}
        };

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
            sync_state_receiver: Some(sync_state_receiver),
            sync_height_sender,
            sync_height_receiver: Some(sync_height_receiver),
            block_chain,
            orphan_blocks,
            genesis_txn:genesis_txn_vec,
            pow_srv,
            rollback_flag
        }
    }

    async fn process_new_block_msg(block_cache_sender: &mut mpsc::Sender<Block<Vec<SignedTransaction>>>, new_block: BlockProto, pow_ctx: PowContext, pow_srv: Arc<PowService>) {
        let block: Block<Vec<SignedTransaction>> = new_block.try_into().expect("parse block pb err.");

        let verify = pow_srv.verify(&pow_ctx.header_hash, pow_ctx.nonce, Proof { solve: pow_ctx.solve });
        if verify == false {
            // Not valid block, pass it.
        }
        // insert into block_cache_sender
        debug!("parent block hash: {:?}, new block hash: {:?}", block.parent_id(), block.id());
        if let Err(err) = block_cache_sender.send(block).await {
            error!("send new block err: {:?}", err);
        }

        //TODO:send other peers
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
        let mut block_cache_sender = self.block_cache_sender.clone();
        let pow_srv = self.pow_srv.clone();
        let fut = async move {
            while let Some(Ok(message)) = all_events.next().await {
                match message {
                    Event::PowMessage((peer_id, pow_ctx, msg)) => {
                        let msg = match msg.message {
                            Some(msg) => msg,
                            None => {
                                warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                continue;
                            }
                        };

                        match msg.clone() {
                            ConsensusMsg_oneof::NewBlock(new_block) => Self::process_new_block_msg(&mut block_cache_sender, new_block, pow_ctx, pow_srv.clone()).await,
                            ConsensusMsg_oneof::ChainInfo(chain_info) => {
                                let height = block_chain.clone().read().compat().await.unwrap().longest_chain_height();
                                println!("ChainInfo get chain lock:{:?}", self_peer_id);
                                println!("ChainInfo myself : {} :{:?}, other: {} :{:?} ", height, self_peer_id, chain_info.height, AccountAddress::try_from(chain_info.author).expect("author to bytes err."));
                                println!("ChainInfo drop chain lock:{:?}", self_peer_id);
                                if height > chain_info.height {
                                    let chain_info_msg = Self::chain_info_msg(height, self_peer_id.clone());
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
                                } else { None };
                                let block_chain_lock = block_chain.clone().read().compat().await.unwrap();
                                println!("RequestBlock get chain lock");
                                let mut exist_flag = false;
                                for i in 0..req_block.num_blocks {
                                    let hash = match latest_block {
                                        Some(child_hash) => {
                                            let child = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&child_hash);
                                            match child {
                                                Some(c) => {
                                                    child_hash
                                                }
                                                None => {
                                                    exist_flag = true;
                                                    break;
                                                }
                                            }
                                        }
                                        None => {
                                            block_chain_lock.indexs.get(&block_chain_lock.longest_chain_height()).unwrap()[0].id
                                        }
                                    };
                                    if hash == *PRE_GENESIS_BLOCK_ID {
                                        break;
                                    }

                                    let block = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&hash).expect("block not exist.");
                                    latest_block = Some(block.parent_id());
                                    blocks.push(block.into());

                                    if hash == *GENESIS_BLOCK_ID {
                                        break;
                                    }
                                }

                                drop(block_chain_lock);
                                println!("RequestBlock drop chain lock");

                                let status = if exist_flag {
                                    //BlockRetrievalStatus::IDNOTFOUND
                                    //1
                                    0
                                } else {
                                    if (blocks.len() as u64) == req_block.num_blocks {
                                        //BlockRetrievalStatus::SUCCEEDED
                                        0
                                    } else {
                                        //BlockRetrievalStatus::NOTENOUGHBLOCKS
                                        2
                                    }
                                };

                                let resp_block = RespondBlock { status, blocks };
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

                                if let Err(err) = sync_block_sender.send((peer_id, response)).await {
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
//                        match msg.message {
//                            Some(RequestBlock(request)) => {
//                                Self::process_sync_block_msg().await;
//                                warn!("RequestBlock : {:?}", request);
//                            }
//                            _ => {
//                                warn!("Unexpected RPC from {}: {:?}", peer_id, msg);
//                                continue;
//                            }
//                        };
                    }
                    Event::NewPeer(peer_id) => {
                        //TODO:
                        let height = block_chain.clone().read().compat().await.unwrap().longest_chain_height();
                        println!("NewPeer get chain lock");
                        println!("NewPeer drop chain lock");
                        let chain_info_msg = Self::chain_info_msg(height, self_peer_id.clone());
                        Self::send_consensus_msg(peer_id, &mut network_sender.clone(), self_peer_id.clone(), &mut self_sender.clone(), chain_info_msg).await;
                        debug!("Peer {} connected", peer_id);
                    }
                    Event::LostPeer(peer_id) => {
                        debug!("Peer {} disconnected", peer_id);
                    }
                    Event::Message((peer_id, message)) => {
                        debug!("Recive message without handle");
                    }
                }
            }
        };
        executor.spawn(fut);
    }

    fn process_orphan_blocks(&self) {
        //TODO:orphan
    }

    fn chain_info_msg(height: u64, author:AccountAddress) -> ConsensusMsg {
        let mut info = LongestChainInfo::default();
        info.height = height;
        info.author = AccountAddress::try_into(author).expect("author to bytes err.");
        ConsensusMsg {
            message: Some(ConsensusMsg_oneof::ChainInfo(info)),
        }
    }

    async fn broadcast_consensus_msg(consensus_peers_config: ConsensusPeersConfig, network_sender: &mut ConsensusNetworkSender, self_flag: bool,
                                     self_peer_id: PeerId, self_sender: &mut channel::Sender<failure::Result<Event<ConsensusMsg>>>, msg: ConsensusMsg, pow_ctx: Option<PowContext>) {
        if self_flag {
            let event_msg = Ok(Event::PowMessage((self_peer_id, pow_ctx.expect("Pow context not set"), msg.clone())));
            if let Err(err) = self_sender.send(event_msg).await {
                error!("Error delivering a self proposal: {:?}", err);
            }
        }
        let msg_raw = msg.to_bytes().unwrap();
            if let Err(err) = network_sender.broadcast_bytes(msg_raw.clone()).await {
                error!(
                    "Error broadcasting proposal  error: {:?}, msg: {:?}",
                     err, msg
                );
            }

//        for peer in consensus_peers_config.get_validator_verifier().get_ordered_account_addresses() {
////            if self_flag || peer != self_peer_id {
//            if let Err(err) = network_sender.send_bytes(peer, msg_raw.clone()).await {
//                error!(
//                    "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
//                    peer, err, msg
//                );
//            }
////            }
//        }
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

    fn sync_block_req(hash: Option<HashValue>) -> ConsensusMsg {
        let num_blocks = 10;
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

    fn sync_height(&mut self, executor: TaskExecutor) {
        let mut sync_height_sender = self.sync_height_sender.clone();
        let task = Interval::new(Instant::now(), Duration::from_secs(10))
            .for_each(move |_| {
                if let Err(err) = block_on(sync_height_sender.send(())) {
                    error!("send sync block err: {:?}", err);
                }
                future::ready(())
            });

        executor.spawn(task);
    }

    fn save_block(&mut self, executor: TaskExecutor) {
        let block_db = self.block_store.clone();
        let block_chain = self.block_chain.clone();
        let orphan_blocks = self.orphan_blocks.clone();
        let mut block_cache_receiver = self.block_cache_receiver.take().expect("block_cache_receiver is none.");
        let txn_manager = self.txn_manager.clone();
        let state_computer = self.state_computer.clone();
        let genesis_txn_vec = self.genesis_txn.clone();
        let self_peer_id = self.author;
        let rollback_flag = self.rollback_flag;
        let chain_fut = async move {
            loop {
                ::futures::select! {
                    block = block_cache_receiver.select_next_some() => {
                        //TODO:Verify block

                        // 2. compute with state_computer
                        let payload = match block.payload() {
                            Some(txns) => txns.clone(),
                            None => vec![],
                        };

                        // Pre compute
                        // 1. orphan block
                        let parent_block_id = block.parent_id();
                        let block_index = BlockIndex { id: block.id(), parent_block_id };
                        let mut chain_lock = block_chain.write().compat().await.unwrap();
                        let mut save_flag = false;
                        if chain_lock.block_exist(&parent_block_id) {
                            let mut commit_txn_vec = Vec::<Vec<SignedTransaction>>::new();
                            let mut pre_compute_parent_block_id = *PRE_GENESIS_BLOCK_ID;
                            if parent_block_id == *GENESIS_BLOCK_ID {
                                commit_txn_vec.push(genesis_txn_vec.clone());
                            } else {
                                // 2. find ancestors
                                let (ancestors, block_index) = chain_lock.find_ancestor_until_main_chain(&parent_block_id).expect("find ancestors err.");

                                // 3. find blocks
                                let blocks = block_db.get_blocks_by_hashs::<Vec<SignedTransaction>>(ancestors).expect("find blocks err.");

                                for b in blocks {
                                    let tmp_txns = match b.payload() {
                                        Some(t) => t.clone(),
                                        None => vec![],
                                    };
                                    commit_txn_vec.push(tmp_txns);
                                }

                                pre_compute_parent_block_id = block_index.parent_block_id;
                            }
                            commit_txn_vec.push(payload.clone());

                            // 4. call pre_compute
                            match state_computer.pre_compute(pre_compute_parent_block_id, commit_txn_vec).await {
                                Ok((compute_state, state_id)) => {
                                    if state_id == block.quorum_cert().certified_state_id() && compute_state.root_hash() == block.quorum_cert().ledger_info().ledger_info().transaction_accumulator_hash() {
                                        save_flag = true;
                                    }
                                }
                                Err(e) => {println!("{:?}", e)},
                            }
                        } else {
                            //save orphan block
                            let mut write_lock = orphan_blocks.lock().compat().await.unwrap();
                            write_lock.insert(block_index.parent_block_id, vec![block_index.id]);
                        }

                        if save_flag {
                            //save index
                            let (orphan_flag, old) = chain_lock.connect_block(block_index.clone());

                            if !orphan_flag {
                                //update main chain
                                match old {
                                    Some(old_root) => {
                                        let mut main_chain_indexs:Vec<&HashValue> = Vec::new();
                                        let height = chain_lock.longest_chain_height();

                                        if (rollback_flag && height > 2) || old_root != parent_block_id {//rollback
                                            println!("--------7777----rollback----");

                                            let (rollback_vec, mut commit_vec) = if rollback_flag {
                                                (vec![&old_root], vec![&old_root])
                                            } else {
                                                chain_lock.find_ancestor(&old_root, &parent_block_id).expect("find ancestor err.")
                                            };
                                            let rollback_len = rollback_vec.len();
                                            let ancestor_block_id = chain_lock.find_index_by_block_hash(rollback_vec.get(rollback_len - 1).expect("latest_block_id err.")).expect("block index is none err.").parent_block_id;

                                            //1. reset exector
                                            state_computer.rollback(ancestor_block_id).await;
                                            println!("--------8888----rollback----{:?}-----{:?}", old_root, ancestor_block_id);

                                            //2. add txn to mempool

                                            //3. commit
                                            for commit in commit_vec.iter().rev() {
                                                println!("--------9999----rollback----");
                                                // 1. query block
                                                let commit_block = block_db.get_block_by_hash::<Vec<SignedTransaction>>(commit).expect("block not find in database err.");
                                                // 2. commit block
                                                Self::execut_and_commit_block(block_db.clone(), commit_block, txn_manager.clone(), state_computer.clone()).await;
                                            }

    //                                      // 4. update main chain
                                            main_chain_indexs.append(&mut commit_vec);
                                        }

                                        //4.save latest block
                                        let id = block.id();
                                        Self::execut_and_commit_block(block_db.clone(), block.clone(), txn_manager.clone(), state_computer.clone()).await;

                                        //5. update main chain
                                        main_chain_indexs.append(&mut vec![&id].to_vec());
                                        for hash in main_chain_indexs {
                                            let (h, b_i) = chain_lock.find_height_and_block_index(hash);
                                            chain_lock.update_main_chain(h, b_i);
                                            block_db.insert_block_index(h, b_i);
                                        }
                                    }
                                    None => {
                                        // save latest block
                                        Self::execut_and_commit_block(block_db.clone(), block, txn_manager.clone(), state_computer.clone()).await;
                                    }
                                }

                                chain_lock.print_block_chain_root(self_peer_id);
                            }

                            drop(chain_lock);
                            println!("save block drop chain lock");
                        }
                    }
                }
            }
        };

        executor.spawn(chain_fut);
    }

    async fn execut_and_commit_block(block_db: Arc<ConsensusDB>, block: Block<Vec<SignedTransaction>>, txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>, state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>) {
        // 2. compute with state_computer
        let payload = match block.payload() {
            Some(txns) => txns.clone(),
            None => vec![],
        };
        let compute_res = state_computer.compute(block.parent_id(), block.id(), &payload).await.expect("compute block err.");

        // 3. remove tx from mempool
        match block.payload() {
            Some(txns) => {
                if let Err(e) = txn_manager.commit_txns(txns, &compute_res, block.timestamp_usecs()).await {
                    error!("Failed to notify mempool: {:?}", e);
                }
            }
            None => {}
        }

        // 4. commit to state_computer
        if let Err(e) = state_computer.commit_with_id(block.id(), block.quorum_cert().ledger_info().clone()).await {
            error!("Failed to commit block: {:?}", e);
        }

        // 5. save block
        let mut blocks: Vec<Block<Vec<SignedTransaction>>> = Vec::new();
        blocks.push(block.clone());
        let mut qcs = Vec::new();
        qcs.push(block.quorum_cert().clone());
        block_db.save_blocks_and_quorum_certificates(blocks, qcs);
    }

    fn sync_block_msg(&mut self, executor: TaskExecutor) {
        let sync_block_chain = self.block_chain.clone();
        let mut sync_block_receiver = self.sync_block_receiver.take().expect("sync_block_receiver is none.");
        let mut sync_signal_receiver = self.sync_signal_receiver.take().expect("sync_signal_receiver is none.");
        let mut sync_network_sender = self.network_sender.clone();
        let mut sync_state_sender = self.sync_state_sender.clone();
        let mut sync_state_receiver = self.sync_state_receiver.take().expect("sync_state_receiver is none.");
        let mut sync_height_receiver = self.sync_height_receiver.take().expect("sync_height_receiver is none.");
        let mut sync_block_cache_sender = self.block_cache_sender.clone();
        let self_peer_id = self.author.clone();
        let mut sync_self_sender = self.self_sender.clone();
        let consensus_peers = self.peers.clone();

        let sync_fut = async move {
            loop {
                ::futures::select! {
                    (_) = sync_height_receiver.select_next_some() => {
                        let height = sync_block_chain.read().compat().await.unwrap().longest_chain_height();
                        let chain_info_msg = Self::chain_info_msg(height, self_peer_id.clone());
                        Self::broadcast_consensus_msg(consensus_peers.clone(), &mut sync_network_sender.clone(), false, self_peer_id.clone(), &mut sync_self_sender.clone(), chain_info_msg, None).await;
                    },
                    (peer_id, height) = sync_signal_receiver.select_next_some() => {
                        //sync data from latest block
                        //TODO:timeout
                        let sync_block_req_msg = Self::sync_block_req(None);

                        Self::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                    },
                    (sync_state) = sync_state_receiver.select_next_some() => {
                        match sync_state {
                            SyncState::END => {
                            },
                            SyncState::GOON((peer_id, hash)) => {
                                let sync_block_req_msg = Self::sync_block_req(Some(hash));
                                Self::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                            }
                        }
                    },
                    (peer_id, sync_block_resp) = sync_block_receiver.select_next_some() => {
                        // 2. save data to cache
                        let status = sync_block_resp.status;
                        let mut blocks = sync_block_resp.blocks;

                        let mut flag = false;
                        let mut end_block = None;
                        if blocks.len() > 0 {
                            blocks.reverse();
                            let sync_block_chain_lock = sync_block_chain.read().compat().await.unwrap();
                            println!("sync block get chain lock");
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

                            drop(sync_block_chain_lock);
                            println!("sync block drop chain lock");
                        }

                        let state = match status {
                            BlockRetrievalStatus::Succeeded => {
                                if flag {
                                    SyncState::END
                                } else {
                                    SyncState::GOON((peer_id, end_block.unwrap()))
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
        let consensus_peers = self.peers.clone();
        let mut self_sender = self.self_sender.clone();
        let block_db = self.block_store.clone();
        let pow_srv = self.pow_srv.clone();
        let genesis_txn_vec = self.genesis_txn.clone();
        let mint_fut = async move {
            let block_chain_clone = block_chain.clone();
            loop {
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
                        let block_read_lock = block_chain_clone.read().compat().await.unwrap();
                        println!("mint block get chain lock");
                        let height = &block_read_lock.longest_chain_height();
                        let root = block_read_lock.root_hash();
                        println!("--------6666--------{:?}===={}------>{:?}", mint_author, height, root);
                        let parent_block = block_read_lock.indexs.get(height).unwrap()[0].clone();
                        println!("mint block drop chain lock:{}", height);
                        drop(block_read_lock);
                        if txns.len() > 0 {
                            //create block
                            let parent_block_id = parent_block.id;
                            let grandpa_block_id = parent_block.parent_block_id;
                            //QC with parent block id
                            let quorum_cert = if parent_block_id != *GENESIS_BLOCK_ID {
                                let parent_block = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&parent_block_id).expect("block not find in database err.");
                                parent_block.quorum_cert().clone()
                            } else {
                                QuorumCert::certificate_for_genesis()
                            };

                            let mut tmp = vec![];
                            tmp.push(genesis_txn_vec.clone());
                            let (pre_compute_parent_block_id, commit_txn_vec) = if parent_block_id != *GENESIS_BLOCK_ID {
                                (grandpa_block_id, vec![txns.clone()].to_vec())
                            } else {
                                tmp.push(txns.clone());
                                (*PRE_GENESIS_BLOCK_ID, tmp)
                            };

                            //compute current block state id
                            match mint_state_computer.pre_compute(pre_compute_parent_block_id, commit_txn_vec).await {
                                Ok((compute_state, state_id)) => {
                                    let txn_len = compute_state.version();
                                    let vote_data = VoteData::new(parent_block_id,state_id, quorum_cert.certified_block_round(), parent_block_id, quorum_cert.parent_block_round());
                                    let parent_li = quorum_cert.ledger_info().ledger_info().clone();
                                    let v_s = match parent_li.next_validator_set() {
                                        Some(n_v_s) => {
                                            Some(n_v_s.clone())
                                        }
                                        None => {
                                            None
                                        }
                                    };
                                    let li = LedgerInfo::new(txn_len, compute_state.root_hash(), vote_data.hash(), parent_block_id, parent_li.epoch_num(), parent_li.timestamp_usecs(), v_s);
                                    let signer = ValidatorSigner::genesis();//TODO:change signer
                                    let signature = signer.sign_message(li.hash()).expect("Fail to sign genesis ledger info");
                                    let mut signatures = BTreeMap::new();
                                    signatures.insert(signer.author(), signature);
                                    let new_qc = QuorumCert::new(vote_data, LedgerInfoWithSignatures::new(li.clone(), signatures));

                                    let block = Block::<Vec<SignedTransaction>>::new_internal(
                                        txns,
                                        0,
                                        height + 1,
                                        0,
                                        new_qc,
                                        &ValidatorSigner::from_int(1),
                                    );

                                    let block_pb = Into::<BlockProto>::into(block);

                                    // send block
                                    let msg = ConsensusMsg {
                                        message: Some(ConsensusMsg_oneof::NewBlock(block_pb)),
                                    };
                                    let nonce = generate_nonce();
                                    let proof = pow_srv.solve(li.hash().as_ref(), nonce);
                                    let solve = match proof {
                                        Some(proof) => proof.solve,
                                        None => vec![]
                                    };
                                    let pow_ctx = PowContext {
                                        header_hash: li.hash().to_vec(),
                                        nonce,
                                        solve,
                                    };
                                    Self::broadcast_consensus_msg(consensus_peers.clone(), &mut mint_network_sender, true, mint_author, &mut self_sender, msg, Some(pow_ctx)).await;
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                },
                            }
                        }

                        let mut r = rand::thread_rng();
                        r.gen::<i32>();
                        let sleep_time = r.gen_range(10, 20);
                        println!("sleep begin.");
                        sleep(Duration::from_secs(sleep_time));
                        println!("sleep end.");
                    }
                    _ => {}
                }


            }
        };
        executor.spawn(mint_fut);
    }
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
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
               synchronizer_client: Arc<StateSyncClient>,
               rollback_flag: bool) -> Self {
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

        let genesis_transaction = node_config
            .get_genesis_transaction()
            .expect("failed to load genesis transaction!");

        let event_handle = EventHandle::new(network_sender, network_events, txn_manager, state_computer, author, peers_config, node_config.get_storage_dir(), sync_flag, genesis_transaction, rollback_flag);
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

                //save
                handle.save_block(executor.clone());

                //sync
                handle.sync_block_msg(executor.clone());

                //sync height
                handle.sync_height(executor);

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
        Ok(())
    }

    fn stop(&mut self) {
        //TODO
        // 1. stop mint
        // 2. stop process event
        unimplemented!()
    }
}