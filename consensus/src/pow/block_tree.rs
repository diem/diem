use crate::pow::payload_ext::genesis_id;
use crate::state_replication::TxnManager;
use atomic_refcell::AtomicRefCell;
use executor::ProcessedVMOutput;
use failure::prelude::*;
use libra_crypto::hash::PRE_GENESIS_BLOCK_ID;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::block_index::BlockIndex;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use libra_types::transaction::{SignedTransaction, Transaction, TransactionToCommit, Version};
use libra_types::PeerId;
use std::collections::{HashMap, LinkedList};
use std::sync::Arc;
use storage_client::StorageWrite;

pub type BlockHeight = u64;

///
/// ```text
///   Committed(B4) --> B5  -> B6  -> B7
///                |
///             B4'└--> B5' -> B6' -> B7'
///                            |
///                            └----> B7"
/// ```
/// height: B7 B7' B7"
/// tail_height: B4 B4'
pub struct BlockTree {
    height: BlockHeight,
    id_to_block: HashMap<HashValue, BlockInfo>,
    indexes: HashMap<BlockHeight, LinkedList<HashValue>>,
    main_chain: AtomicRefCell<HashMap<BlockHeight, BlockIndex>>,
    write_storage: Arc<dyn StorageWrite>,
    tail_height: BlockHeight,
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    rollback_mode: bool,
}

impl BlockTree {
    pub fn new(
        write_storage: Arc<dyn StorageWrite>,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        rollback_mode: bool,
    ) -> Self {
        // genesis block info
        let genesis_block_info = BlockInfo::genesis_block_info();
        let genesis_id = genesis_block_info.id();
        let genesis_height = genesis_block_info.height();

        // indexes
        let mut genesis_indexes = LinkedList::new();
        genesis_indexes.push_front(genesis_id.clone());
        let mut indexes = HashMap::new();
        indexes.insert(genesis_height, genesis_indexes);

        // main chain
        let main_chain = AtomicRefCell::new(HashMap::new());
        main_chain
            .borrow_mut()
            .insert(genesis_height, genesis_block_info.block_index().clone());

        // id to block
        let mut id_to_block = HashMap::new();
        id_to_block.insert(genesis_id.clone(), genesis_block_info);

        BlockTree {
            height: genesis_height,
            id_to_block,
            indexes,
            main_chain,
            write_storage,
            tail_height: genesis_height,
            txn_manager,
            rollback_mode,
        }
    }

    //    fn prune(&mut self) {
    //        let ct = 1000;
    //        if self.tail_height + ct < self.height {
    //            let times = self.height - self.tail_height - ct;
    //            for _i in 0..times {
    //                let tmp_height = self.tail_height;
    //                //1. indexes
    //                let tmp_indexes = self.indexes.remove(&tmp_height).expect("indexes is none.");
    //                //2. id_to_block
    //                for block_id in tmp_indexes {
    //                    self.id_to_block.remove(&block_id);
    //                }
    //                //3. tail_height
    //                self.tail_height = tmp_height + 1;
    //            }
    //        }
    //    }

    async fn add_block_info_inner(&mut self, new_block_info: BlockInfo, new_root: bool) {
        //4. new root, rollback, commit
        if new_root {
            let old_root = self.root_hash();

            //rollback
            if old_root != new_block_info.parent_id() {
                let (ancestors, pre_block_index) = self
                    .find_ancestor_until_main_chain(&new_block_info.parent_id())
                    .expect("find ancestor failed.");

                let rollback_block_id = pre_block_index.parent_id();

                info!(
                    "Rollback : Block Id {:?} , Rollback Id {:?}",
                    new_block_info.id(),
                    rollback_block_id
                );
                self.write_storage.rollback_by_block_id(rollback_block_id);

                // commit
                for ancestor in ancestors {
                    let block_info = self
                        .find_block_info_by_block_id(&ancestor)
                        .expect("ancestor block info is none.");
                    self.commit_block(block_info).await;
                }
            } else {
                if self.rollback_mode && (self.height - self.tail_height) > 2 {
                    //rollback mode
                    let block_info = self
                        .find_block_info_by_block_id(&new_block_info.parent_id())
                        .expect("Parent block info is none.");
                    let grandpa_id = block_info.parent_id();
                    info!(
                        "Rollback mode: Block Id {:?} , Parent Id {:?}, Grandpa Id {:?}",
                        new_block_info.id(),
                        new_block_info.parent_id(),
                        grandpa_id
                    );
                    self.write_storage.rollback_by_block_id(grandpa_id);

                    self.commit_block(block_info).await;
                }
            }

            // save self
            self.commit_block(&new_block_info).await;

            self.height = new_block_info.height();
            let mut hash_list = LinkedList::new();
            hash_list.push_front(new_block_info.id().clone());
            self.indexes.insert(new_block_info.height(), hash_list);
        } else {
            self.indexes
                .get_mut(&new_block_info.height())
                .unwrap()
                .push_back(new_block_info.id().clone());
        }

        //5. add new block info
        self.id_to_block
            .insert(new_block_info.id().clone(), new_block_info);
    }

