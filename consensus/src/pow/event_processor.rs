use crate::chained_bft::consensusdb::ConsensusDB;
use crate::counters;
use crate::pow::chain_manager::ChainManager;
use crate::pow::mint_manager::MintManager;
use crate::pow::payload_ext::BlockPayloadExt;
use crate::pow::sync_manager::SyncManager;
use crate::state_replication::{StateComputer, TxnManager};
use anyhow::{Error, Result};
use atomic_refcell::AtomicRefCell;
use channel;
use consensus_types::block::Block;
use consensus_types::block_retrieval::{
    BlockRetrievalRequest, BlockRetrievalResponse, BlockRetrievalStatus,
};
use cuckoo::consensus::{PowCuckoo, PowService, Proof};
use futures::channel::mpsc;
use futures::{stream::select, SinkExt, StreamExt, TryStreamExt};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_crypto::hash::CryptoHash;
use libra_crypto::hash::PRE_GENESIS_BLOCK_ID;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_prost_ext::MessageExt;
use libra_types::account_address::AccountAddress;
use libra_types::transaction::SignedTransaction;
use libra_types::PeerId;
use miner::types::{MineStateManager, CYCLE_LENGTH, MAX_EDGE};
use network::{
    proto::{
        Block as BlockProto, ConsensusMsg,
        ConsensusMsg_oneof::{self},
    },
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event},
};
use std::convert::TryInto;
use std::sync::Arc;
use std::{convert::TryFrom, path::PathBuf};
use storage_client::{StorageRead, StorageWrite};
use tokio::runtime::Handle;

pub struct EventProcessor {
    block_cache_sender: mpsc::Sender<Block<BlockPayloadExt>>,
    block_store: Arc<ConsensusDB>,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    self_sender: channel::Sender<Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<Result<Event<ConsensusMsg>>>>,
    author: AccountAddress,

    //sync
    sync_block_sender: mpsc::Sender<(PeerId, BlockRetrievalResponse<BlockPayloadExt>)>,
    sync_signal_sender: mpsc::Sender<(PeerId, (u64, HashValue))>,
    pub sync_manager: Arc<AtomicRefCell<SyncManager>>,

    pow_srv: Arc<dyn PowService>,
    pub chain_manager: Arc<AtomicRefCell<ChainManager>>,
    pub mint_manager: Arc<AtomicRefCell<MintManager>>,
}

impl EventProcessor {
    pub fn new(
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
        author: AccountAddress,
        storage_dir: PathBuf,
        rollback_flag: bool,
        mine_state: MineStateManager,
        read_storage: Arc<dyn StorageRead>,
        write_storage: Arc<dyn StorageWrite>,
        pri_key: Ed25519PrivateKey,
    ) -> Self {
        let (block_cache_sender, block_cache_receiver) = mpsc::channel(10);

        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        //sync
        let (sync_block_sender, sync_block_receiver) = mpsc::channel(10);
        let (sync_signal_sender, sync_signal_receiver) = mpsc::channel(1024);

        let block_store = Arc::new(ConsensusDB::new(storage_dir));
        let chain_manager = Arc::new(AtomicRefCell::new(ChainManager::new(
            Some(block_cache_receiver),
            Arc::clone(&block_store),
            txn_manager.clone(),
            state_computer.clone(),
            rollback_flag,
            author.clone(),
            read_storage,
            write_storage,
        )));

        let sync_manager = Arc::new(AtomicRefCell::new(SyncManager::new(
            author.clone(),
            self_sender.clone(),
            network_sender.clone(),
            block_cache_sender.clone(),
            Some(sync_block_receiver),
            Some(sync_signal_receiver),
            chain_manager.clone(),
        )));
        let pow_srv = Arc::new(PowCuckoo::new(MAX_EDGE, CYCLE_LENGTH));
        let mint_manager = Arc::new(AtomicRefCell::new(MintManager::new(
            txn_manager.clone(),
            state_computer.clone(),
            network_sender.clone(),
            author.clone(),
            self_sender.clone(),
            block_store.clone(),
            pow_srv.clone(),
            chain_manager.clone(),
            mine_state,
            pri_key,
        )));
        EventProcessor {
            block_cache_sender,
            block_store,
            network_sender,
            network_events: Some(network_events),
            self_sender,
            self_receiver: Some(self_receiver),
            author,
            sync_block_sender,
            sync_signal_sender,
            sync_manager,
            pow_srv,
            chain_manager,
            mint_manager,
        }
    }

