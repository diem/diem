use crate::chained_bft::consensusdb::ConsensusDB;
use crate::state_replication::TxnManager;
use anyhow::{Result, *};
use atomic_refcell::AtomicRefCell;
use consensus_types::{block::Block, common::Payload};
use executor::ProcessedVMOutput;
use libra_crypto::hash::PRE_GENESIS_BLOCK_ID;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::block_index::BlockIndex;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use libra_types::transaction::{
    SignedTransaction, Transaction, TransactionStatus, TransactionToCommit, Version,
};
use libra_types::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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
    indexes: HashMap<BlockHeight, Vec<HashValue>>,
    main_chain: AtomicRefCell<HashMap<BlockHeight, BlockIndex>>,
    write_storage: Arc<dyn StorageWrite>,
    tail_height: BlockHeight,
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    rollback_mode: bool,
    block_store: Arc<ConsensusDB>,
    dump_path: PathBuf,
}

impl Drop for BlockTree {
    fn drop(&mut self) {
        let dump = self.to_dump();
        let bytes: Vec<u8> = dump.into();
        let mut file =
            File::create(self.dump_path.clone()).expect("Unable to create blocktree.blob");
        file.write_all(bytes.as_ref())
            .expect("Unable to dump block tree.");
        file.flush().expect("flush block tree file err.");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct BlockTree4Dump {
    height: BlockHeight,
    id_to_block: HashMap<HashValue, BlockInfo>,
    indexes: HashMap<BlockHeight, Vec<HashValue>>,
    main_chain: HashMap<BlockHeight, BlockIndex>,
    tail_height: BlockHeight,
    rollback_mode: bool,
}

impl From<BlockTree4Dump> for Vec<u8> {
    fn from(dump: BlockTree4Dump) -> Vec<u8> {
        lcs::to_bytes(&dump).expect("BlockTree4Dump to bytes err.")
    }
}

impl TryFrom<Vec<u8>> for BlockTree4Dump {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<BlockTree4Dump> {
        lcs::from_bytes(bytes.as_ref())
            .map_err(|error| format_err!("Deserialize BlockTree4Dump error: {:?}", error,))
    }
}

fn from_dump(
    dump: BlockTree4Dump,
    write_storage: Arc<dyn StorageWrite>,
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    block_store: Arc<ConsensusDB>,
    dump_path: PathBuf,
) -> BlockTree {
    BlockTree {
        height: dump.height,
        id_to_block: dump.id_to_block,
        indexes: dump.indexes,
        main_chain: AtomicRefCell::new(dump.main_chain),
        write_storage,
        tail_height: dump.tail_height,
        txn_manager,
        rollback_mode: dump.rollback_mode,
        block_store,
        dump_path,
    }
}

impl BlockTree {
    fn to_dump(&self) -> BlockTree4Dump {
        BlockTree4Dump {
            height: self.height.clone(),
            id_to_block: self.id_to_block.clone(),
            indexes: self.indexes.clone(),
            main_chain: self.main_chain.borrow().clone(),
            tail_height: self.tail_height.clone(),
            rollback_mode: self.rollback_mode.clone(),
        }
    }

    pub fn new<T: Payload>(
        write_storage: Arc<dyn StorageWrite>,
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        rollback_mode: bool,
        block_store: Arc<ConsensusDB>,
        dump_path: PathBuf,
    ) -> Self {
        //dump path
        let tmp: &Path = dump_path.as_ref();
        let path = tmp.join("block_tree.blob");
        info!("Dump path : {:?}", path);

        if !path.exists() {
            let genesis_height = 0;
            let genesis_block: Block<T> = Block::make_genesis_block();
            let genesis_id = genesis_block.id();
            let genesis_block_info =
                BlockInfo::new_inner(&genesis_id, &PRE_GENESIS_BLOCK_ID, genesis_height, 0, None);

            //init block_store
            let mut genesis_qcs = Vec::new();
            genesis_qcs.push(genesis_block.quorum_cert().clone());
            block_store
                .save_blocks_and_quorum_certificates(vec![genesis_block], genesis_qcs)
                .expect("save blocks failed.");

            // indexes
            let mut genesis_indexes = Vec::new();
            genesis_indexes.push(genesis_id.clone());
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
                block_store,
                dump_path: path,
            }
        } else {
            let mut file = File::open(path.clone()).expect("open genesis err.");
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).expect("reload genesis err.");
            let block_tree_dump =
                BlockTree4Dump::try_from(buffer).expect("BlockTree4Dump try_from err.");
            from_dump(
                block_tree_dump,
                write_storage,
                txn_manager,
                block_store,
                path,
            )
        }
    }

    fn prune(&mut self) {
        let ct = 10;
        if self.tail_height + ct < self.height {
            let times = self.height - self.tail_height - ct;
            for _i in 0..times {
                let tmp_height = self.tail_height;
                //1. indexes
                let tmp_indexes = self.indexes.remove(&tmp_height).expect("indexes is none.");
                //2. id_to_block
                for block_id in tmp_indexes {
                    self.id_to_block.remove(&block_id);
                }
                //3. tail_height
                self.tail_height = tmp_height + 1;
            }
        }
    }

