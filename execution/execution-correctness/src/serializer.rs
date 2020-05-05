// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor_types::{BlockExecutor, Error, StateComputeResult};
use libra_crypto::HashValue;
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize)]
pub enum ExecutionCorrectnessInput {
    CommittedBlockId,
    Reset,
    ExecuteBlock(Box<((HashValue, Vec<Transaction>), HashValue)>),
    CommitBlocks(Box<(Vec<HashValue>, LedgerInfoWithSignatures)>),
}

pub struct SerializerService {
    internal: Box<dyn BlockExecutor>,
}

impl SerializerService {
    pub fn new(internal: Box<dyn BlockExecutor>) -> Self {
        Self { internal }
    }

    pub fn handle_message(&mut self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;

        let output = match input {
            ExecutionCorrectnessInput::CommittedBlockId => {
                lcs::to_bytes(&self.internal.committed_block_id())
            }
            ExecutionCorrectnessInput::Reset => lcs::to_bytes(&self.internal.reset()),
            ExecutionCorrectnessInput::ExecuteBlock(block_with_parent_id) => lcs::to_bytes(
                &self
                    .internal
                    .execute_block(block_with_parent_id.0, block_with_parent_id.1),
            ),
            ExecutionCorrectnessInput::CommitBlocks(blocks_with_li) => lcs::to_bytes(
                &self
                    .internal
                    .commit_blocks(blocks_with_li.0, blocks_with_li.1),
            ),
        };

        Ok(output?)
    }
}

pub struct SerializerClient {
    service: Box<dyn TSerializerClient>,
}

impl SerializerClient {
    pub fn new(serializer_service: Arc<Mutex<SerializerService>>) -> Self {
        let service = Box::new(LocalService { serializer_service });
        Self { service }
    }

    pub fn new_client(service: Box<dyn TSerializerClient>) -> Self {
        Self { service }
    }

    fn request(&mut self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl BlockExecutor for SerializerClient {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        let response = self.request(ExecutionCorrectnessInput::CommittedBlockId)?;
        lcs::from_bytes(&response)?
    }

    fn reset(&mut self) -> Result<(), Error> {
        let response = self.request(ExecutionCorrectnessInput::Reset)?;
        lcs::from_bytes(&response)?
    }

    fn execute_block(
        &mut self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let response = self.request(ExecutionCorrectnessInput::ExecuteBlock(Box::new((
            block,
            parent_block_id,
        ))))?;
        lcs::from_bytes(&response)?
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        let response = self.request(ExecutionCorrectnessInput::CommitBlocks(Box::new((
            block_ids,
            ledger_info_with_sigs,
        ))))?;
        lcs::from_bytes(&response)?
    }
}

pub trait TSerializerClient: Send + Sync {
    fn request(&mut self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error>;
}

struct LocalService {
    pub serializer_service: Arc<Mutex<SerializerService>>,
}

impl TSerializerClient for LocalService {
    fn request(&mut self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.serializer_service
            .lock()
            .unwrap()
            .handle_message(input_message)
    }
}
