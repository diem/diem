use crate::chained_bft::consensusdb::{BlockIndex, ConsensusDB};
use crate::pow::payload_ext::BlockPayloadExt;
use crate::state_replication::{StateComputer, TxnManager};
use atomic_refcell::AtomicRefCell;
use consensus_types::block::Block;
use futures::compat::Future01CompatExt;
use futures::{channel::mpsc, StreamExt};
use futures_locks::{Mutex, RwLock};
use libra_crypto::hash::{GENESIS_BLOCK_ID, PRE_GENESIS_BLOCK_ID};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::transaction::SignedTransaction;
use libra_types::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::TaskExecutor;

pub struct ChainManager {
    block_cache_receiver: Option<mpsc::Receiver<Block<BlockPayloadExt>>>,
    block_store: Arc<ConsensusDB>,
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
    block_chain: Arc<RwLock<BlockChain>>,
    orphan_blocks: Arc<Mutex<HashMap<HashValue, Vec<HashValue>>>>, //key -> parent_block_id, value -> block_id
    rollback_flag: bool,
    author: AccountAddress,
}

impl ChainManager {
    pub fn new(
        block_cache_receiver: Option<mpsc::Receiver<Block<BlockPayloadExt>>>,
        block_store: Arc<ConsensusDB>,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
        rollback_flag: bool,
        author: AccountAddress,
    ) -> Self {
        let genesis_block_index = BlockIndex {
            id: *GENESIS_BLOCK_ID,
            parent_block_id: *PRE_GENESIS_BLOCK_ID,
        };
        let genesis_height = 0;
        let mut index_map = HashMap::new();
        index_map.insert(genesis_height, vec![genesis_block_index.clone()]);
        let mut hash_height_index = HashMap::new();
        hash_height_index.insert(*GENESIS_BLOCK_ID, (genesis_height, 0));
        let main_chain = AtomicRefCell::new(HashMap::new());
        main_chain
            .borrow_mut()
            .insert(genesis_height, genesis_block_index);
        let init_block_chain = BlockChain {
            height: genesis_height,
            indexes: index_map,
            hash_height_index,
            main_chain,
        };
        let block_chain = Arc::new(RwLock::new(init_block_chain));
        let orphan_blocks = Arc::new(Mutex::new(HashMap::new()));
        ChainManager {
            block_cache_receiver,
            block_store,
            txn_manager,
            state_computer,
            block_chain,
            orphan_blocks,
            rollback_flag,
            author,
        }
    }

    pub fn _process_orphan_blocks(&self) {
        //TODO:orphan
    }

