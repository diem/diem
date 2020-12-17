// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Core configuration for x.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct XCoreConfig {
    /// Subsets of this workspace
    pub subsets: BTreeMap<String, SubsetConfig>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SubsetConfig {
    /// The root members in this subset
    pub root_members: Vec<String>,
}
