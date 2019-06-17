// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        common::Round,
        consensus_types::{block::Block, quorum_cert::QuorumCert},
        safety::vote_msg::VoteMsg,
    },
    state_replication::ExecutedState,
};
use crypto::{hash::CryptoHash, HashValue};
use futures::{channel::mpsc, executor::block_on};
use logger::{set_simple_logger, set_simple_logger_prefix};
use std::{collections::HashMap, sync::Arc};
use termion::color::*;
use tokio::runtime;
use tools::output_capture::OutputCapture;
use types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_signer::ValidatorSigner,
};

mod mock_state_computer;
mod mock_storage;
mod mock_txn_manager;

pub use mock_state_computer::MockStateComputer;
pub use mock_storage::{EmptyStorage, MockStorage};
pub use mock_txn_manager::MockTransactionManager;

pub type TestPayload = Vec<usize>;

pub fn build_empty_tree() -> Arc<BlockStore<Vec<usize>>> {
    let signer = ValidatorSigner::random();
    build_empty_tree_with_custom_signing(signer.clone())
}

pub fn build_empty_tree_with_custom_signing(
    my_signer: ValidatorSigner,
) -> Arc<BlockStore<Vec<usize>>> {
    let (commit_cb_sender, _commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();
    let (storage, initial_data) = EmptyStorage::start_for_testing();
    Arc::new(block_on(BlockStore::new(
        storage,
        initial_data,
        my_signer,
        Arc::new(MockStateComputer::new(commit_cb_sender)),
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
        parent: &Block<Vec<usize>>,
        round: Round,
    ) -> Arc<Block<Vec<usize>>> {
        // Node must carry a QC to its parent
        let parent_qc = placeholder_certificate_for_block(
            vec![self.block_store.signer().clone()],
            parent.id(),
            parent.round(),
        );

        self.insert_block_with_qc(parent_qc, parent, round)
    }

    pub fn insert_block_with_qc(
        &mut self,
        parent_qc: QuorumCert,
        parent: &Block<Vec<usize>>,
        round: Round,
    ) -> Arc<Block<Vec<usize>>> {
        self.payload_val += 1;
        block_on(self.block_store.insert_block_with_qc(Block::make_block(
            parent,
            vec![self.payload_val],
            round,
            parent.timestamp_usecs() + 1,
            parent_qc,
            self.block_store.signer(),
        )))
        .unwrap()
    }

    pub fn insert_pre_made_block(
        &mut self,
        block: Block<Vec<usize>>,
        block_signer: &ValidatorSigner,
        qc_signers: Vec<ValidatorSigner>,
    ) -> Arc<Block<Vec<usize>>> {
        self.payload_val += 1;
        let new_round = if block.round() > 0 {
            block.round() - 1
        } else {
            0
        };
        let parent_qc = placeholder_certificate_for_block(qc_signers, block.parent_id(), new_round);
        let new_block = Block::new_internal(
            block.get_payload().clone(),
            block.parent_id(),
            block.round(),
            block.height(),
            block.timestamp_usecs(),
            parent_qc,
            block_signer,
        );
        block_on(self.block_store.insert_block_with_qc(new_block)).unwrap()
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
    )
}

pub fn placeholder_certificate_for_block(
    signers: Vec<ValidatorSigner>,
    certified_block_id: HashValue,
    certified_block_round: u64,
) -> QuorumCert {
    // Assuming executed state to be Genesis state.
    let certified_block_state = ExecutedState::state_for_genesis();
    let consensus_data_hash = VoteMsg::vote_digest(
        certified_block_id,
        certified_block_state,
        certified_block_round,
    );

    // This ledger info doesn't carry any meaningful information: it is all zeros except for
    // the consensus data hash that carries the actual vote.
    let mut ledger_info_placeholder = placeholder_ledger_info();
    ledger_info_placeholder.set_consensus_data_hash(consensus_data_hash);

    let mut signatures = HashMap::new();
    for signer in signers {
        let li_sig = signer
            .sign_message(ledger_info_placeholder.hash())
            .expect("Failed to sign LedgerInfo");
        signatures.insert(signer.author(), li_sig);
    }

    QuorumCert::new(
        certified_block_id,
        certified_block_state,
        certified_block_round,
        LedgerInfoWithSignatures::new(ledger_info_placeholder, signatures),
    )
}

pub fn consensus_runtime() -> runtime::Runtime {
    set_simple_logger("consensus");
    let capture = OutputCapture::grab();
    runtime::Builder::new()
        .after_start(move || capture.apply())
        .build()
        .expect("Failed to create Tokio runtime!")
}

pub fn with_smr_id(id: String) -> impl Fn() {
    let capture = OutputCapture::grab();
    move || {
        capture.apply();
        set_simple_logger_prefix(format!("{}[{}]{}", Fg(LightBlack), id.clone(), Fg(Reset)))
    }
}