    pub fn save_block(&mut self, executor: TaskExecutor) {
        let block_db = self.block_store.clone();
        let block_chain = self.block_chain.clone();
        let orphan_blocks = self.orphan_blocks.clone();
        let mut block_cache_receiver = self
            .block_cache_receiver
            .take()
            .expect("block_cache_receiver is none.");
        let txn_manager = self.txn_manager.clone();
        let state_computer = self.state_computer.clone();
        let rollback_flag = self.rollback_flag;
        let author = self.author.clone();
        let chain_fut = async move {
            loop {
                ::futures::select! {
                    block = block_cache_receiver.select_next_some() => {
                        //TODO:Verify block

                        // 2. compute with state_computer
                        let mut payload = match block.payload() {
                            Some(p) => p.get_txns(),
                            None => vec![],
                        };

                        // Pre compute
                        // 1. orphan block
                        let parent_block_id = block.parent_id();
                        let block_index = BlockIndex { id: block.id(), parent_block_id };
                        let mut chain_lock = block_chain.write().compat().await.unwrap();
                        let mut save_flag = false;
                        if chain_lock.block_exist(&parent_block_id) {
                            let mut commit_txn_vec = Vec::<SignedTransaction>::new();
                            // 2. find ancestors
                            let (ancestors, pre_block_index) = chain_lock.find_ancestor_until_main_chain(&parent_block_id).expect("find ancestors err.");

                            // 3. find blocks
                            let blocks = block_db.get_blocks_by_hashs::<BlockPayloadExt>(ancestors).expect("find blocks err.");

                            for b in blocks {
                                let mut tmp_txns = match b.payload() {
                                    Some(t) => t.get_txns(),
                                    None => vec![],
                                };
                                commit_txn_vec.append(&mut tmp_txns);
                            }

                            let pre_compute_grandpa_block_id = pre_block_index.parent_block_id;
                            let pre_compute_parent_block_id = pre_block_index.id;
                            commit_txn_vec.append(&mut payload);

                            // 4. call pre_compute
                            match state_computer.compute_by_hash(pre_compute_grandpa_block_id, pre_compute_parent_block_id, block.id(), &commit_txn_vec).await {
                                Ok(processed_vm_output) => {
                                    let executed_trees = processed_vm_output.executed_trees();
                                    let state_id = executed_trees.state_root();
                                    let txn_accumulator_hash = executed_trees.txn_accumulator().root_hash();
                                    let txn_len = executed_trees.version().expect("version err.");

                                    if state_id == block.quorum_cert().certified_block().executed_state_id() && txn_accumulator_hash == block.quorum_cert().ledger_info().ledger_info().transaction_accumulator_hash() {
                                        save_flag = true;
                                    } else {
                                        warn!("Peer id {:?}, Drop block {:?}, parent_block_id {:?}, grandpa_block_id {:?}", author, block.id(), pre_compute_parent_block_id, pre_compute_grandpa_block_id);
                                    }
                                }
                                Err(e) => {error!("{:?}", e)},
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
                                match old {
                                    Some(old_root) => {//update main chain
                                        let mut main_chain_indexes:Vec<&HashValue> = Vec::new();
                                        let height = chain_lock.longest_chain_height();

                                        if (rollback_flag && height > 2) || old_root != parent_block_id {//rollback
                                            let (rollback_vec, mut commit_vec) = if rollback_flag {
                                                (vec![&old_root], vec![&old_root])
                                            } else {
                                                chain_lock.find_ancestor(&old_root, &parent_block_id).expect("find ancestor err.")
                                            };
                                            let rollback_len = rollback_vec.len();
                                            let ancestor_block_id = chain_lock.find_index_by_block_hash(rollback_vec.get(rollback_len - 1).expect("latest_block_id err.")).expect("block index is none err.").parent_block_id;
                                            let ancestor_block_id = chain_lock.find_index_by_block_hash(&ancestor_block_id).expect("block index is none err.").parent_block_id;
                                            //1. reset executor
                                            state_computer.rollback(ancestor_block_id).await.expect("rollback failed.");
                                            info!("rollback[ old root : {:?} , ancestor block id : {:?}]", old_root, ancestor_block_id);

                                            //2. add txn to mempool

                                            //3. commit
                                            for commit in commit_vec.iter().rev() {
                                                // 1. query block
                                                let commit_block = block_db.get_block_by_hash::<BlockPayloadExt>(commit).expect("block not find in database err.");
                                                let grandpa_block_id = chain_lock.find_index_by_block_hash(&commit_block.parent_id()).expect("block index is none err.").parent_block_id;
                                                // 2. commit block
                                                Self::execut_and_commit_block(block_db.clone(), grandpa_block_id, commit_block, txn_manager.clone(), state_computer.clone()).await;
                                            }

                                            // 4. update main chain
                                            main_chain_indexes.append(&mut commit_vec);
                                        }

                                        //4.save latest block
                                        let id = block.id();
                                        let grandpa_block_id = chain_lock.find_index_by_block_hash(&block.parent_id()).expect("block index is none err.").parent_block_id;
                                        Self::execut_and_commit_block(block_db.clone(), grandpa_block_id, block.clone(), txn_manager.clone(), state_computer.clone()).await;

                                        //5. update main chain
                                        main_chain_indexes.append(&mut vec![&id].to_vec());
                                        for hash in main_chain_indexes {
                                            let (h, b_i) = chain_lock.find_height_and_block_index(hash);
                                            chain_lock.update_main_chain(h, b_i);
                                            block_db.insert_block_index(h, b_i).expect("insert_block_index err.");
                                        }

                                        chain_lock.print_block_chain_root(author);
                                    }
                                    None => {
                                        // save block, not commit
                                        let mut blocks: Vec<Block<BlockPayloadExt>> = Vec::new();
                                        blocks.push(block.clone());
                                        let mut qcs = Vec::new();
                                        qcs.push(block.quorum_cert().clone());
                                        block_db.save_blocks_and_quorum_certificates(blocks, qcs).expect("save_blocks err.");
                                    }
                                }
                            }

                            drop(chain_lock);
                            debug!("save block drop chain lock");
                        }
                    }
                }
            }
        };

        executor.spawn(chain_fut);
    }

