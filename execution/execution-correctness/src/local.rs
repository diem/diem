// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{execution_correctness::ExecutionCorrectness, id_and_transactions_from_block};
use consensus_types::{block::Block, vote_proposal::VoteProposal};
use executor_types::{BlockExecutor, Error, StateComputeResult};
use libra_crypto::{ed25519::Ed25519PrivateKey, hash::CryptoHash, traits::SigningKey, HashValue};
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use std::{
    boxed::Box,
    sync::{Arc, Mutex},
};

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
    internal: Arc<Mutex<LocalService>>,
}

impl LocalClient {
    pub fn new(internal: Arc<Mutex<LocalService>>) -> Self {
        Self { internal }
    }
}

impl ExecutionCorrectness for LocalClient {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        self.internal
            .lock()
            .unwrap()
            .block_executor
            .committed_block_id()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.internal.lock().unwrap().block_executor.reset()
    }

    fn execute_block(
        &mut self,
        block: Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let mut local = self.internal.lock().unwrap();
        let mut result = local
            .block_executor
            .execute_block(id_and_transactions_from_block(&block), parent_block_id)?;
        if let Some(prikey) = local.prikey.as_ref() {
            let vote_proposal = VoteProposal::new(
                result.extension_proof(),
                block,
                result.epoch_state().clone(),
            );
            let signature = prikey.sign_message(&vote_proposal.hash());
            result.set_signature(signature);
        }
        Ok(result)
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        self.internal
            .lock()
            .unwrap()
            .block_executor
            .commit_blocks(block_ids, ledger_info_with_sigs)
    }
}
