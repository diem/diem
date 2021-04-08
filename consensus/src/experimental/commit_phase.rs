// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputer;
use consensus_types::executed_block::ExecutedBlock;
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use futures::StreamExt;
use std::{collections::BTreeMap, sync::Arc};

pub struct CommitPhase {
    committer: Arc<dyn StateComputer>,
    receive_channel: channel::Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
}

impl CommitPhase {
    pub fn new(
        committer: Arc<dyn StateComputer>,
        receive_channel: channel::Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    ) -> Self {
        Self {
            committer,
            receive_channel,
        }
    }

    pub async fn start(mut self) {
        while let Some((executed_blocks, _)) = self.receive_channel.next().await {
            info!(
                "Receive executed block at round {}",
                executed_blocks.last().unwrap().round()
            );
        }
    }
}
