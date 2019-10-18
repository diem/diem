// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::Block;
use failure::prelude::{Error, Result};
use libra_crypto::hash::HashValue;
use libra_types::transaction::Version;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically. It is used for the protocol safety and liveness (please see the detailed protocol description).
pub type Round = u64;

/// This structure contains all the information needed for tracking a block
/// without having access to the block or its execution output state. It
/// assumes that the block is the last block executed within the ledger.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockInfo {
    /// Epoch number corresponds to the set of validators that are active for this block.
    epoch: u64,
    /// The consensus protocol executes proposals (blocks) in rounds, which monotically increase per epoch.
    round: Round,
    /// The identifier (hash) of the block.
    id: HashValue,
    /// The accumulator root hash after executing this block.
    executed_state_id: HashValue,
    /// The version of the latest transaction in the ledger.
    version: Version,
    /// The timestamp this block was proposed by a proposer.
    timestamp_usecs: u64,
}

impl BlockInfo {
    pub fn new(
        epoch: u64,
        round: Round,
        id: HashValue,
        executed_state_id: HashValue,
        version: Version,
        timestamp_usecs: u64,
    ) -> Self {
        Self {
            epoch,
            round,
            id,
            executed_state_id,
            timestamp_usecs,
            version,
        }
    }

    pub fn empty() -> Self {
        Self {
            epoch: 0,
            round: 0,
            id: HashValue::zero(),
            executed_state_id: HashValue::zero(),
            version: 0,
            timestamp_usecs: 0,
        }
    }

    pub fn from_block<T>(block: &Block<T>, executed_state_id: HashValue, version: Version) -> Self {
        Self {
            epoch: block.epoch(),
            round: block.round(),
            id: block.id(),
            executed_state_id,
            version,
            timestamp_usecs: block.timestamp_usecs(),
        }
    }

    pub fn random(round: Round) -> Self {
        Self {
            epoch: 0,
            round,
            id: HashValue::zero(),
            executed_state_id: HashValue::zero(),
            version: 0,
            timestamp_usecs: 0,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn executed_state_id(&self) -> HashValue {
        self.executed_state_id
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.timestamp_usecs
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

impl From<BlockInfo> for network::proto::BlockInfo {
    fn from(block_info: BlockInfo) -> Self {
        Self {
            epoch: block_info.epoch(),
            round: block_info.round(),
            id: block_info.id().to_vec(),
            executed_state_id: block_info.executed_state_id().to_vec(),
            version: block_info.version(),
            timestamp_usecs: block_info.timestamp_usecs(),
        }
    }
}

impl TryFrom<network::proto::BlockInfo> for BlockInfo {
    type Error = Error;

    fn try_from(proto: network::proto::BlockInfo) -> Result<Self> {
        Ok(BlockInfo::new(
            proto.epoch,
            proto.round,
            HashValue::from_slice(&proto.id)?,
            HashValue::from_slice(&proto.executed_state_id)?,
            proto.version,
            proto.timestamp_usecs,
        ))
    }
}
