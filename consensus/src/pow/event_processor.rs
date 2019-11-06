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
use crate::pow::sync_manager::{SyncManager, BlockRetrievalResponse};
use crate::pow::mint_manager::MintManager;

pub struct EventProcessor {
    block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
    block_store: Arc<ConsensusDB>,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
    author: AccountAddress,

    //sync
    sync_block_sender: mpsc::Sender<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>,
    sync_signal_sender: mpsc::Sender<(PeerId, u64)>,
    pub sync_manager: Arc<AtomicRefCell<SyncManager>>,

    genesis_txn: Vec<SignedTransaction>,
    pow_srv: Arc<PowService>,
    pub chain_manager: Arc<AtomicRefCell<ChainManager>>,

    pub mint_manager: Arc<MintManager>,
}

impl EventProcessor {
    pub fn new(network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               author: AccountAddress, storage_dir: PathBuf,
               genesis_txn: Transaction, rollback_flag: bool) -> Self {
        let (block_cache_sender, block_cache_receiver) = mpsc::channel(10);

        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        //sync
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(1024);

        let mint_block_vec = Arc::new(AtomicRefCell::new(vec![*GENESIS_BLOCK_ID]));
        let block_store = Arc::new(ConsensusDB::new(storage_dir));

        let pow_srv = Arc::new(PowCuckoo::new(6, 8));

        let genesis_txn_vec = match genesis_txn {
            Transaction::UserTransaction(signed_txn) => {
                vec![signed_txn].to_vec()
            },
            _ => {vec![]}
        };

        let chain_manager = Arc::new(AtomicRefCell::new(ChainManager::new(Some(block_cache_receiver), Arc::clone(&block_store),
                                                                          txn_manager.clone(), state_computer.clone(), genesis_txn_vec.clone(), rollback_flag)));

        let sync_manager = Arc::new(AtomicRefCell::new(SyncManager::new(author.clone(), self_sender.clone(), network_sender.clone(),
                                                                        block_cache_sender.clone(), Some(sync_block_receiver),
                                                                        Some(sync_signal_receiver), chain_manager.clone())));

        let mint_manager = Arc::new(MintManager::new(txn_manager.clone(), state_computer.clone(), block_cache_sender.clone(), network_sender.clone(), author.clone(),
                                                     self_sender.clone(), block_store.clone(), pow_srv.clone(), genesis_txn_vec.clone(), chain_manager.clone()));

        EventProcessor {
            block_cache_sender,
            block_store,
            network_sender,
            network_events: Some(network_events),
            txn_manager,
            state_computer,
            self_sender,
            self_receiver: Some(self_receiver),
            author,
            sync_block_sender,
            sync_signal_sender,
            sync_manager,
            genesis_txn:genesis_txn_vec,
            pow_srv,
            chain_manager,
            mint_manager,
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
                        debug!("RpcRequest from {:?} ", peer_id);
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

    pub async fn broadcast_consensus_msg(network_sender: &mut ConsensusNetworkSender, self_flag: bool,
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
    }

    pub async fn send_consensus_msg(send_peer_id: PeerId, network_sender: &mut ConsensusNetworkSender, self_peer_id: PeerId,
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
}