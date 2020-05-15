// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ExecutionCorrectnessManager;
use executor_types::{BlockExecutor, Error, StateComputeResult};
use libra_config::{
    config::{ExecutionCorrectnessService, NodeConfig, RemoteExecutionService},
    utils,
};
use libra_crypto::HashValue;
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// This container exists only so that we can kill the spawned process after testing is complete.
pub struct ProcessClientWrapper {
    _execution_correctness_manager: ExecutionCorrectnessManager,
    block_executor: Box<dyn BlockExecutor>,
}

impl ProcessClientWrapper {
    pub fn new(storage_addr: SocketAddr) -> Self {
        let server_port = utils::get_available_port();
        let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

        let mut config = NodeConfig::random();
        config.execution.service =
            ExecutionCorrectnessService::SpawnedProcess(RemoteExecutionService { server_address });
        config.storage.address = storage_addr;

        let execution_correctness_manager = ExecutionCorrectnessManager::new(&mut config);
        let block_executor = execution_correctness_manager.client();

        Self {
            _execution_correctness_manager: execution_correctness_manager,
            block_executor,
        }
    }
}

impl BlockExecutor for ProcessClientWrapper {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        self.block_executor.committed_block_id()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.block_executor.reset()
    }

    fn execute_block(
        &mut self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        self.block_executor.execute_block(block, parent_block_id)
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        self.block_executor
            .commit_blocks(block_ids, ledger_info_with_sigs)
    }
}