    async fn commit_block(&self, block_info: &BlockInfo) {
        let timestamp_usecs = block_info.timestamp_usecs();
        let vm_output = block_info.output().expect("output is none.");
        let commit_data = block_info.commit_data().expect("commit_data is none.");
        let height = block_info.height();
        let block_index = block_info.block_index();
        // 1. remove tx from mempool
        if commit_data.txns_len() > 0 {
            let signed_txns = commit_data.signed_txns();
            let signed_txns_len = signed_txns.len();
            let txns_status_len = vm_output.state_compute_result().status().len();

            let mut txns_status = vec![];
            for i in 0..signed_txns_len {
                txns_status.push(
                    vm_output.state_compute_result().status()
                        [txns_status_len - signed_txns_len + i]
                        .clone(),
                );
            }
            if let Err(e) = self
                .txn_manager
                .commit_txns_with_status(&signed_txns, txns_status, timestamp_usecs)
                .await
            {
                error!("Failed to notify mempool: {:?}", e);
            }
        }

        // 2. commit
        self.write_storage
            .save_transactions(
                commit_data.txns_to_commit,
                commit_data.first_version,
                commit_data.ledger_info_with_sigs,
            )
            .expect("save transactions failed.");

        // 3. update main chain
        self.main_chain.borrow_mut().insert(height, block_index);
    }

    pub async fn add_block_info(
        &mut self,
        id: &HashValue,
        parent_id: &HashValue,
        timestamp_usecs: u64,
        vm_output: ProcessedVMOutput,
        commit_data: CommitData,
    ) -> Result<()> {
        //1. new_block_info not exist
        let id_exist = self.id_to_block.contains_key(id);
        ensure!(!id_exist, "block already exist in block tree.");

        //2. parent exist
        let parent_height = self
            .id_to_block
            .get(parent_id)
            .expect("parent block not exist in block tree.")
            .height();

        //3. is new root
        let (height, new_root) = if parent_height == self.height {
            // new root
            (self.height + 1, true)
        } else {
            (parent_height + 1, false)
        };

        let new_block_info = BlockInfo::new(
            id,
            parent_id,
            height,
            timestamp_usecs,
            vm_output,
            commit_data,
        );
        self.add_block_info_inner(new_block_info, new_root).await;
        Ok(())
    }

    fn find_block_info_by_block_id(&self, block_id: &HashValue) -> Option<&BlockInfo> {
        self.id_to_block.get(block_id)
    }

    pub fn chain_height_and_root(&self) -> (BlockHeight, BlockIndex) {
        let height = self.height;
        let root_index = self
            .main_chain
            .borrow()
            .get(&height)
            .expect("root is none.")
            .clone();
        (height, root_index)
    }

