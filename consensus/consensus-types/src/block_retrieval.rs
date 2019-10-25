// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, common::Payload};
use failure::prelude::*;
use libra_crypto::hash::HashValue;
use libra_types::crypto_proxies::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockRetrievalMode {
    /// Retrieve the chain of ancestors with the given length.
    Ancestors(u64),
    /// Retrieve the chain of descendants of a committed block.
    /// The chain ends with the given block id and must carry a LedgerInfo to this id.
    /// In case no such chain can be found, the response status is `IdNotFound`.
    /// The number of the blocks in the chain is limited by the max message size constraints at the
    /// sender.
    Descendants,
}

/// RPC to get a chain of blocks for the given block id.
/// There are two modes of the block retrieval requests: ancestors or descendants, see
/// `BlockRetrievalMode`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockRetrievalRequest {
    block_id: HashValue,
    retrieval_mode: BlockRetrievalMode,
}

impl BlockRetrievalRequest {
    pub fn new(block_id: HashValue, retrieval_mode: BlockRetrievalMode) -> Self {
        Self {
            block_id,
            retrieval_mode,
        }
    }
    pub fn block_id(&self) -> HashValue {
        self.block_id
    }
    pub fn retrieval_mode(&self) -> BlockRetrievalMode {
        self.retrieval_mode.clone()
    }
}

impl fmt::Display for BlockRetrievalRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[BlockRetrievalRequest for id {}, retrieval mode {:?}]",
            self.block_id, self.retrieval_mode
        )
    }
}

impl TryFrom<network::proto::RequestBlock> for BlockRetrievalRequest {
    type Error = failure::Error;

    fn try_from(proto: network::proto::RequestBlock) -> failure::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<BlockRetrievalRequest> for network::proto::RequestBlock {
    type Error = failure::Error;

    fn try_from(block_retrieval_request: BlockRetrievalRequest) -> failure::Result<Self> {
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
        retrieval_request: &BlockRetrievalRequest,
        sig_verifier: &ValidatorVerifier,
    ) -> failure::Result<()> {
        match retrieval_request.retrieval_mode {
            BlockRetrievalMode::Ancestors(num_blocks) => self.verify_ancestors_retrieval(
                retrieval_request.block_id(),
                num_blocks,
                sig_verifier,
            ),
            BlockRetrievalMode::Descendants => {
                self.verify_descendants_retrieval(retrieval_request.block_id(), sig_verifier)
            }
        }
    }

    fn verify_ancestors_retrieval(
        &self,
        block_id: HashValue,
        num_blocks: u64,
        sig_verifier: &ValidatorVerifier,
    ) -> failure::Result<()> {
        ensure!(
            self.status != BlockRetrievalStatus::Succeeded
                || self.blocks.len() as u64 == num_blocks,
            "not enough blocks returned, expect {}, get {}",
            num_blocks,
            self.blocks.len(),
        );
        self.verify_chain(block_id, sig_verifier)
    }

    fn verify_descendants_retrieval(
        &self,
        target_id: HashValue,
        sig_verifier: &ValidatorVerifier,
    ) -> failure::Result<()> {
        if self.status != BlockRetrievalStatus::Succeeded {
            ensure!(
                self.blocks.is_empty(),
                "Non-empty chain in a failed retrieval response."
            );
            return Ok(());
        }
        // Things to verify:
        // 1) the chain is well formed
        // 2) the last block is the target
        // 3) the chain carries a ledger info to the target
        ensure!(
            self.blocks.last().map_or(false, |b| b.id() == target_id),
            "Descendants block retrieval does not end with target id {}",
            target_id
        );
        ensure!(
            self.blocks()
                .iter()
                .any(|b| b.quorum_cert().committed_block_id() == Some(target_id)),
            "Descendants block retrieval does not carry a commit for target id {}",
            target_id
        );

        let highest_block_id = match self.blocks.first() {
            Some(b) => b.id(),
            None => {
                bail!("No blocks retrieved but status is Success");
            }
        };
        self.verify_chain(highest_block_id, sig_verifier)
    }

    // Verifies that the blocks are well-formed and form a valid chain.
    fn verify_chain(
        &self,
        first_id: HashValue,
        sig_verifier: &ValidatorVerifier,
    ) -> failure::Result<()> {
        self.blocks
            .iter()
            .try_fold(first_id, |expected_id, block| {
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
    type Error = failure::Error;

    fn try_from(proto: network::proto::RespondBlock) -> failure::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl<T: Payload> TryFrom<BlockRetrievalResponse<T>> for network::proto::RespondBlock {
    type Error = failure::Error;

    fn try_from(block_retrieval_response: BlockRetrievalResponse<T>) -> failure::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&block_retrieval_response)?,
        })
    }
}
