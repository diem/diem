// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Libra Validator software.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LibraVersion {
    pub major: u64,
}

impl OnChainConfig for LibraVersion {
    const IDENTIFIER: &'static str = "LibraVersion";
}
