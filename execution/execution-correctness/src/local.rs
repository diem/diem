// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor_types::{BlockExecutor, Error, StateComputeResult};
use libra_crypto::HashValue;
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use std::{
    boxed::Box,
    sync::{Arc, Mutex},
};

/// A local interface into ExecutionCorrectness. Constructed in such a way that the container / caller
/// cannot distinguish this API from an actual client/server process without being exposed to
/// the actual container instead the caller can access a Box<dyn TExecutionCorrectness>.
pub struct LocalClient {
    internal: Arc<Mutex<Box<dyn BlockExecutor>>>,
}

impl LocalClient {
    pub fn new(internal: Arc<Mutex<Box<dyn BlockExecutor>>>) -> Self {
        Self { internal }
    }
}

impl BlockExecutor for LocalClient {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        self.internal.lock().unwrap().committed_block_id()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.internal.lock().unwrap().reset()
    }

    fn execute_block(
        &mut self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        self.internal
            .lock()
            .unwrap()
            .execute_block(block, parent_block_id)
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        self.internal
            .lock()
            .unwrap()
            .commit_blocks(block_ids, ledger_info_with_sigs)
    }
}
