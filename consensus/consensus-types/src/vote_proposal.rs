// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::Block;
use failure::prelude::{Error, Result};
use libra_crypto::hash::HashValue;
use libra_types::{transaction::Version, validator_set::ValidatorSet};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
};

/// This structure contains all the information needed by safety rules to
/// evaluate a proposal / block for correctness / safety and to produce a Vote.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VoteProposal<T> {
    /// The block / proposal to evaluate
    #[serde(bound(deserialize = "Block<T>: Deserialize<'de>"))]
    block: Block<T>,
    /// The accumulator root hash after executing this block.
    executed_state_id: HashValue,
    /// The version of the latest transaction in the ledger.
    version: Version,
    /// An optional field containing the set of validators for the start of the next epoch
    next_validator_set: Option<ValidatorSet>,
}

impl<T> VoteProposal<T> {
    pub fn new(
        block: Block<T>,
        executed_state_id: HashValue,
        version: Version,
        next_validator_set: Option<ValidatorSet>,
    ) -> Self {
        Self {
            block,
            executed_state_id,
            version,
            next_validator_set,
        }
    }

    pub fn block(&self) -> &Block<T> {
        &self.block
    }

    pub fn executed_state_id(&self) -> HashValue {
        self.executed_state_id
    }

    pub fn next_validator_set(&self) -> Option<&ValidatorSet> {
        self.next_validator_set.as_ref()
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

impl<T: PartialEq> Display for VoteProposal<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "VoteProposal[block: {}, executed_state_id: {}, version: {}, next_validator_set: {}]",
            self.block,
            self.executed_state_id,
            self.version,
            self.next_validator_set
                .as_ref()
                .map_or("None".to_string(), |validator_set| format!(
                    "{}",
                    validator_set
                )),
        )
    }
}

impl<T> TryFrom<VoteProposal<T>> for network::proto::VoteProposal
where
    T: Serialize + Default + PartialEq,
{
    type Error = Error;

    fn try_from(vote_proposal: VoteProposal<T>) -> Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&vote_proposal)?,
        })
    }
}

impl<T> TryFrom<network::proto::VoteProposal> for VoteProposal<T>
where
    T: DeserializeOwned + Serialize,
{
    type Error = Error;

    fn try_from(proto: network::proto::VoteProposal) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}
