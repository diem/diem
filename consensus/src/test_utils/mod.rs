// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_storage::{BlockReader, BlockStore};
use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    common::Round,
    executed_block::ExecutedBlock,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use libra_crypto::HashValue;
use libra_logger::Level;
use libra_types::{ledger_info::LedgerInfo, validator_signer::ValidatorSigner};
use std::{future::Future, sync::Arc, time::Duration};
use tokio::{runtime, time::timeout};

mod mock_state_computer;
mod mock_storage;
#[cfg(any(test, feature = "fuzzing"))]
mod mock_txn_manager;

use crate::util::mock_time_service::SimulatedTimeService;
use consensus_types::{block::block_test_utils::gen_test_certificate, common::Payload};
use libra_types::block_info::BlockInfo;
pub use mock_state_computer::{EmptyStateComputer, MockStateComputer};
pub use mock_storage::{EmptyStorage, MockSharedStorage, MockStorage};
pub use mock_txn_manager::MockTransactionManager;

pub const TEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn build_simple_tree() -> (Vec<Arc<ExecutedBlock>>, Arc<BlockStore>) {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let genesis_block_id = genesis.id();
    let genesis_block = block_store
        .get_block(genesis_block_id)
        .expect("genesis block must exist");
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    assert_eq!(block_store.block_exists(genesis_block.id()), true);

    //       ╭--> A1--> A2--> A3
    // Genesis--> B1--> B2
    //             ╰--> C1
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis_block, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_block(&a2, 3, Some(genesis.block_info()));
    let b1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis_block, 4);
    let b2 = inserter.insert_block(&b1, 5, None);
    let c1 = inserter.insert_block(&b1, 6, None);

    assert_eq!(block_store.len(), 7);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    (vec![genesis_block, a1, a2, a3, b1, b2, c1], block_store)
}

pub fn build_empty_tree() -> Arc<BlockStore> {
    let (initial_data, storage) = EmptyStorage::start_for_testing();
    Arc::new(BlockStore::new(
        storage,
        initial_data,
        Arc::new(EmptyStateComputer),
        10, // max pruned blocks in mem
        Arc::new(SimulatedTimeService::new()),
    ))
}

pub struct TreeInserter {
    signer: ValidatorSigner,
    block_store: Arc<BlockStore>,
}

impl TreeInserter {
    pub fn default() -> Self {
        Self::new(ValidatorSigner::random(None))
    }

    pub fn new(signer: ValidatorSigner) -> Self {
        let block_store = build_empty_tree();
        Self {
            signer,
            block_store,
        }
    }

    pub fn new_with_store(signer: ValidatorSigner, block_store: Arc<BlockStore>) -> Self {
        Self {
            signer,
            block_store,
        }
    }

    pub fn signer(&self) -> &ValidatorSigner {
        &self.signer
    }

    pub fn block_store(&self) -> Arc<BlockStore> {
        Arc::clone(&self.block_store)
    }

    /// This function is generating a placeholder QC for a block's parent that is signed by a single
    /// signer kept by the block store. If more sophisticated QC required, please use
    /// `insert_block_with_qc`.
    pub fn insert_block(
        &mut self,
        parent: &ExecutedBlock,
        round: Round,
        committed_block: Option<BlockInfo>,
    ) -> Arc<ExecutedBlock> {
        // Node must carry a QC to its parent
        let parent_qc = self.create_qc_for_block(parent, committed_block);
        self.insert_block_with_qc(parent_qc, parent, round)
    }

    pub fn insert_block_with_qc(
        &mut self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock,
        round: Round,
    ) -> Arc<ExecutedBlock> {
        self.block_store
            .insert_block_with_qc(self.create_block_with_qc(
                parent_qc,
                parent.timestamp_usecs() + 1,
                round,
                vec![],
            ))
            .unwrap()
    }

    pub fn create_qc_for_block(
        &self,
        block: &ExecutedBlock,
        committed_block: Option<BlockInfo>,
    ) -> QuorumCert {
        gen_test_certificate(
            vec![&self.signer],
            block.block_info(),
            block.quorum_cert().certified_block().clone(),
            committed_block,
        )
    }

    pub fn insert_qc_for_block(&self, block: &ExecutedBlock, committed_block: Option<BlockInfo>) {
        self.block_store
            .insert_single_quorum_cert(self.create_qc_for_block(block, committed_block))
            .unwrap()
    }

    pub fn create_block_with_qc(
        &self,
        parent_qc: QuorumCert,
        timestamp_usecs: u64,
        round: Round,
        payload: Payload,
    ) -> Block {
        Block::new_proposal(payload, round, timestamp_usecs, parent_qc, &self.signer)
    }
}

pub fn placeholder_ledger_info() -> LedgerInfo {
    LedgerInfo::new(BlockInfo::empty(), HashValue::zero())
}

pub fn placeholder_sync_info() -> SyncInfo {
    SyncInfo::new(certificate_for_genesis(), certificate_for_genesis(), None)
}

fn nocapture() -> bool {
    ::std::env::args().any(|arg| arg == "--nocapture")
}

pub fn consensus_runtime() -> runtime::Runtime {
    if nocapture() {
        ::libra_logger::Logger::new().level(Level::Debug).init();
    }

    runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime!")
}

pub fn timed_block_on<F>(runtime: &mut runtime::Runtime, f: F) -> <F as Future>::Output
where
    F: Future,
{
    runtime
        .block_on(async { timeout(TEST_TIMEOUT, f).await })
        .expect("test timed out")
}
