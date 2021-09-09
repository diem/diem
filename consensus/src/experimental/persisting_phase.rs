// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use crate::{
    experimental::pipeline_phase::{ResponseWithInstruction, StatelessPipeline},
    state_replication::{StateComputer, StateComputerCommitCallBackType},
};
use async_trait::async_trait;
use consensus_types::executed_block::ExecutedBlock;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::Error;

/// [ This class is used when consensus.decoupled = true ]
/// PersistingPhase is a singleton that receives aggregated blocks from
/// the buffer manager and persists them. Upon success, it returns
/// a response.

pub struct PersistingRequest {
    pub blocks: Vec<Arc<ExecutedBlock>>,
    pub commit_ledger_info: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

impl Debug for PersistingRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for PersistingRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "PersistingRequest({:?}, {})",
            self.blocks, self.commit_ledger_info,
        )
    }
}

pub type PersistingResponse = Result<(), Error>;

pub struct PersistingPhase {
    persisting_handle: Arc<dyn StateComputer>,
}

impl PersistingPhase {
    pub fn new(persisting_handle: Arc<dyn StateComputer>) -> Self {
        Self { persisting_handle }
    }
}

#[async_trait]
impl StatelessPipeline for PersistingPhase {
    type Request = PersistingRequest;
    type Response = PersistingResponse;
    async fn process(&self, req: PersistingRequest) -> ResponseWithInstruction<PersistingResponse> {
        let PersistingRequest {
            blocks,
            commit_ledger_info,
            callback,
        } = req;

        ResponseWithInstruction::from(
            self.persisting_handle
                .commit(&blocks, commit_ledger_info, callback)
                .await,
        )
    }
}