    async fn execut_and_commit_block(
        block_db: Arc<ConsensusDB>,
        grandpa_block_id: HashValue,
        block: Block<BlockPayloadExt>,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
    ) {
        // 2. compute with state_computer
        let payload = match block.payload() {
            Some(txns) => txns.get_txns(),
            None => vec![],
        };

        // 3. Query data from db
        let processed_vm_output = state_computer
            .compute_by_hash(grandpa_block_id, block.parent_id(), block.id(), &payload)
            .await
            .expect("compute block err.");

        // 3. remove tx from mempool
        if payload.len() > 0 {
            if let Err(e) = txn_manager
                .commit_txns(
                    &payload,
                    &processed_vm_output.state_compute_result(),
                    block.timestamp_usecs(),
                )
                .await
            {
                error!("Failed to notify mempool: {:?}", e);
            }
        }

        // 4. commit to state_computer
        if let Err(e) = state_computer
            .commit(
                vec![(payload, Arc::new(processed_vm_output))],
                block.quorum_cert().ledger_info().clone(),
            )
            .await
        {
            error!("Failed to commit block: {:?}", e);
        }

        // 5. save block
        let mut blocks: Vec<Block<BlockPayloadExt>> = Vec::new();
        blocks.push(block.clone());
        let mut qcs = Vec::new();
        qcs.push(block.quorum_cert().clone());
        block_db
            .save_blocks_and_quorum_certificates(blocks, qcs)
            .expect("save_blocks err.");
    }

    pub async fn _chain_height(&self) -> u64 {
        self.block_chain
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .longest_chain_height()
    }

    pub async fn chain_root(&self) -> HashValue {
        self.block_chain
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .root_hash()
    }

    pub async fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.block_chain
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .block_exist(block_hash)
    }

    pub async fn _get_block_index_by_height(&self, height: &u64) -> BlockIndex {
        self.block_chain
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .indexes
            .get(height)
            .unwrap()[0]
            .clone()
    }

    pub async fn chain_height_and_root(&self) -> (u64, BlockIndex) {
        self.block_chain
            .clone()
            .read()
            .compat()
            .await
            .unwrap()
            .chain_height_and_root()
    }
}

#[derive(Clone)]
pub struct BlockChain {
    pub height: u64,
    pub indexes: HashMap<u64, Vec<BlockIndex>>,
    pub hash_height_index: HashMap<HashValue, (u64, usize)>,
    pub main_chain: AtomicRefCell<HashMap<u64, BlockIndex>>,
}

impl BlockChain {
    pub fn longest_chain_height(&self) -> u64 {
        self.chain_height_and_root().0
    }

    pub fn find_height_index_by_block_hash(&self, block_hash: &HashValue) -> Option<&(u64, usize)> {
        self.hash_height_index.get(block_hash)
    }

    pub fn print_block_chain_root(&self, peer_id: PeerId) {
        let height = self.main_chain.borrow().len() as u64;
        for index in 0..height {
            info!(
                "Main Chain Block, PeerId: {:?} , Height: {} , Block Root: {:?}",
                peer_id,
                index,
                self.main_chain
                    .borrow()
                    .get(&index)
                    .expect("print block err.")
            );
        }
    }

