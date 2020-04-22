// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Request to get a EpochChangeProof from current_epoch to target_epoch
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EpochRetrievalRequest {
    pub start_epoch: u64,
    pub end_epoch: u64,
}