    async fn add_block_info_inner<T: Payload>(
        &mut self,
        block: Block<T>,
        new_block_info: BlockInfo,
        new_root: bool,
    ) {
        //4. new root, rollback, commit
        if new_root {
            let old_root = self.root_hash().unwrap();

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
            let mut hash_list = Vec::new();
            hash_list.push(new_block_info.id().clone());
            self.indexes.insert(new_block_info.height(), hash_list);

            self.prune();
        } else {
            self.indexes
                .get_mut(&new_block_info.height())
                .unwrap()
                .push(new_block_info.id().clone());
        }

        //5. add new block info
        self.id_to_block
            .insert(new_block_info.id().clone(), new_block_info);

        //6. insert block
        let mut qcs = Vec::new();
        qcs.push(block.quorum_cert().clone());
        let mut blocks: Vec<Block<T>> = Vec::new();
        blocks.push(block);
        self.block_store
            .save_blocks_and_quorum_certificates(blocks, qcs)
            .expect("save_blocks err.");
    }

    async fn commit_block(&self, block_info: &BlockInfo) {
        let timestamp_usecs = block_info.timestamp_usecs();
        let vm_output_status = block_info.output_status().expect("output is none.");
        let commit_data = block_info.commit_data().expect("commit_data is none.");
        let height = block_info.height();
        let block_index = block_info.block_index();
        // 1. remove tx from mempool
        if commit_data.txns_len() > 0 {
            let signed_txns = commit_data.signed_txns();
            let signed_txns_len = signed_txns.len();
            let txns_status_len = vm_output_status.len();

            let mut txns_status = vec![];
            for i in 0..signed_txns_len {
                txns_status.push(vm_output_status[txns_status_len - signed_txns_len + i].clone());
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
        self.block_store
            .insert_block_index(&height, &block_index)
            .expect("insert_block_index err.");
        self.main_chain.borrow_mut().insert(height, block_index);
    }

    pub async fn add_block_info<T: Payload>(
        &mut self,
        block: Block<T>,
        parent_id: &HashValue,
        vm_output: ProcessedVMOutput,
        commit_data: CommitData,
    ) -> Result<()> {
        //1. new_block_info not exist
        let id_exist = self.id_to_block.contains_key(&block.id());
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
            &block.id(),
            parent_id,
            height,
            block.timestamp_usecs(),
            vm_output.state_compute_result().status().clone(),
            commit_data,
        );
        self.add_block_info_inner(block, new_block_info, new_root)
            .await;
        Ok(())
    }

    fn find_block_info_by_block_id(&self, block_id: &HashValue) -> Option<&BlockInfo> {
        self.id_to_block.get(block_id)
    }

    pub fn chain_height_and_root(&self) -> Option<(BlockHeight, BlockIndex)> {
        let height = self.height;
        match self.main_chain.borrow().get(&height) {
            Some(i) => Some((height, i.clone())),
            None => None,
        }
    }

    pub fn block_exist(&self, block_hash: &HashValue) -> bool {
        self.id_to_block.contains_key(block_hash)
    }

    pub fn root_hash(&self) -> Option<HashValue> {
        let a = self.chain_height_and_root();
        match a {
            Some(b) => Some(b.1.id()),
            None => None,
        }
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
        let begin_height = if height > 1000 { height - 300 } else { 0 };
        for index in begin_height..height {
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockInfo {
    block_index: BlockIndex,
    height: BlockHeight,
    output_commit_data: Option<(Vec<TransactionStatus>, CommitData)>,
    timestamp_usecs: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
        vm_output_status: Vec<TransactionStatus>,
        commit_data: CommitData,
    ) -> Self {
        Self::new_inner(
            id,
            parent_id,
            height,
            timestamp_usecs,
            Some((vm_output_status, commit_data)),
        )
    }

    fn new_inner(
        id: &HashValue,
        parent_id: &HashValue,
        height: BlockHeight,
        timestamp_usecs: u64,
        output_commit_data: Option<(Vec<TransactionStatus>, CommitData)>,
    ) -> Self {
        let block_index = BlockIndex::new(id, parent_id);
        BlockInfo {
            block_index,
            height,
            output_commit_data,
            timestamp_usecs,
        }
    }

    //    fn genesis_block_info() -> Self {
    //        BlockInfo::new_inner(&genesis_id(), &PRE_GENESIS_BLOCK_ID, 0, 0, None)
    //    }

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

    fn output_status(&self) -> Option<Vec<TransactionStatus>> {
        match &self.output_commit_data {
            Some(output_commit_data) => Some(output_commit_data.0.clone()),
            None => None,
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl BlockInfo {
    fn new_for_test(id: &HashValue, parent_id: &HashValue, height: BlockHeight) -> Self {
        Self::new_inner(id, parent_id, height, 0, None)
    }
}

#[test]
fn test_hash() {
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    struct HashTest {
        hash: HashValue,
    }

    let a = HashTest {
        hash: HashValue::random(),
    };
    let json = lcs::to_bytes(&a).expect("HashTest to bytes err.");
    let obj: HashTest = lcs::from_bytes(json.as_ref()).expect("Deserialize HashTest error.");
}
