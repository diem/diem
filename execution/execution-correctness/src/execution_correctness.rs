// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::block::Block;
use executor_types::{Error, StateComputeResult};
use libra_crypto::HashValue;
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};

/// Interface for ExecutionCorrectness.
/// It is basically the same as BlockExecutor except some interfaces will return signature with result.
pub trait ExecutionCorrectness: Send {
    fn committed_block_id(&mut self) -> Result<HashValue, Error>;

    fn reset(&mut self) -> Result<(), Error>;

    /// Executes a block.
    fn execute_block(
        &mut self,
        block: Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error>;

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error>;
}
