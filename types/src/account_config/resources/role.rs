// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::resources::{
    ChildVASP, Credential, DesignatedDealer, ParentVASP, PreburnResource,
};
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::BTreeMap;

/// A enum that captures the collection of role-specific resources stored under each account type
#[derive(Debug, Serialize, Deserialize)]
pub enum AccountRole {
    ParentVASP {
        vasp: ParentVASP,
        credential: Credential,
    },
    ChildVASP(ChildVASP),
    DesignatedDealer {
        dd_credential: Credential,
        preburn_balances: BTreeMap<Identifier, PreburnResource>,
        designated_dealer: DesignatedDealer,
    },
    Unknown,
    // TODO: add other roles
}