    pub fn event_process(&mut self, executor: Handle) {
        let network_events = self
            .network_events
            .take()
            .expect("[consensus] Failed to start; network_events stream is already taken")
            .map_err(Into::<Error>::into);

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

        let sync_signal_sender = self.sync_signal_sender.clone();
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
                                let block: Block<BlockPayloadExt> =
                                    Block::try_from(new_block).expect("parse block pb err.");

                                debug!(
                                    "Self is {:?}, Peer Id is {:?}, Block Id is {:?}, height {}",
                                    self_peer_id,
                                    peer_id,
                                    block.id(),
                                    block.round()
                                );

                                let payload = block.payload().expect("payload is none");
                                let verify = pow_srv.verify(
                                    block
                                        .quorum_cert()
                                        .ledger_info()
                                        .ledger_info()
                                        .hash()
                                        .as_ref(),
                                    payload.nonce,
                                    Proof {
                                        solve: payload.solve.clone(),
                                    },
                                );

                                if verify {
                                    if self_peer_id != peer_id {
                                        let (height, block_index) =
                                            chain_manager.borrow().chain_height_and_root().await;

                                        debug!(
                                            "Self is {:?}, height is {}, Peer Id is {:?}, Block Id is {:?}, verify {}, height {}",
                                            self_peer_id,
                                            height,
                                            peer_id,
                                            block.id(),
                                            verify,
                                            block.round()
                                        );

                                        if height < block.round()
                                            && block.parent_id() != block_index.id()
                                        {
                                            if let Err(err) = sync_signal_sender
                                                .clone()
                                                .send((peer_id, (block.round(), HashValue::zero())))
                                                .await
                                            {
                                                error!("send sync signal err: {:?}", err);
                                            }
                                        }

                                        //broadcast new block
                                        let block_pb =
                                            TryInto::<BlockProto>::try_into(block.clone())
                                                .expect("parse block err.");

                                        // send block
                                        let msg = ConsensusMsg {
                                            message: Some(ConsensusMsg_oneof::NewBlock(block_pb)),
                                        };
                                        Self::broadcast_consensus_msg_but(
                                            &mut network_sender,
                                            false,
                                            self_peer_id,
                                            &mut self_sender,
                                            msg,
                                            vec![peer_id],
                                        )
                                        .await;
                                    }

                                    if let Err(err) = (&mut block_cache_sender).send(block).await {
                                        error!("send new block err: {:?}", err);
                                    }
                                } else {
                                    warn!(
                                        "block : {:?} from : {:?} verify fail.",
                                        block.id(),
                                        peer_id
                                    );
                                }
                            }
                            ConsensusMsg_oneof::RequestBlock(req_block) => {
                                let block_req =
                                    BlockRetrievalRequest::try_from(req_block).expect("parse err.");
                                if block_req.num_blocks() > 0 {
                                    let mut blocks = vec![];
                                    let mut latest_block =
                                        if block_req.block_id() != HashValue::zero() {
                                            Some(block_req.block_id())
                                        } else {
                                            None
                                        };
                                    let mut not_exist_flag = false;
                                    for _i in 0..block_req.num_blocks() {
                                        let block = match latest_block {
                                            Some(child_hash) => {
                                                if child_hash == *PRE_GENESIS_BLOCK_ID {
                                                    break;
                                                }

                                                let child = block_db
                                                    .get_block_by_hash::<BlockPayloadExt>(
                                                        &child_hash,
                                                    );
                                                match child {
                                                    Some(c) => c,
                                                    None => {
                                                        not_exist_flag = true;
                                                        break;
                                                    }
                                                }
                                            }
                                            None => block_db
                                                .get_block_by_hash::<BlockPayloadExt>(
                                                    &chain_manager.borrow().chain_root().await,
                                                )
                                                .expect("root not exist"),
                                        };

                                        latest_block = Some(block.parent_id());
                                        blocks.push(block.into());

                                        if latest_block.unwrap() == *PRE_GENESIS_BLOCK_ID {
                                            break;
                                        }
                                    }

                                    let status = if not_exist_flag {
                                        BlockRetrievalStatus::IdNotFound
                                    } else {
                                        if (blocks.len() as u64) == block_req.num_blocks() {
                                            BlockRetrievalStatus::Succeeded
                                        } else {
                                            BlockRetrievalStatus::NotEnoughBlocks
                                        }
                                    };

                                    let resp_block = BlockRetrievalResponse::new(status, blocks);
                                    let resp_block_msg = ConsensusMsg {
                                        message: Some(ConsensusMsg_oneof::RespondBlock(
                                            resp_block.try_into().expect("into err."),
                                        )),
                                    };

                                    Self::send_consensus_msg(
                                        peer_id,
                                        &mut network_sender.clone(),
                                        self_peer_id.clone(),
                                        &mut self_sender.clone(),
                                        resp_block_msg,
                                    )
                                    .await;
                                }
                            }
                            ConsensusMsg_oneof::RespondBlock(resp_block) => {
                                let block_resp = BlockRetrievalResponse::try_from(resp_block)
                                    .expect("parse err.");
                                if let Err(err) =
                                    sync_block_sender.send((peer_id, block_resp)).await
                                {
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
                    Event::RpcRequest((peer_id, _msg, _callback)) => {
                        debug!("RpcRequest from {:?} ", peer_id);
                    }
                    Event::NewPeer(peer_id) => {
                        debug!("Peer {:?} connected111", peer_id);
                    }
                    Event::LostPeer(peer_id) => {
                        debug!("Peer {:?} disconnected", peer_id);
                    }
                }
            }
        };
        executor.spawn(fut);
    }

    pub async fn broadcast_consensus_msg(
        network_sender: &mut ConsensusNetworkSender,
        self_flag: bool,
        self_peer_id: PeerId,
        self_sender: &mut channel::Sender<Result<Event<ConsensusMsg>>>,
        msg: ConsensusMsg,
    ) {
        Self::broadcast_consensus_msg_but(
            network_sender,
            self_flag,
            self_peer_id,
            self_sender,
            msg,
            vec![],
        )
        .await;
    }

    pub async fn broadcast_consensus_msg_but(
        network_sender: &mut ConsensusNetworkSender,
        self_flag: bool,
        self_peer_id: PeerId,
        self_sender: &mut channel::Sender<Result<Event<ConsensusMsg>>>,
        msg: ConsensusMsg,
        ignore_peers: Vec<PeerId>,
    ) {
        if self_flag {
            let event_msg = Ok(Event::Message((self_peer_id, msg.clone())));
            if let Err(err) = self_sender.send(event_msg).await {
                error!("Error delivering a self proposal: {:?}", err);
            }
        }
        let msg_raw = msg.to_bytes().unwrap();
        if let Err(err) = network_sender
            .broadcast_bytes(msg_raw.clone(), ignore_peers)
            .await
        {
            error!(
                "Error broadcasting proposal  error: {:?}, msg: {:?}",
                err, msg
            );
        }
    }

    pub async fn send_consensus_msg(
        send_peer_id: PeerId,
        network_sender: &mut ConsensusNetworkSender,
        self_peer_id: PeerId,
        self_sender: &mut channel::Sender<Result<Event<ConsensusMsg>>>,
        msg: ConsensusMsg,
    ) {
        if send_peer_id == self_peer_id {
            let event_msg = Ok(Event::Message((self_peer_id, msg.clone())));
            if let Err(err) = self_sender.send(event_msg).await {
                error!("Error delivering a self proposal: {:?}", err);
            }
        } else {
            if let Err(err) = network_sender.send_to(send_peer_id, msg.clone()).await {
                error!(
                    "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
                    send_peer_id, err, msg
                );
            }
        }
    }
}
