// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, common::Payload};
use anyhow::ensure;
use libra_crypto::hash::HashValue;
use libra_types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// RPC to get a chain of block of the given length starting from the given block id.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockRetrievalRequest {
    block_id: HashValue,
    num_blocks: u64,
}

impl BlockRetrievalRequest {
    pub fn new(block_id: HashValue, num_blocks: u64) -> Self {
        Self {
            block_id,
            num_blocks,
        }
    }
    pub fn block_id(&self) -> HashValue {
        self.block_id
    }
    pub fn num_blocks(&self) -> u64 {
        self.num_blocks
    }
}

impl fmt::Display for BlockRetrievalRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[BlockRetrievalRequest starting from id {} with {} blocks]",
            self.block_id, self.num_blocks
        )
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
        self.blocks
            .iter()
            .try_fold(block_id, |expected_id, block| {
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
