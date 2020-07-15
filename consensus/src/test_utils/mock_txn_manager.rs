// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{state_replication::TxnManager, txn_manager::MempoolProxy};
use anyhow::Result;
use consensus_types::{
    block::{block_test_utils::random_payload, Block},
    common::Payload,
};
use executor_types::StateComputeResult;
use futures::channel::mpsc;
use libra_mempool::ConsensusRequest;
use libra_types::{
    transaction::TransactionStatus,
    vm_status::{KeptVMStatus, StatusCode},
};
use rand::Rng;

#[derive(Clone)]
pub struct MockTransactionManager {
    rejected_txns: Payload,
    // used non-mocked TxnManager to test interaction with shared mempool
    mempool_proxy: Option<MempoolProxy>,
}

impl MockTransactionManager {
    pub fn new(consensus_to_mempool_sender: Option<mpsc::Sender<ConsensusRequest>>) -> Self {
        let mempool_proxy = consensus_to_mempool_sender.map(MempoolProxy::new);
        Self {
            rejected_txns: vec![],
            mempool_proxy,
        }
    }
}

// mock transaction status on the fly
fn mock_transaction_status(count: usize) -> Vec<TransactionStatus> {
    let mut statuses = vec![];
    // generate count + 1 status to mock the block metadata txn in mempool proxy
    for _ in 0..=count {
        let random_status = match rand::thread_rng().gen_range(0, 2) {
            0 => TransactionStatus::Keep(KeptVMStatus::Executed),
            1 => TransactionStatus::Discard(StatusCode::UNKNOWN_VALIDATION_STATUS),
            _ => unreachable!(),
        };
        statuses.push(random_status);
    }
    statuses
}

#[async_trait::async_trait]
impl TxnManager for MockTransactionManager {
    /// The returned future is fulfilled with the vector of SignedTransactions
    async fn pull_txns(&self, max_size: u64, _exclude_txns: Vec<&Payload>) -> Result<Payload> {
        Ok(random_payload(max_size as usize))
    }

    async fn notify(&self, block: &Block, compute_results: &StateComputeResult) -> Result<()> {
        if self.mempool_proxy.is_some() {
            let mock_compute_result = StateComputeResult::new(
                compute_results.root_hash(),
                compute_results.frozen_subtree_roots().clone(),
                compute_results.num_leaves(),
                compute_results.parent_frozen_subtree_roots().clone(),
                compute_results.parent_num_leaves(),
                compute_results.epoch_state().clone(),
                mock_transaction_status(block.payload().map_or(0, |txns| txns.len())),
                compute_results.transaction_info_hashes().clone(),
            );
            assert!(self
                .mempool_proxy
                .as_ref()
                .unwrap()
                .notify(&block, &mock_compute_result)
                .await
                .is_ok());
        }
        Ok(())
    }
}
