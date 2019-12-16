// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Request to get a ValidatorChangeProof from current_epoch to target_epoch
#[derive(Serialize, Deserialize)]
pub struct EpochRetrievalRequest {
    pub start_epoch: u64,
    pub end_epoch: u64,
}

impl TryFrom<network::proto::RequestEpoch> for EpochRetrievalRequest {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::RequestEpoch) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<EpochRetrievalRequest> for network::proto::RequestEpoch {
    type Error = anyhow::Error;

    fn try_from(epoch_retrieval_request: EpochRetrievalRequest) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&epoch_retrieval_request)?,
        })
    }
}
