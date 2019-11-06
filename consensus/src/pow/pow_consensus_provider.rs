use failure::prelude::*;
use config::config::NodeConfig;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, RequestBlock, RespondBlock, BlockRetrievalStatus,
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
use crate::pow::chain_manager::{ChainManager, BlockChain};
use std::cell::RefCell;

pub struct EventHandle {
    block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
    block_store: Arc<ConsensusDB>,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
    author: AccountAddress,
    peers: ConsensusPeersConfig,

    //sync
    sync_block_sender: mpsc::Sender<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>,
    sync_block_receiver: Option<mpsc::Receiver<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>>,
    sync_signal_sender: mpsc::Sender<(PeerId, u64)>,
    sync_signal_receiver: Option<mpsc::Receiver<(PeerId, u64)>>,
    sync_state_sender: mpsc::Sender<SyncState>,
    sync_state_receiver: Option<mpsc::Receiver<SyncState>>,

    genesis_txn: Vec<SignedTransaction>,
    pow_srv: Arc<PowService>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
}

enum SyncState {
    GOON((PeerId, HashValue)),
    END,
}

struct BlockRetrievalResponse<T> {
    pub status: BlockRetrievalStatus,
    pub blocks: Vec<Block<T>>,
}

impl EventHandle {
    pub fn new(network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               author: AccountAddress, peers: ConsensusPeersConfig, storage_dir: PathBuf,
               genesis_txn: Transaction, rollback_flag: bool) -> Self {
        let (block_cache_sender, block_cache_receiver) = mpsc::channel(10);

        //sync
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(1024);
        let (sync_state_sender, sync_state_receiver) = mpsc::channel(10);

        let mint_block_vec = Arc::new(AtomicRefCell::new(vec![*GENESIS_BLOCK_ID]));
        let block_store = Arc::new(ConsensusDB::new(storage_dir));
        //TODO:init block index
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let pow_srv = Arc::new(PowCuckoo::new(6, 8));

        let genesis_txn_vec = match genesis_txn {
            Transaction::UserTransaction(signed_txn) => {
                vec![signed_txn].to_vec()
            },
            _ => {vec![]}
        };

        let chain_manager = Arc::new(AtomicRefCell::new(ChainManager::new(Some(block_cache_receiver), Arc::clone(&block_store),
                                              txn_manager.clone(), state_computer.clone(), genesis_txn_vec.clone(), rollback_flag)));

        EventHandle {
            block_cache_sender,
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
            genesis_txn:genesis_txn_vec,
            pow_srv,
            chain_manager,
        }
    }

