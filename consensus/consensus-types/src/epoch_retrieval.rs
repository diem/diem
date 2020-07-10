// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::Epoch;
use serde::{Deserialize, Serialize};

/// Request to get a EpochChangeProof from current_epoch to target_epoch
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EpochRetrievalRequest {
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
}
