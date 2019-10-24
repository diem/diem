// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::{BlockReader, BlockStore};
use consensus_types::{
    block::{block_test_utils::placeholder_certificate_for_block, Block},
    common::Round,
    executed_block::ExecutedBlock,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use futures::executor::block_on;
use libra_crypto::HashValue;
use libra_logger::{set_simple_logger, set_simple_logger_prefix};
use libra_types::{crypto_proxies::ValidatorSigner, ledger_info::LedgerInfo};
use std::sync::Arc;
use termion::color::*;
use tokio::runtime;

mod mock_state_computer;
mod mock_storage;
mod mock_txn_manager;

pub use mock_state_computer::{EmptyStateComputer, MockStateComputer};
pub use mock_storage::{EmptyStorage, MockStorage};
pub use mock_txn_manager::MockTransactionManager;

pub type TestPayload = Vec<usize>;

pub fn build_simple_tree() -> (
    Vec<Arc<ExecutedBlock<TestPayload>>>,
    Arc<BlockStore<TestPayload>>,
) {
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
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis_block, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_block(&a2, 3, Some(genesis.id()));
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis_block, 4);
    let b2 = inserter.insert_block(&b1, 5, None);
    let c1 = inserter.insert_block(&b1, 6, None);

    assert_eq!(block_store.len(), 7);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    (vec![genesis_block, a1, a2, a3, b1, b2, c1], block_store)
}

pub fn build_chain() -> Vec<Arc<ExecutedBlock<TestPayload>>> {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_block(&a2, 3, Some(genesis.id()));
    let a4 = inserter.insert_block(&a3, 4, Some(a1.id()));
    let a5 = inserter.insert_block(&a4, 5, Some(a2.id()));
    let a6 = inserter.insert_block(&a5, 6, Some(a3.id()));
    let a7 = inserter.insert_block(&a6, 7, Some(a4.id()));
    vec![genesis, a1, a2, a3, a4, a5, a6, a7]
}

pub fn build_empty_tree() -> Arc<BlockStore<TestPayload>> {
    let (storage, initial_data) = EmptyStorage::start_for_testing();
    Arc::new(block_on(BlockStore::new(
        storage,
        initial_data,
        Arc::new(EmptyStateComputer),
        true,
        10, // max pruned blocks in mem
    )))
}

pub struct TreeInserter {
    signer: ValidatorSigner,
    payload_val: usize,
    block_store: Arc<BlockStore<TestPayload>>,
}

impl TreeInserter {
    pub fn default() -> Self {
        Self::new(ValidatorSigner::random(None))
    }

    pub fn new(signer: ValidatorSigner) -> Self {
        let block_store = build_empty_tree();
        Self {
            signer,
            payload_val: 0,
            block_store,
        }
    }

    pub fn new_with_store(
        signer: ValidatorSigner,
        block_store: Arc<BlockStore<TestPayload>>,
    ) -> Self {
        Self {
            signer,
            payload_val: 0,
            block_store,
        }
    }

    pub fn signer(&self) -> &ValidatorSigner {
        &self.signer
    }

    pub fn block_store(&self) -> Arc<BlockStore<TestPayload>> {
        Arc::clone(&self.block_store)
    }

    /// This function is generating a placeholder QC for a block's parent that is signed by a single
    /// signer kept by the block store. If more sophisticated QC required, please use
    /// `insert_block_with_qc`.
    pub fn insert_block(
        &mut self,
        parent: &ExecutedBlock<TestPayload>,
        round: Round,
        consensus_block_id: Option<HashValue>,
    ) -> Arc<ExecutedBlock<TestPayload>> {
        // Node must carry a QC to its parent
        let parent_qc = self.create_qc_for_block(parent, consensus_block_id);
        self.insert_block_with_qc(parent_qc, parent, round)
    }

    pub fn insert_block_with_qc(
        &mut self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock<TestPayload>,
        round: Round,
    ) -> Arc<ExecutedBlock<TestPayload>> {
        self.payload_val += 1;
        block_on(
            self.block_store
                .insert_block_with_qc(self.create_block_with_qc(
                    parent_qc,
                    parent,
                    round,
                    vec![self.payload_val],
                )),
        )
        .unwrap()
    }

    pub fn create_qc_for_block(
        &self,
        block: &ExecutedBlock<TestPayload>,
        consensus_block_id: Option<HashValue>,
    ) -> QuorumCert {
        placeholder_certificate_for_block(
            vec![&self.signer],
            block.id(),
            block.round(),
            block.quorum_cert().certified_block().id(),
            block.quorum_cert().certified_block().round(),
            consensus_block_id,
        )
    }

    pub fn insert_qc_for_block(
        &self,
        block: &ExecutedBlock<TestPayload>,
        consensus_block_id: Option<HashValue>,
    ) {
        self.block_store
            .insert_single_quorum_cert(self.create_qc_for_block(block, consensus_block_id))
            .unwrap()
    }

    pub fn create_block_with_qc(
        &self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock<TestPayload>,
        round: Round,
        payload: TestPayload,
    ) -> Block<TestPayload> {
        Block::new_proposal(
            payload,
            round,
            parent.timestamp_usecs() + 1,
            parent_qc,
            &self.signer,
        )
    }

    pub fn insert_reconfiguration_block(
        &mut self,
        parent: &ExecutedBlock<TestPayload>,
        round: Round,
    ) -> Arc<ExecutedBlock<TestPayload>> {
        self.payload_val += 1;
        block_on(
            self.block_store
                .insert_reconfiguration_block(self.create_block_with_qc(
                    self.create_qc_for_block(parent, None),
                    parent,
                    round,
                    vec![self.payload_val],
                )),
        )
        .unwrap()
    }
}

pub fn placeholder_ledger_info() -> LedgerInfo {
    LedgerInfo::new(
        0,
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        None,
    )
}

pub fn placeholder_sync_info() -> SyncInfo {
    SyncInfo::new(
        QuorumCert::certificate_for_genesis(),
        QuorumCert::certificate_for_genesis(),
        None,
    )
}

fn nocapture() -> bool {
    ::std::env::args().any(|arg| arg == "--nocapture")
}

pub fn consensus_runtime() -> runtime::Runtime {
    if nocapture() {
        set_simple_logger("consensus");
    }

    runtime::Builder::new()
        .build()
        .expect("Failed to create Tokio runtime!")
}

pub fn with_smr_id(id: String) -> impl Fn() {
    move || set_simple_logger_prefix(format!("{}[{}]{}", Fg(LightBlack), id, Fg(Reset)))
}
