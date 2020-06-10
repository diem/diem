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

impl AccountRole {
    pub fn parent_vasp_data(&self) -> Option<&ParentVASP> {
        match self {
            AccountRole::ParentVASP(vasp) => Some(vasp),
            _ => None,
        }
    }

    pub fn child_vasp_data(&self) -> Option<&ChildVASP> {
        match self {
            AccountRole::ChildVASP(vasp) => Some(vasp),
            _ => None,
        }
    }

    pub fn is_unhosted(&self) -> bool {
        match self {
            AccountRole::Unhosted => true,
            _ => false,
        }
    }
}
