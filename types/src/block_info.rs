// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{epoch_state::EpochState, on_chain_config::ValidatorSet, transaction::Version};
use libra_crypto::hash::HashValue;
#[cfg(any(test, feature = "fuzzing"))]
use libra_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically.
pub type Round = u64;

// Constants for the initial genesis block.
pub const GENESIS_EPOCH: u64 = 0;
pub const GENESIS_ROUND: Round = 0;
pub const GENESIS_VERSION: Version = 0;
pub const GENESIS_TIMESTAMP_USECS: u64 = 0;

/// This structure contains all the information needed for tracking a block
/// without having access to the block or its execution output state. It
/// assumes that the block is the last block executed within the ledger.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct BlockInfo {
    /// Epoch number corresponds to the set of validators that are active for this block.
    epoch: u64,
    /// The consensus protocol is executed in rounds, which monotonically increase per epoch.
    round: Round,
    /// The identifier (hash) of the block.
    id: HashValue,
    /// The accumulator root hash after executing this block.
    executed_state_id: HashValue,
    /// The version of the latest transaction after executing this block.
    version: Version,
    /// The timestamp this block was proposed by a proposer.
    timestamp_usecs: u64,
    /// An optional field containing the next epoch info
    next_epoch_state: Option<EpochState>,
}

impl BlockInfo {
    pub fn new(
        epoch: u64,
        round: Round,
        id: HashValue,
        executed_state_id: HashValue,
        version: Version,
        timestamp_usecs: u64,
        next_epoch_state: Option<EpochState>,
    ) -> Self {
        Self {
            epoch,
            round,
            id,
            executed_state_id,
            version,
            timestamp_usecs,
            next_epoch_state,
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
            next_epoch_state: None,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn random(round: Round) -> Self {
        Self {
            epoch: 1,
            round,
            id: HashValue::zero(),
            executed_state_id: HashValue::zero(),
            version: 0,
            timestamp_usecs: 0,
            next_epoch_state: None,
        }
    }

    /// Create a new genesis block. The genesis block is effectively the
    /// blockchain state after executing the initial genesis transaction.
    ///
    /// * `genesis_state_root_hash` - the state tree root hash after executing the
    /// initial genesis transaction.
    ///
    /// * `validator_set` - the initial validator set, configured when generating
    /// the genesis transaction itself and emitted after executing the genesis
    /// transaction. Using this genesis block means transitioning to a new epoch
    /// (GENESIS_EPOCH + 1) with this `validator_set`.
    pub fn genesis(genesis_state_root_hash: HashValue, validator_set: ValidatorSet) -> Self {
        Self {
            epoch: GENESIS_EPOCH,
            round: GENESIS_ROUND,
            id: HashValue::zero(),
            executed_state_id: genesis_state_root_hash,
            version: GENESIS_VERSION,
            timestamp_usecs: GENESIS_TIMESTAMP_USECS,
            next_epoch_state: Some(EpochState {
                epoch: 1,
                verifier: (&validator_set).into(),
            }),
        }
    }

    /// Create a mock genesis `BlockInfo` with an empty state tree and empty
    /// validator set.
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn mock_genesis(validator_set: Option<ValidatorSet>) -> Self {
        let validator_set = validator_set.unwrap_or_else(ValidatorSet::empty);
        Self::genesis(*ACCUMULATOR_PLACEHOLDER_HASH, validator_set)
    }

    /// The epoch after this block committed
    pub fn next_block_epoch(&self) -> u64 {
        self.next_epoch_state().map_or(self.epoch(), |e| e.epoch)
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn executed_state_id(&self) -> HashValue {
        self.executed_state_id
    }

    pub fn has_reconfiguration(&self) -> bool {
        self.next_epoch_state.is_some()
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn next_epoch_state(&self) -> Option<&EpochState> {
        self.next_epoch_state.as_ref()
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

impl Display for BlockInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "BlockInfo: [epoch: {}, round: {}, id: {}, executed_state_id: {}, version: {}, timestamp (us): {}, next_epoch_state: {}]",
            self.epoch(),
            self.round(),
            self.id(),
            self.executed_state_id(),
            self.version(),
            self.timestamp_usecs(),
            self.next_epoch_state.as_ref().map_or("None".to_string(), |epoch_state| format!("{}", epoch_state)),
        )
    }
}