    async fn process_new_block_msg(block_cache_sender: &mut mpsc::Sender<Block<Vec<SignedTransaction>>>, new_block: Block<Vec<SignedTransaction>>, pow_srv: Arc<PowService>) {
//        let verify = pow_srv.verify(&pow_ctx.header_hash, pow_ctx.nonce, Proof { solve: pow_ctx.solve });
//        if verify == false {
//            // Not valid block, pass it.
//        }
        // insert into block_cache_sender
        debug!("parent block hash: {:?}, new block hash: {:?}", new_block.parent_id(), new_block.id());
        if let Err(err) = block_cache_sender.send(new_block).await {
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
        let consensus_peers = self.peers.clone();
        let mut network_sender = self.network_sender.clone();
        let self_peer_id = self.author;
        let mut self_sender = self.self_sender.clone();
        let chain_manager = self.chain_manager.clone();

        let mut sync_signal_sender = self.sync_signal_sender.clone();
        let mut sync_block_sender = self.sync_block_sender.clone();
        let mut block_cache_sender = self.block_cache_sender.clone();
        let pow_srv = self.pow_srv.clone();
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
                            ConsensusMsg_oneof::NewBlock(new_block) => {
                                //TODO:verify block
                                let block: Block<Vec<SignedTransaction>> = new_block.try_into().expect("parse block pb err.");
                                if self_peer_id != peer_id {
                                    let height = chain_manager.borrow().chain_height().await;
                                    if height < block.round() {
                                        if let Err(err) = sync_signal_sender.clone().send((peer_id, block.round())).await {
                                            error!("send sync signal err: {:?}", err);
                                        }
                                    }
                                }
                                Self::process_new_block_msg(&mut block_cache_sender, block, pow_srv.clone()).await
                            },
                            ConsensusMsg_oneof::RequestBlock(req_block) => {
                                let mut blocks = vec![];
                                let mut latest_block = if req_block.block_id.len() > 0 {
                                    Some(HashValue::from_slice(req_block.block_id.as_ref()).unwrap())
                                } else { None };
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
                                            chain_manager.borrow().chain_root().await
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

                                println!("RequestBlock drop chain lock");

                                let status = if exist_flag {
                                    //BlockRetrievalStatus::IDNOTFOUND
                                    //1
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
                        debug!("Peer {:?} connected", peer_id);
                    }
                    Event::LostPeer(peer_id) => {
                        debug!("Peer {:?} disconnected", peer_id);
                    }
                    Event::PowMessage((peer_id, pow_ctx, message)) => {
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

    async fn broadcast_consensus_msg(consensus_peers_config: ConsensusPeersConfig, network_sender: &mut ConsensusNetworkSender, self_flag: bool,
                                     self_peer_id: PeerId, self_sender: &mut channel::Sender<failure::Result<Event<ConsensusMsg>>>, msg: ConsensusMsg, pow_ctx: Option<PowContext>) {
        if self_flag {
            //let event_msg = Ok(Event::PowMessage((self_peer_id, pow_ctx.expect("Pow context not set"), msg.clone())));
            let event_msg = Ok(Event::Message((self_peer_id, msg.clone())));
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

    fn sync_block_msg(&mut self, executor: TaskExecutor) {
        let mut sync_block_receiver = self.sync_block_receiver.take().expect("sync_block_receiver is none.");
        let mut sync_signal_receiver = self.sync_signal_receiver.take().expect("sync_signal_receiver is none.");
        let mut sync_network_sender = self.network_sender.clone();
        let mut sync_state_sender = self.sync_state_sender.clone();
        let mut sync_state_receiver = self.sync_state_receiver.take().expect("sync_state_receiver is none.");
        let mut sync_block_cache_sender = self.block_cache_sender.clone();
        let self_peer_id = self.author.clone();
        let mut sync_self_sender = self.self_sender.clone();
        let consensus_peers = self.peers.clone();
        let chain_manager = self.chain_manager.clone();

        let sync_fut = async move {
            loop {
                ::futures::select! {
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
                            println!("sync block get chain lock");
                            for block in blocks {
                                let hash = block.hash();
                                if chain_manager.borrow().block_exist(&hash).await {
                                    flag = true;
                                    break;
                                } else {
                                    end_block = Some(hash);
                                    sync_block_cache_sender.send(block).await;
                                }
                            }

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
        let chain_manager = self.chain_manager.clone();

        let mint_fut = async move {
            let chain_manager_clone = chain_manager.clone();
            loop {
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
                        let (height, parent_block) = chain_manager_clone.borrow().chain_height_and_root().await;
                        println!("--------6666--------{:?}===={}------>{:?}", mint_author, height, parent_block.id);
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

        let genesis_transaction = node_config
            .get_genesis_transaction()
            .expect("failed to load genesis transaction!");

        let event_handle = EventHandle::new(network_sender, network_events, txn_manager, state_computer, author, peers_config, node_config.get_storage_dir(), genesis_transaction, rollback_flag);
        Self {
            runtime,
            event_handle: Some(event_handle),
        }
    }

    pub fn event_handle(&mut self, executor: TaskExecutor) {
        match self.event_handle.take() {
            Some(mut handle) => {
//                println!("========5555==========");
//                //mint
                handle.mint(executor.clone());
//                println!("========6666==========");
//
//                //msg
                handle.event_process(executor.clone());
//                println!("========7777==========");
//
//                //save
                handle.chain_manager.borrow_mut().save_block(executor.clone());
//                println!("========8888==========");
//
//                //sync
                handle.sync_block_msg(executor.clone());
//                println!("========9999==========");

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