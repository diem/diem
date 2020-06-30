// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{execution_correctness::ExecutionCorrectness, ExecutionCorrectnessManager};
use consensus_types::block::Block;
use executor_types::{Error, StateComputeResult};
use libra_config::{
    config::{ExecutionCorrectnessService, NodeConfig, RemoteExecutionService},
    utils,
};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue,
};
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// This container exists only so that we can kill the spawned process after testing is complete.
pub struct ProcessClientWrapper {
    _execution_correctness_manager: ExecutionCorrectnessManager,
    execution_correctness: Box<dyn ExecutionCorrectness>,
    prikey: Option<Ed25519PrivateKey>,
}

impl ProcessClientWrapper {
    pub fn new(storage_addr: SocketAddr) -> Self {
        let server_port = utils::get_available_port();
        let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

        let mut config = NodeConfig::random();
        let mut test_config = config.test.as_ref().unwrap().clone();
        let prikey = test_config
            .execution_keypair
            .as_mut()
            .unwrap()
            .take_private();
        config.execution.service =
            ExecutionCorrectnessService::SpawnedProcess(RemoteExecutionService { server_address });
        config.storage.address = storage_addr;

        let execution_correctness_manager = ExecutionCorrectnessManager::new(&mut config);
        let execution_correctness = execution_correctness_manager.client();

        Self {
            _execution_correctness_manager: execution_correctness_manager,
            execution_correctness,
            prikey,
        }
    }

    pub fn pubkey(&self) -> Option<Ed25519PublicKey> {
        self.prikey.as_ref().map(Ed25519PublicKey::from)
    }
}

impl ExecutionCorrectness for ProcessClientWrapper {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        self.execution_correctness.committed_block_id()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.execution_correctness.reset()
    }

    fn execute_block(
        &mut self,
        block: Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        self.execution_correctness
            .execute_block(block, parent_block_id)
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        self.execution_correctness
            .commit_blocks(block_ids, ledger_info_with_sigs)
    }
}
