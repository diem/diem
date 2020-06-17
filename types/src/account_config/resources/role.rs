// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::resources::{ChildVASP, ParentVASP};
use serde::{Deserialize, Serialize};

/// A enum that captures the collection of role-specific resources stored under each account type
#[derive(Debug, Serialize, Deserialize)]
pub enum AccountRole {
    ParentVASP(ParentVASP),
    ChildVASP(ChildVASP),
    Unhosted,
    Unknown,
    // TODO: add other roles
}
