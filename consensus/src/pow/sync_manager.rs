use failure::prelude::*;
use std::sync::Arc;
use futures_locks::{Mutex, RwLock};
use std::collections::HashMap;
use crypto::HashValue;
use libra_types::transaction::SignedTransaction;
use crate::chained_bft::consensusdb::{BlockIndex, ConsensusDB};
use atomic_refcell::AtomicRefCell;
use libra_types::PeerId;
use tokio::runtime::{self, TaskExecutor};
use crate::state_replication::{StateComputer, TxnManager};
use futures::{channel::mpsc, StreamExt};
use consensus_types::block::Block;
use {
    futures::{
        compat::Future01CompatExt,
        future::{self, FutureExt, TryFutureExt},
    },
};
use channel;
use logger::prelude::*;
use crypto::hash::{GENESIS_BLOCK_ID, PRE_GENESIS_BLOCK_ID};
use crate::pow::chain_manager::ChainManager;
use libra_types::account_address::AccountAddress;
use crate::pow::event_processor::EventProcessor;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, RequestBlock, RespondBlock, BlockRetrievalStatus,
    },
    validator_network::{Event, ConsensusNetworkSender}
};
use futures::SinkExt;
use crypto::hash::CryptoHash;

pub struct SyncManager {
    author: AccountAddress,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
    sync_block_receiver: Option<mpsc::Receiver<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>>,
    sync_signal_receiver: Option<mpsc::Receiver<(PeerId, u64)>>,
    sync_state_sender: mpsc::Sender<SyncState>,
    sync_state_receiver: Option<mpsc::Receiver<SyncState>>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
}

impl SyncManager {

    pub fn new(author: AccountAddress,
               self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
               network_sender: ConsensusNetworkSender,
               block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
               sync_block_receiver: Option<mpsc::Receiver<(PeerId, BlockRetrievalResponse<Vec<SignedTransaction>>)>>,
               sync_signal_receiver: Option<mpsc::Receiver<(PeerId, u64)>>,
               chain_manager: Arc<AtomicRefCell<ChainManager>>) -> Self {
        let (sync_state_sender, sync_state_receiver) = mpsc::channel(10);
        SyncManager{author, self_sender, network_sender, block_cache_sender, sync_block_receiver,
            sync_signal_receiver, sync_state_sender, sync_state_receiver:Some(sync_state_receiver), chain_manager}
    }

    pub fn sync_block_msg(&mut self, executor: TaskExecutor) {
        let mut sync_block_receiver = self.sync_block_receiver.take().expect("sync_block_receiver is none.");
        let mut sync_signal_receiver = self.sync_signal_receiver.take().expect("sync_signal_receiver is none.");
        let mut sync_network_sender = self.network_sender.clone();
        let mut sync_state_sender = self.sync_state_sender.clone();
        let mut sync_state_receiver = self.sync_state_receiver.take().expect("sync_state_receiver is none.");
        let mut sync_block_cache_sender = self.block_cache_sender.clone();
        let self_peer_id = self.author.clone();
        let mut sync_self_sender = self.self_sender.clone();
        let chain_manager = self.chain_manager.clone();

        let sync_fut = async move {
            loop {
                ::futures::select! {
                    (peer_id, height) = sync_signal_receiver.select_next_some() => {
                        //sync data from latest block
                        //TODO:timeout
                        let sync_block_req_msg = Self::sync_block_req(None);

                        EventProcessor::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                    },
                    (sync_state) = sync_state_receiver.select_next_some() => {
                        match sync_state {
                            SyncState::END => {
                            },
                            SyncState::GOON((peer_id, hash)) => {
                                let sync_block_req_msg = Self::sync_block_req(Some(hash));
                                EventProcessor::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
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

enum SyncState {
    GOON((PeerId, HashValue)),
    END,
}

pub struct BlockRetrievalResponse<T> {
    pub status: BlockRetrievalStatus,
    pub blocks: Vec<Block<T>>,
}