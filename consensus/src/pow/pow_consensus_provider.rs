use failure::prelude::*;
use config::config::NodeConfig;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto,
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
use std::convert::TryFrom;
use crypto::hash::CryptoHash;
use libra_types::crypto_proxies::ValidatorSigner;
use libra_types::account_address::AccountAddress;
use channel;
use atomic_refcell::AtomicRefCell;
use config::trusted_peers::ConsensusPeersConfig;
use prost_ext::MessageExt;

pub struct EventHandle {
    chain_vec: Arc<AtomicRefCell<Vec<HashValue>>>,
    block_store: Arc<ConsensusDB>,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
    author: AccountAddress,
    peers: ConsensusPeersConfig,
}

impl EventHandle {
    pub fn new(network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               author: AccountAddress,
               peers: ConsensusPeersConfig) -> Self {
        let chain_vec = Arc::new(AtomicRefCell::new(vec![]));
        let block_store = Arc::new(ConsensusDB::new("/tmp/block.db"));
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        EventHandle { chain_vec, block_store, network_sender, network_events: Some(network_events), txn_manager, state_computer, self_sender, self_receiver: Some(self_receiver), author, peers }
    }

    async fn process_new_block_msg(chain: Arc<AtomicRefCell<Vec<HashValue>>>, db: Arc<ConsensusDB>, new_block: BlockProto) -> failure::Result<()> {
        let block = Block::try_from(new_block).expect("parse block pb err.");
        let mut blocks: Vec<Block<Vec<SignedTransaction>>> = Vec::new();
        blocks.push(block.clone());
        db.save_blocks_and_quorum_certificates(blocks, Vec::new());
        let mut chain_lock = chain.borrow_mut();
        let len = chain_lock.len();
        if len > 0 {
            let latest_root_hash = chain_lock.get(len - 1).expect("chain vec is empty.");
            if latest_root_hash.clone() == block.parent_id() {
                chain_lock.push(block.id());
            }
        } else {
            //TODO:genesis
        }
        drop(chain_lock);
        Ok(())
    }

    pub fn event_process(&mut self, executor: TaskExecutor) {
        println!("----->>>{}<<<-----", "4444");
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
        let chain = self.chain_vec.clone();

        let fut = async move {
            println!("----->>>{}<<<-----", "2222");
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
                            ConsensusMsg_oneof::NewBlock(new_block) => Self::process_new_block_msg(chain.clone(), block_db.clone(), new_block).await,
                            _ => {
                                warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                continue;
                            }
                        };
                    }
                    _ => {}
                }
            }
        };
        executor.spawn(fut);
    }

    pub fn mint(&self, executor: TaskExecutor) {
        println!("----->>>{}<<<-----", "3333");
        let mint_txn_manager = self.txn_manager.clone();
        let mint_chain_vec = self.chain_vec.clone();
        let mint_state_computer = self.state_computer.clone();
        let mut mint_sender = self.self_sender.clone();
        let mint_peers = self.peers.clone();
        let mut mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let mint_fut = async move {
            loop {
                println!("----->>>{}<<<-----", "111111");
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
                        let block_id = HashValue::random();

                        let len = mint_chain_vec.borrow().len();
                        let parent_block_id = mint_chain_vec.borrow().get(len - 1).unwrap().clone();

                        let state_compute_result = block_on(mint_state_computer.compute(parent_block_id, block_id, &txns)).unwrap();

                        let mut write_lock = mint_chain_vec.borrow_mut();
                        let tmp_len = write_lock.len();
                        debug!("block height: {:?}, latest block height: {:?}, new block hash: {:?}", len, tmp_len, parent_block_id);
                        if len == tmp_len {
                            write_lock.push(block_id);
                        }
                        drop(write_lock);

                        if len == tmp_len {
                            // commit
                            let info = LedgerInfo::new(state_compute_result.version(), state_compute_result.root_hash(), HashValue::random(), block_id, 0, u64::max_value(), None);
                            let ledger_hash = info.hash();
                            let info_sign = LedgerInfoWithSignatures::new(info, BTreeMap::new());

                            let quorum_cert = QuorumCert::new(
                                VoteData::new(block_id, ledger_hash, 0, parent_block_id, 0),
                                info_sign.clone(),
                            );
                            let block = Block::new_internal(txns.clone(), 0, 0, 0, quorum_cert, &ValidatorSigner::from_int(1));
                            let msg = ConsensusMsg {
                                message: Some(ConsensusMsg_oneof::NewBlock(block.into())),
                            };
                            //TODO:1. send block
//                            let msg = Ok(Event::Message((mint_author, msg)));
//                            if let Err(err) = mint_sender.send(msg).await {
////                                error!("Error delivering a self proposal: {:?}", err);
////                            }

                            let msg_raw = msg.to_bytes().unwrap();
                            for peer in mint_peers.get_validator_verifier().get_ordered_account_addresses() {
                                if let Err(err) = mint_network_sender.send_bytes(peer, msg_raw.clone()).await {
                                    error!(
                                        "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
                                        peer, err, msg
                                    );
                                }
                            }

                            // 2. commit block
                            block_on(mint_state_computer.commit(info_sign)).unwrap();

                            let keep = TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED));
                            //let state_compute = StateComputeResult { executed_state: ExecutedState::state_for_genesis(), compute_status: vec![keep] };

                            // 3. remove from mem pool
                            mint_txn_manager.commit_txns(&txns, &state_compute_result, u64::max_value());
                        }
                    }
                    _ => {}
                }

                let mut r = rand::thread_rng();
                r.gen::<i32>();
                let sleep_time = r.gen_range(0, 5);
                println!("---->{}", sleep_time);
                thread::sleep(Duration::from_secs(sleep_time));
            }
        };
        println!("----->>>{}<<<-----", "5555");
        executor.spawn(mint_fut);
    }
}

pub struct PowConsensusProvider {
    event_handle: Option<EventHandle>
}

impl PowConsensusProvider {
    pub fn new(node_config: &mut NodeConfig,
               network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               mempool_client: Arc<MempoolClient>,
               executor: Arc<Executor<MoveVM>>,
               synchronizer_client: Arc<StateSyncClient>) -> Self {
        let txn_manager = Arc::new(MempoolProxy::new(mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client.clone()));

        let peer_id_str = node_config
            .get_validator_network_config()
            .unwrap()
            .peer_id
            .clone();
        let author =
            AccountAddress::try_from(peer_id_str).expect("Failed to parse peer id of a validator");

        let peers_config = node_config
            .consensus
            .consensus_peers.clone();
        let event_handle = EventHandle::new(network_sender, network_events, txn_manager, state_computer, author, peers_config);
        Self {
            event_handle: Some(event_handle)
        }
    }

    pub fn event_handle(&mut self, executor: TaskExecutor) {
        match self.event_handle.take() {
            Some(mut handle) => {
                //mint
                handle.mint(executor.clone());

                //msg
                handle.event_process(executor);
            }
            _ => {}
        }
    }
}

impl ConsensusProvider for PowConsensusProvider {
    fn start(&mut self) -> Result<()> {
        let runtime = runtime::Builder::new()
            .name_prefix("pow-consensus-")
            .build()
            .expect("Failed to create Tokio runtime!");
        let executor = runtime.executor();
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