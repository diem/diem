use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::block_tree::{BlockTree, CommitData};
use crate::pow::payload_ext::BlockPayloadExt;
use crate::state_replication::{StateComputer, TxnManager};
use atomic_refcell::AtomicRefCell;
use consensus_types::block::Block;
use futures::compat::Future01CompatExt;
use futures::{channel::mpsc, StreamExt};
use futures_locks::{Mutex, RwLock};
use itertools;
use libra_crypto::hash::PRE_GENESIS_BLOCK_ID;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::block_index::BlockIndex;
use libra_types::block_metadata::BlockMetadata;
use libra_types::transaction::TransactionStatus;
use libra_types::transaction::TransactionToCommit;
use libra_types::transaction::{SignedTransaction, Transaction};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use storage_client::{StorageRead, StorageWrite};
use tokio::runtime::Handle;
use libra_types::transaction::TransactionStatus;

pub struct ChainManager {
    block_cache_receiver: Option<mpsc::Receiver<Block<BlockPayloadExt>>>,
    block_store: Arc<ConsensusDB>,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
    block_tree: Arc<RwLock<BlockTree>>,
    orphan_blocks: Arc<Mutex<HashMap<HashValue, Vec<HashValue>>>>, //key -> parent_block_id, value -> block_id
    author: AccountAddress,
    read_storage: Arc<dyn StorageRead>,
}

impl ChainManager {
    pub fn new(
        block_cache_receiver: Option<mpsc::Receiver<Block<BlockPayloadExt>>>,
        block_store: Arc<ConsensusDB>,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
        rollback_mode: bool,
        author: AccountAddress,
        read_storage: Arc<dyn StorageRead>,
        write_storage: Arc<dyn StorageWrite>,
    ) -> Self {
        let genesis_block: Block<BlockPayloadExt> = Block::make_genesis_block();
        let genesis_id = genesis_block.id();
        let genesis_block_index = BlockIndex::new(&genesis_id, &PRE_GENESIS_BLOCK_ID);
        let genesis_height = 0;

        //init block_store
        let mut genesis_qcs = Vec::new();
        genesis_qcs.push(genesis_block.quorum_cert().clone());
        block_store
            .save_blocks_and_quorum_certificates(vec![genesis_block], genesis_qcs)
            .expect("save blocks failed.");

        //init main chain
        let mut index_map = HashMap::new();
        index_map.insert(genesis_height, vec![genesis_block_index.clone()]);
        let mut hash_height_index = HashMap::new();
        hash_height_index.insert(genesis_id, (genesis_height, 0));
        let main_chain = AtomicRefCell::new(HashMap::new());
        main_chain
            .borrow_mut()
            .insert(genesis_height, genesis_block_index);

        //orphan block
        let orphan_blocks = Arc::new(Mutex::new(HashMap::new()));

        //block tree
        let block_tree = Arc::new(RwLock::new(BlockTree::new(
            write_storage,
            txn_manager,
            rollback_mode,
        )));
        ChainManager {
            block_cache_receiver,
            block_store,
            state_computer,
            block_tree,
            orphan_blocks,
            author,
            read_storage,
        }
    }

    pub fn _process_orphan_blocks(&self) {
        //TODO:orphan
    }