    pub fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.id_to_block.contains_key(block_hash)
    }

    pub fn root_hash(&self) -> HashValue {
        self.chain_height_and_root().1.id()
    }

    fn find_height_and_index_by_block_id(
        &self,
        block_id: &HashValue,
    ) -> Option<(BlockHeight, BlockIndex)> {
        match self.id_to_block.get(block_id) {
            Some(block_info) => Some((block_info.height(), block_info.block_index())),
            None => None,
        }
    }

    pub fn find_ancestor_until_main_chain(
        &self,
        block_id: &HashValue,
    ) -> Option<(Vec<HashValue>, BlockIndex)> {
        let mut ancestors = vec![];
        let mut latest_id = block_id.clone();
        let mut block_index = None;
        let mut height = self.height;
        while height >= self.tail_height {
            let (h, b_i) = match self.find_height_and_index_by_block_id(&latest_id) {
                Some(h_i) => h_i,
                None => return None,
            };

            let current_id = b_i.id();
            latest_id = b_i.parent_id();
            block_index = Some(b_i.clone());

            if self
                .main_chain
                .borrow()
                .get(&h)
                .expect("get block index from main chain err.")
                .clone()
                .id()
                == current_id
            {
                break;
            } else {
                ancestors.push(current_id);
            }

            height = h;
        }

        ancestors.reverse();
        Some((ancestors, block_index.expect("block_index is none.")))
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
}

/// Can find parent block or children block by BlockInfo
pub struct BlockInfo {
    block_index: BlockIndex,
    height: BlockHeight,
    output_commit_data: Option<(ProcessedVMOutput, CommitData)>,
    timestamp_usecs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitData {
    pub txns_to_commit: Vec<TransactionToCommit>,
    pub first_version: Version,
    pub ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
}

impl CommitData {
    pub fn txns_len(&self) -> usize {
        self.txns_to_commit.len()
    }

    pub fn signed_txns(&self) -> Vec<SignedTransaction> {
        let mut signed_txns = vec![];
        for txn_to_commit in &self.txns_to_commit {
            match txn_to_commit.transaction() {
                Transaction::UserTransaction(txn) => signed_txns.push(txn.clone()),
                _ => {}
            }
        }

        signed_txns
    }
}

impl BlockInfo {
    pub fn new(
        id: &HashValue,
        parent_id: &HashValue,
        height: BlockHeight,
        timestamp_usecs: u64,
        vm_output: ProcessedVMOutput,
        commit_data: CommitData,
    ) -> Self {
        Self::new_inner(
            id,
            parent_id,
            height,
            timestamp_usecs,
            Some((vm_output, commit_data)),
        )
    }

    fn new_inner(
        id: &HashValue,
        parent_id: &HashValue,
        height: BlockHeight,
        timestamp_usecs: u64,
        output_commit_data: Option<(ProcessedVMOutput, CommitData)>,
    ) -> Self {
        let block_index = BlockIndex::new(id, parent_id);
        BlockInfo {
            block_index,
            height,
            output_commit_data,
            timestamp_usecs,
        }
    }

    fn genesis_block_info() -> Self {
        BlockInfo::new_inner(&genesis_id(), &PRE_GENESIS_BLOCK_ID, 0, 0, None)
    }

    fn block_index(&self) -> BlockIndex {
        self.block_index
    }

    fn timestamp_usecs(&self) -> u64 {
        self.timestamp_usecs
    }

    fn id(&self) -> HashValue {
        self.block_index.id()
    }

    fn height(&self) -> BlockHeight {
        self.height
    }

    fn parent_id(&self) -> HashValue {
        self.block_index.parent_id()
    }

    fn commit_data(&self) -> Option<CommitData> {
        match &self.output_commit_data {
            Some(output_commit_data) => Some(output_commit_data.1.clone()),
            None => None,
        }
    }

    fn output(&self) -> Option<ProcessedVMOutput> {
        match &self.output_commit_data {
            Some(output_commit_data) => Some(output_commit_data.0.clone()),
            None => None,
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl BlockTree {
    pub fn add_block_info_for_test(&mut self, id: &HashValue, parent_id: &HashValue) {
        //1. new_block_info not exist
        let id_exist = self.id_to_block.contains_key(id);
        ensure!(!id_exist, "block already exist in block tree.");

        //2. parent exist
        let parent_height = self
            .id_to_block
            .get(parent_id)
            .expect("parent block not exist in block tree.")
            .height();

        //3. is new root
        let (height, new_root) = if parent_height == self.height {
            // new root
            (self.height + 1, true)
        } else {
            (parent_height + 1, false)
        };

        let new_block_info = BlockInfo::new_for_test(id, parent_id, height);
        self.add_block_info_inner(new_block_info, new_root);
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl BlockInfo {
    fn new_for_test(id: &HashValue, parent_id: &HashValue, height: BlockHeight) -> Self {
        Self::new_inner(id, parent_id, height, 0, None)
    }
}
