// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the travel rule limit in microLibra LBR above which the travel rule is enacted
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DualAttestationLimit {
    pub micro_lbr_limit: u64,
}

impl OnChainConfig for DualAttestationLimit {
    const IDENTIFIER: &'static str = "DualAttestationLimit";
}
