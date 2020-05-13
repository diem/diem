// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use std::convert::TryFrom;

use libra_types::on_chain_config::ValidatorSet;

impl TryFrom<crate::proto::types::ValidatorSet> for ValidatorSet {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorSet) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl From<ValidatorSet> for crate::proto::types::ValidatorSet {
    fn from(set: ValidatorSet) -> Self {
        Self {
            bytes: lcs::to_bytes(&set).expect("failed to serialize validator set"),
        }
    }
}
