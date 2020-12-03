// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Diem Validator software.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DiemVersion {
    pub major: u64,
}

impl OnChainConfig for DiemVersion {
    const IDENTIFIER: &'static str = "DiemVersion";
}
