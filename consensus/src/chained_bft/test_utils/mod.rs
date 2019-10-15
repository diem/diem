// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::{BlockStore, BlockReader};
use consensus_types::{
    block::{Block, ExecutedBlock},
    common::Round,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
    vote_data::VoteData,
    test_utils::placeholder_certificate_for_block,
};
use crypto::{hash::CryptoHash, HashValue};
use executor::ExecutedState;
use futures::executor::block_on;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner},
    ledger_info::LedgerInfo,
};
use logger::{set_simple_logger, set_simple_logger_prefix};
use std::{collections::BTreeMap, sync::Arc};
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
    Vec<Arc<ExecutedBlock<Vec<usize>>>>,
    Arc<BlockStore<Vec<usize>>>,
) {
    let block_store = build_empty_tree();
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
    let mut inserter = TreeInserter::new(block_store.clone());
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis_block, 1);
    let a2 = inserter.insert_guaranteed_block(&a1, 2);
    let a3 = inserter.insert_guaranteed_block(&a2, 3);
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis_block, 4);
    let b2 = inserter.insert_guaranteed_block(&b1, 5);
    let c1 = inserter.insert_guaranteed_block(&b1, 6);

    assert_eq!(block_store.len(), 7);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    (vec![genesis_block, a1, a2, a3, b1, b2, c1], block_store)
}

pub fn build_chain() -> (Vec<Arc<ExecutedBlock<Vec<usize>>>>, Arc<BlockStore<Vec<usize>>>) {
    let block_store = build_empty_tree(); // this seems to call `find_root` -- is this ok?
    let mut inserter = TreeInserter::new(block_store.clone());
    let genesis = block_store.root();
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis, 1);
    let a2 = inserter.insert_guaranteed_block(&a1, 2);
    let a3 = inserter.insert_guaranteed_block(&a2, 3);
    let a4 = inserter.insert_guaranteed_block(&a3, 4);
    let a5 = inserter.insert_guaranteed_block(&a4, 5);
    let a6 = inserter.insert_guaranteed_block(&a5, 6);
    let a7 = inserter.insert_guaranteed_block(&a6, 7);
    (vec![genesis, a1, a2, a3, a4, a5, a6, a7], block_store)
}

pub fn build_empty_tree() -> Arc<BlockStore<Vec<usize>>> {
    let signer = ValidatorSigner::random(None);
    build_empty_tree_with_custom_signing(signer)
}

pub fn build_empty_tree_with_custom_signing(
    my_signer: ValidatorSigner,
) -> Arc<BlockStore<Vec<usize>>> {
    let (storage, initial_data) = EmptyStorage::start_for_testing();
    Arc::new(block_on(BlockStore::new(
        storage,
        initial_data,
        my_signer,
        Arc::new(EmptyStateComputer),
        true,
        10, // max pruned blocks in mem
    )))
}

pub struct TreeInserter {
    payload_val: usize,
    block_store: Arc<BlockStore<Vec<usize>>>,
}

impl TreeInserter {
    pub fn new(block_store: Arc<BlockStore<Vec<usize>>>) -> Self {
        Self {
            payload_val: 0,
            block_store,
        }
    }

    /// This function is generating a placeholder QC for a block's parent that is signed by a single
    /// signer kept by the block store. If more sophisticated QC required, please use
    /// `insert_block_with_qc`.
    pub fn insert_block(
        &mut self,
        parent: &ExecutedBlock<Vec<usize>>,
        round: Round,
    ) -> Arc<ExecutedBlock<Vec<usize>>> {
        // Node must carry a QC to its parent
        let parent_qc = placeholder_certificate_for_block(
            vec![self.block_store.signer()],
            parent.id(),
            parent.round(),
            parent.quorum_cert().certified_block_id(),
            parent.quorum_cert().certified_block_round(),
            false,
        );
        self.insert_block_with_qc(parent_qc, parent, round)
    }

    // Insert a block that is guaranteed to be committed.
    // This function is only to be used for testing other components, given that the block will commit.
    pub fn insert_guaranteed_block(
        &mut self,
        parent: &ExecutedBlock<Vec<usize>>,
        round: Round,
    ) -> Arc<ExecutedBlock<Vec<usize>>> {
        // Node must carry a QC to its parent
        let parent_qc = placeholder_certificate_for_block(
            vec![self.block_store.signer()],
            parent.id(),
            parent.round(),
            parent.quorum_cert().certified_block_id(),
            parent.quorum_cert().certified_block_round(),
            true,
        );
        self.insert_block_with_qc(parent_qc, parent, round)
    }

    pub fn insert_block_with_qc(
        &mut self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock<Vec<usize>>,
        round: Round,
    ) -> Arc<ExecutedBlock<Vec<usize>>> {
        self.payload_val += 1;
        block_on(self.block_store.insert_block_with_qc(Block::make_block(
            parent.block(),
            vec![self.payload_val],
            round,
            parent.timestamp_usecs() + 1,
            parent_qc,
            self.block_store.signer(),
        )))
        .unwrap()
    }
}

pub fn placeholder_ledger_info_for_consensus_block_id(consensus_block_id: HashValue) -> LedgerInfo {
    LedgerInfo::new(
        0,
        HashValue::zero(),
        HashValue::zero(),
        consensus_block_id,
        0,
        0,
        None,
    )
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