    pub fn save_block(&mut self, executor: Handle) {
        let block_db = self.block_store.clone();
        let orphan_blocks = self.orphan_blocks.clone();
        let mut block_cache_receiver = self
            .block_cache_receiver
            .take()
            .expect("block_cache_receiver is none.");
        let state_computer = self.state_computer.clone();
        let author = self.author.clone();
        let block_tree = self.block_tree.clone();
        let _read_storage = self.read_storage.clone();
        let chain_fut = async move {
            loop {
                ::futures::select! {
                block = block_cache_receiver.select_next_some() => {
                    let mut payload = match block.payload() {
                        Some(p) => p.get_txns(),
                        None => vec![],
                    };

                    // Pre compute
                    // 1. orphan block
                    let parent_block_id = block.parent_id();
                    let block_index = BlockIndex::new(&block.id(), &parent_block_id);
                    let aa = block.id().clone();
                    let mut chain_lock = block_tree.write().compat().await.unwrap();
                    if chain_lock.block_exist(&parent_block_id) {
                        // 2. find ancestors
                        let (ancestors, pre_block_index) = chain_lock.find_ancestor_until_main_chain(&parent_block_id).expect("find ancestors err.");
                        // 3. find blocks
                        let blocks = block_db.get_blocks_by_hashs::<BlockPayloadExt>(ancestors).expect("find blocks err.");

                        let mut commit_txn_vec = Vec::<(BlockMetadata, Vec<SignedTransaction>)>::new();
                        for b in blocks {
                            let mut tmp_txns = match b.payload() {
                                Some(t) => t.get_txns(),
                                None => vec![],
                            };

                            let miner_address = b.quorum_cert().commit_info().next_validator_set().expect("validator_set err.").payload().clone()[0].account_address();
                            let block_meta_data = BlockMetadata::new(b.parent_id().clone(), b.timestamp_usecs(), BTreeMap::new(), miner_address.clone());
                            commit_txn_vec.push((block_meta_data, tmp_txns));
                        }

                        let pre_compute_grandpa_block_id = pre_block_index.parent_id();
                        let pre_compute_parent_block_id = pre_block_index.id();
                        let miner_address = block.quorum_cert().commit_info().next_validator_set().expect("validator_set err.").payload().clone()[0].account_address();
                        let block_meta_data = BlockMetadata::new(parent_block_id.clone(), block.timestamp_usecs(), BTreeMap::new(), miner_address.clone());
                        commit_txn_vec.push((block_meta_data.clone(), payload.clone()));

                        // 4. call pre_compute
                        match state_computer.compute_by_hash(&pre_compute_grandpa_block_id, &parent_block_id, &block.id(), commit_txn_vec).await {
                            Ok(processed_vm_output) => {
                                let executed_trees = processed_vm_output.executed_trees();
                                let state_id = executed_trees.state_root();
                                let txn_accumulator_hash = executed_trees.txn_accumulator().root_hash();
                                let txn_len = executed_trees.version().expect("version err.");

                                if txn_accumulator_hash == block.quorum_cert().ledger_info().ledger_info().transaction_accumulator_hash() && state_id == block.quorum_cert().ledger_info().ledger_info().consensus_data_hash() {

                                let mut txn_vec = vec![Transaction::BlockMetadata(block_meta_data)];
                                txn_vec.extend(
                                    payload
                                        .iter()
                                        .map(|txn| Transaction::UserTransaction(txn.clone())),
                                );
                                let len = txn_vec.len();
                                let mut txn_data_list = vec![];
                                let total_len = processed_vm_output.transaction_data().len();

                                for i in 0..len {
                                    txn_data_list.push(processed_vm_output.transaction_data()[total_len - len + i].clone());
                                }

                                let mut txns_to_commit = vec![];
                                for (txn, txn_data) in itertools::zip_eq(txn_vec, txn_data_list) {
                                    if let TransactionStatus::Keep(_) = txn_data.status() {
                                        txns_to_commit.push(TransactionToCommit::new(
                                            txn,
                                            txn_data.account_blobs().clone(),
                                            txn_data.events().to_vec(),
                                            txn_data.gas_used(),
                                            txn_data.status().vm_status().major_status,
                                        ));
                                    }
                                }
                                let commit_len = txns_to_commit.len();
                                if (block.quorum_cert().ledger_info().ledger_info().commit_info().version() == txn_len) {
                                    let commit_data = CommitData {txns_to_commit,
                                        first_version: (txn_len - (commit_len as u64) + 1) as u64,
                                        ledger_info_with_sigs: Some(block.quorum_cert().ledger_info().clone())};

                                    chain_lock.add_block_info(&block.id(), &parent_block_id, block.timestamp_usecs(), processed_vm_output, commit_data).await.expect("add_block_info failed.");

                                    let mut blocks: Vec<Block<BlockPayloadExt>> = Vec::new();
                                    blocks.push(block.clone());
                                    let mut qcs = Vec::new();
                                    qcs.push(block.quorum_cert().clone());
                                    block_db.save_blocks_and_quorum_certificates(blocks, qcs).expect("save_blocks err.");

                                    chain_lock.print_block_chain_root(author);
                                } else {
                                    warn!("Peer id {:?}, Drop block {:?}, block version is {}, vm output version is {}", author, block.id(),
                                    block.quorum_cert().ledger_info().ledger_info().commit_info().version(), txn_len);
                                }
                            } else {
                                warn!("Peer id {:?}, Drop block {:?}, parent_block_id {:?}, grandpa_block_id {:?}", author, block.id(), parent_block_id, pre_compute_grandpa_block_id);
                            }
                        }
                        Err(e) => {error!("error: {:?}", e)},
                    }
                } else {
                    //save orphan block
                    let mut write_lock = orphan_blocks.lock().compat().await.unwrap();
                    write_lock.insert(block_index.parent_id(), vec![block_index.id()]);
                }
                    }
                    complete => {
                       break;
                   }
                }
            }
        };

        executor.spawn(chain_fut);
    }

    pub async fn chain_root(&self) -> HashValue {
        self.block_tree
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .root_hash()
    }

    pub async fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.block_tree
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .block_exist(block_hash)
    }

    pub async fn chain_height_and_root(&self) -> (u64, BlockIndex) {
        self.block_tree
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .chain_height_and_root()
    }
}
