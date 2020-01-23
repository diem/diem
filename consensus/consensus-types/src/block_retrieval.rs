// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, common::Payload};
use anyhow::{bail, ensure};
use libra_crypto::hash::HashValue;
use libra_types::crypto_proxies::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockRetrievalMode {
    Ancestor(u64),
    // Initial Sync mode is used when initializing from empty consensus db.
    // This mode will retrieve 3 child blocks of the latest commited block from peers
    InitialSync,
}

/// RPC to get a chain of block of the given length starting from the given block id.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockRetrievalRequest {
    block_id: HashValue,
    mode: BlockRetrievalMode,
}

impl BlockRetrievalRequest {
    pub fn new(block_id: HashValue, mode: BlockRetrievalMode) -> Self {
        Self { block_id, mode }
    }
    pub fn block_id(&self) -> HashValue {
        self.block_id
    }
    pub fn retrieval_mode(&self) -> BlockRetrievalMode {
        self.mode.clone()
    }
}

impl fmt::Display for BlockRetrievalRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[BlockRetrievalRequest starting from id {}, retrieval model {:?}]",
            self.block_id, self.mode
        )
    }
}

impl TryFrom<network::proto::RequestBlock> for BlockRetrievalRequest {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::RequestBlock) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<BlockRetrievalRequest> for network::proto::RequestBlock {
    type Error = anyhow::Error;

    fn try_from(block_retrieval_request: BlockRetrievalRequest) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&block_retrieval_request)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockRetrievalStatus {
    // Successfully fill in the request.
    Succeeded,
    // Can not find the block corresponding to block_id.
    IdNotFound,
    // Can not find enough blocks but find some.
    NotEnoughBlocks,
}

/// Carries the returned blocks and the retrieval status.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockRetrievalResponse<T> {
    status: BlockRetrievalStatus,
    #[serde(bound(deserialize = "Block<T>: Deserialize<'de>"))]
    blocks: Vec<Block<T>>,
}

impl<T: Payload> BlockRetrievalResponse<T> {
    pub fn new(status: BlockRetrievalStatus, blocks: Vec<Block<T>>) -> Self {
        Self { status, blocks }
    }

    pub fn status(&self) -> BlockRetrievalStatus {
        self.status.clone()
    }

    pub fn blocks(&self) -> &Vec<Block<T>> {
        &self.blocks
    }

    pub fn verify(
        &self,
        req: &BlockRetrievalRequest,
        sig_verifier: &ValidatorVerifier,
    ) -> anyhow::Result<()> {
        match req.mode {
            BlockRetrievalMode::Ancestor(num_blocks) => {
                self.verify_ancester_retrieval(req.block_id(), num_blocks, sig_verifier)
            }
            BlockRetrievalMode::InitialSync => {
                self.verify_initial_sync(req.block_id(), sig_verifier)
            }
        }
    }

    fn verify_ancester_retrieval(
        &self,
        block_id: HashValue,
        num_blocks: u64,
        sig_verifier: &ValidatorVerifier,
    ) -> anyhow::Result<()> {
        ensure!(
            self.status != BlockRetrievalStatus::Succeeded
                || self.blocks.len() as u64 == num_blocks,
            "not enough blocks returned, expect {}, get {}",
            num_blocks,
            self.blocks.len(),
        );
        self.verify_chain_well_formed(block_id, sig_verifier)
    }

    fn verify_initial_sync(
        &self,
        block_id: HashValue,
        sig_verifier: &ValidatorVerifier,
    ) -> anyhow::Result<()> {
        ensure!(
            self.status() == BlockRetrievalStatus::Succeeded,
            "Initial Sync Request Failed"
        );
        // Things to verify:
        // 1) the last block is the target
        // 2) the chain carries a ledger info to the target
        // 3) the chain is well formed
        ensure!(
            self.blocks().last().map_or(false, |b| b.id() == block_id),
            "Initial Sync block retrieval doesn't end with target block id {}",
            block_id
        );

        ensure!(
            self.blocks()
                .iter()
                .any(|b| b.quorum_cert().commit_info().id() == block_id),
            "Descendants block retrieval does not carry a commit for target id {}",
            block_id
        );

        let highest_block_id = match self.blocks.first() {
            Some(b) => b.id(),
            None => {
                bail!("No blocks retrieved but status is Success");
            }
        };
        self.verify_chain_well_formed(highest_block_id, sig_verifier)
    }

    fn verify_chain_well_formed(
        &self,
        first_block_id: HashValue,
        sig_verifier: &ValidatorVerifier,
    ) -> anyhow::Result<()> {
        self.blocks
            .iter()
            .try_fold(first_block_id, |expected_id, block| {
                block.validate_signatures(sig_verifier)?;
                block.verify_well_formed()?;
                ensure!(
                    block.id() == expected_id,
                    "blocks doesn't form a chain: expect {}, get {}",
                    expected_id,
                    block.id()
                );
                Ok(block.parent_id())
            })
            .map(|_| ())
    }
}

impl<T: Payload> fmt::Display for BlockRetrievalResponse<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status() {
            BlockRetrievalStatus::Succeeded => {
                let block_ids = self
                    .blocks
                    .iter()
                    .map(|b| b.id().short_str())
                    .collect::<Vec<String>>();
                write!(
                    f,
                    "[BlockRetrievalResponse: status: {:?}, num_blocks: {}, block_ids: {:?}]",
                    self.status(),
                    self.blocks().len(),
                    block_ids
                )
            }
            _ => write!(f, "[BlockRetrievalResponse: status: {:?}", self.status()),
        }
    }
}

impl<T: Payload> TryFrom<network::proto::RespondBlock> for BlockRetrievalResponse<T> {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::RespondBlock) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl<T: Payload> TryFrom<BlockRetrievalResponse<T>> for network::proto::RespondBlock {
    type Error = anyhow::Error;

    fn try_from(block_retrieval_response: BlockRetrievalResponse<T>) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&block_retrieval_response)?,
        })
    }
}
