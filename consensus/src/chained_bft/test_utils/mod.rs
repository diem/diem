// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::BlockStore;
use consensus_types::{
    block::{block_test_utils::placeholder_certificate_for_block, Block, ExecutedBlock},
    common::Round,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use crypto::HashValue;
use futures::executor::block_on;
use libra_types::{crypto_proxies::ValidatorSigner, ledger_info::LedgerInfo};
use logger::{set_simple_logger, set_simple_logger_prefix};
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
        let parent_qc = self.create_qc_for_block(parent);

        self.insert_block_with_qc(parent_qc, parent, round)
    }

    pub fn insert_block_with_qc(
        &mut self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock<Vec<usize>>,
        round: Round,
    ) -> Arc<ExecutedBlock<Vec<usize>>> {
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

    pub fn create_qc_for_block(&self, block: &ExecutedBlock<TestPayload>) -> QuorumCert {
        placeholder_certificate_for_block(
            vec![self.block_store.signer()],
            block.id(),
            block.round(),
            block.quorum_cert().certified_block().id(),
            block.quorum_cert().certified_block().round(),
        )
    }

    pub fn insert_qc_for_block(&self, block: &ExecutedBlock<TestPayload>) {
        self.block_store
            .insert_single_quorum_cert(self.create_qc_for_block(block))
            .unwrap()
    }

    pub fn create_block_with_qc(
        &self,
        parent_qc: QuorumCert,
        parent: &ExecutedBlock<TestPayload>,
        round: Round,
        payload: TestPayload,
    ) -> Block<TestPayload> {
        Block::make_block(
            parent.block(),
            payload,
            round,
            parent.timestamp_usecs() + 1,
            parent_qc,
            self.block_store.signer(),
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
                    self.create_qc_for_block(parent),
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
