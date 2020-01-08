use crate::pow::chain_manager::ChainManager;
use crate::pow::event_processor::EventProcessor;
use anyhow::Result;
use atomic_refcell::AtomicRefCell;
use channel;
use consensus_types::block_retrieval::{
    BlockRetrievalRequest, BlockRetrievalResponse, BlockRetrievalStatus,
};
use consensus_types::{block::Block, payload_ext::BlockPayloadExt};
use futures::compat::Future01CompatExt;
use futures::SinkExt;
use futures::{channel::mpsc, StreamExt};
use futures_locks::Mutex;
use libra_crypto::HashValue;
use libra_types::account_address::AccountAddress;
use libra_types::PeerId;
use network::{
    proto::{
        ConsensusMsg,
        ConsensusMsg_oneof::{self},
    },
    validator_network::{ConsensusNetworkSender, Event},
};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct SyncManager {
    author: AccountAddress,
    self_sender: channel::Sender<Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    block_cache_sender: mpsc::Sender<Block<BlockPayloadExt>>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
    sync_block_cache: Arc<Mutex<HashMap<PeerId, Vec<Block<BlockPayloadExt>>>>>,
    sync_block_height: Arc<AtomicU64>,
}

impl SyncManager {
    pub fn new(
        author: AccountAddress,
        self_sender: channel::Sender<Result<Event<ConsensusMsg>>>,
        network_sender: ConsensusNetworkSender,
        block_cache_sender: mpsc::Sender<Block<BlockPayloadExt>>,
        chain_manager: Arc<AtomicRefCell<ChainManager>>,
    ) -> Self {
        SyncManager {
            author,
            self_sender,
            network_sender,
            block_cache_sender,
            chain_manager,
            sync_block_cache: Arc::new(Mutex::new(HashMap::new())),
            sync_block_height: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn sync_block_msg(
        &self,
        executor: Handle,
        mut sync_block_receiver: mpsc::Receiver<(PeerId, BlockRetrievalResponse<BlockPayloadExt>)>,
        mut sync_signal_receiver: mpsc::Receiver<(PeerId, (u64, HashValue))>,
    ) {
        let sync_network_sender = self.network_sender.clone();
        let mut sync_block_cache_sender = self.block_cache_sender.clone();
        let self_peer_id = self.author.clone();
        let sync_self_sender = self.self_sender.clone();
        let chain_manager = self.chain_manager.clone();
        let max_height = self.sync_block_height.clone();
        let sync_block_cache = self.sync_block_cache.clone();

        let sync_fut = async move {
            loop {
                ::futures::select! {
                                    (peer_id, (height, root_hash)) = sync_signal_receiver.select_next_some() => {
                                        //1. sync data from latest block
                                        //TODO:timeout
                                        if max_height.load(Ordering::Relaxed) < height {
                                            max_height.store(height, Ordering::Relaxed);
                                            let sync_block_req_msg = Self::sync_block_req(root_hash);

                                            EventProcessor::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                                        }
                                    },
                                    (peer_id, sync_block_resp) = sync_block_receiver.select_next_some() => {
                                        // 2. save data to cache
                                        let status = sync_block_resp.status();
                                        let mut blocks = sync_block_resp.blocks();

                                        let mut end_flag = false;
                                        let mut end_block_hash = None;
                                        let mut sync_block_cache_lock = sync_block_cache.clone().lock().compat().await.unwrap();
                                        if blocks.len() > 0 {
                                            if !sync_block_cache_lock.contains_key(&peer_id) {
                                                let block_vec = Vec::new();
                                                sync_block_cache_lock.insert(peer_id, block_vec);
                                            }
                                            for block in blocks {
                                                let tmp_hash = block.id();
                                                if chain_manager.borrow().block_exist(&tmp_hash).await {
                                                    end_flag = true;
                                                    break;
                                                };

                                                // add to sync_block_cache
                                                end_block_hash = Some(block.parent_id());
                                                sync_block_cache_lock.get_mut(&peer_id).expect("peer block not exist.").push(block.clone());
                                            }
                                        }

                                        if end_flag {
                                            let mut block_vec = sync_block_cache_lock.remove(&peer_id).expect("peer block not exist.");
                                            block_vec.reverse();
                                            for b in block_vec {
                                                sync_block_cache_sender.send(b).await.expect("send block err.");
                                            }
                                        } else {
                                            match status {
                                                BlockRetrievalStatus::Succeeded => {
                                                    let sync_block_req_msg = Self::sync_block_req(end_block_hash.unwrap());
                                                    EventProcessor::send_consensus_msg(peer_id, &mut sync_network_sender.clone(), self_peer_id.clone(), &mut sync_self_sender.clone(), sync_block_req_msg).await;
                                                }
                                                _ => {
                //                                    BlockRetrievalStatus::IdNotFound
                //                                    BlockRetrievalStatus::NotEnoughBlocks
                                                    let _ = sync_block_cache_lock.remove(&peer_id);
                                                }
                                            };
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

    fn sync_block_req(hash: HashValue) -> ConsensusMsg {
        let num_blocks = 10;

        let req = BlockRetrievalRequest::new(hash, num_blocks)
            .try_into()
            .expect("BlockRetrievalRequest pb err.");

        ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RequestBlock(req)),
        }
    }
}
