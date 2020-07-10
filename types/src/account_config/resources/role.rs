// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::resources::{ChildVASP, Credential, ParentVASP};
use serde::{Deserialize, Serialize};

/// A enum that captures the collection of role-specific resources stored under each account type
#[derive(Debug, Serialize, Deserialize)]
pub enum AccountRole {
    ParentVASP {
        vasp: ParentVASP,
        credential: Credential,
    },
    ChildVASP(ChildVASP),
    DesignatedDealer(Credential), // TODO: add DesignatedDealer resource as well
    Unhosted,
    Unknown,
    // TODO: add other roles
}
