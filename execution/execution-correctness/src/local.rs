// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execution_correctness::ExecutionCorrectness;
use consensus_types::{block::Block, vote_proposal::VoteProposal};
use diem_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, HashValue};
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{BlockExecutor, Error, StateComputeResult};
use std::{boxed::Box, sync::Arc};

pub struct LocalService {
    block_executor: Box<dyn BlockExecutor>,
    prikey: Option<Ed25519PrivateKey>,
}

impl LocalService {
    pub fn new(block_executor: Box<dyn BlockExecutor>, prikey: Option<Ed25519PrivateKey>) -> Self {
        Self {
            block_executor,
            prikey,
        }
    }
}

/// A local interface into ExecutionCorrectness. Constructed in such a way that the container / caller
/// cannot distinguish this API from an actual client/server process without being exposed to
/// the actual container instead the caller can access a Box<dyn TExecutionCorrectness>.
pub struct LocalClient {
    internal: Arc<LocalService>,
}

impl LocalClient {
    pub fn new(internal: Arc<LocalService>) -> Self {
        Self { internal }
    }
}

impl ExecutionCorrectness for LocalClient {
    fn committed_block_id(&self) -> Result<HashValue, Error> {
        self.internal.block_executor.committed_block_id()
    }

    fn reset(&self) -> Result<(), Error> {
        self.internal.block_executor.reset()
    }

    fn execute_block(
        &self,
        block: Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let local = &self.internal;
        let mut result = local.block_executor.execute_block(
            (block.id(), block.transactions_to_execute()),
            parent_block_id,
        )?;
        if let Some(prikey) = local.prikey.as_ref() {
            let vote_proposal = VoteProposal::new(
                result.extension_proof(),
                block,
                result.epoch_state().clone(),
            );
            let signature = prikey.sign(&vote_proposal);
            result.set_signature(signature);
        }
        Ok(result)
    }

    fn commit_blocks(
        &self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        self.internal
            .block_executor
            .commit_blocks(block_ids, ledger_info_with_sigs)
    }
}
