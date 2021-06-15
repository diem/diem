// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::Receiver;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_metrics::monitor;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use execution_correctness::ExecutionCorrectness;
use consensus_types::executed_block::ExecutedBlock;
use state_sync::client::StateSyncClient;
use futures::StreamExt;

pub struct CommitPhase {
    commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    execution_correctness_client: Mutex<Box<dyn ExecutionCorrectness + Send + Sync>>,
    synchronizer: StateSyncClient,
}

impl CommitPhase {
    pub fn new(
        commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
        execution_correctness_client: Box<dyn ExecutionCorrectness + Send + Sync>,
        synchronizer: StateSyncClient,
    ) -> Self {
        Self {
            commit_channel_recv,
            execution_correctness_client: Mutex::new(execution_correctness_client),
            synchronizer,
        }
    }

    pub async fn start(mut self){
        while let Some((vecblock, ledger_info)) = self.commit_channel_recv.next().await {
            let mut block_ids = Vec::new();
            let mut txns = Vec::new();
            let mut reconfig_events = Vec::new();

            for block in vecblock {
                block_ids.push(block.id());
                txns.extend(block.transactions_to_commit());
                reconfig_events.extend(block.compute_result().reconfig_events().to_vec());
            }

            monitor!(
                "commit_block",
                self.execution_correctness_client
                    .lock()
                    .commit_blocks(block_ids, ledger_info)
                    .unwrap()
            );

            if let Err(e) = monitor!(
                "notify_state_sync",
                self.synchronizer.commit(txns, reconfig_events).await
            ) {
                error!(error = ?e, "Failed to notify state synchronizer");
            }
        }
    }
}