    fn find_index_by_block_hash(&self, block_hash: &HashValue) -> Option<&BlockIndex> {
        match self.find_height_index_by_block_hash(block_hash) {
            Some((height, index)) => {
                let tmp_index = self.indexes.get(height).expect("block hash not exist.");
                tmp_index.get(index.clone())
            }
            None => None,
        }
    }

    pub fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.hash_height_index.contains_key(block_hash)
    }

    pub fn root_hash(&self) -> HashValue {
        self.chain_height_and_root().1.id.clone()
    }

    pub fn chain_height_and_root(&self) -> (u64, BlockIndex) {
        let height = self.height;
        let block_index = self.indexes.get(&height).expect("get root hash err.")[0].clone();
        (height, block_index)
    }

    pub fn connect_block(&mut self, block_index: BlockIndex) -> (bool, Option<HashValue>) {
        let parent = self.find_height_index_by_block_hash(&block_index.parent_block_id);
        match parent {
            Some((parent_height, _parent_index)) => {
                let height = parent_height.clone();
                let current_block_id = block_index.id.clone();
                let (old, current_index) = if self.height == height {
                    let old_root_hash = self.root_hash();
                    let tmp = self.height + 1;
                    self.height = tmp;
                    self.indexes.insert(tmp, vec![block_index.clone()]);
                    (Some(old_root_hash), 0)
                } else {
                    let tmp_indexes = self
                        .indexes
                        .get_mut(&(height + 1))
                        .expect("height block index not exist.");
                    let current_index = tmp_indexes.len();
                    tmp_indexes.push(block_index);
                    (None, current_index)
                };

                self.hash_height_index
                    .insert(current_block_id, ({ height + 1 }, current_index));
                (false, old)
            }
            None => (true, None),
        }
    }

    pub fn update_main_chain(&self, height: &u64, block_index: &BlockIndex) {
        self.main_chain
            .borrow_mut()
            .insert(height.clone(), block_index.clone());
    }

    pub fn find_height_and_block_index(&self, hash: &HashValue) -> (&u64, &BlockIndex) {
        let (index, _) = self
            .find_height_index_by_block_hash(hash)
            .expect("block height not exist.");
        let block_index = self
            .find_index_by_block_hash(hash)
            .expect("block index not exist.");
        (index, block_index)
    }

    pub fn find_ancestor_until_main_chain(
        &self,
        hash: &HashValue,
    ) -> Option<(Vec<HashValue>, BlockIndex)> {
        let mut ancestors = vec![];
        let mut latest_hash = hash;
        let mut block_index = None;
        for _i in 0..100 {
            let (height, index) = match self.find_height_index_by_block_hash(latest_hash) {
                Some(h_i) => h_i,
                None => return None,
            };

            let tmp_index = self.indexes.get(height).expect("block hash not exist.");
            let tmp_block_index = tmp_index.get(index.clone());

            match tmp_block_index {
                Some(b_i) => {
                    let current_id = b_i.id;
                    latest_hash = &b_i.parent_block_id;
                    block_index = Some(b_i.clone());

                    if self
                        .main_chain
                        .borrow()
                        .get(height)
                        .expect("get block index from main chain err.")
                        .clone()
                        .id
                        == current_id
                    {
                        break;
                    } else {
                        ancestors.push(current_id);
                    }
                }
                None => return None,
            }
        }

        ancestors.reverse();
        Some((ancestors, block_index.expect("block_index is none.")))
    }

    fn find_ancestor(
        &self,
        first_hash: &HashValue,
        second_hash: &HashValue,
    ) -> Option<(Vec<&HashValue>, Vec<&HashValue>)> {
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

                                let ancestors = self.find_ancestor(
                                    &block_index_1.parent_block_id,
                                    &block_index_2.parent_block_id,
                                );
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
