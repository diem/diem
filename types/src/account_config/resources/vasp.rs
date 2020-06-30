// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentVASP {
    human_name: String,
    base_url: String,
    expiration_date: u64,
    compliance_public_key: Vec<u8>,
    num_children: u64,
}

impl ParentVASP {
    pub fn human_name(&self) -> &str {
        &self.human_name
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn expiration_date(&self) -> u64 {
        self.expiration_date
    }

    pub fn compliance_public_key(&self) -> &[u8] {
        &self.compliance_public_key
    }

    pub fn num_children(&self) -> u64 {
        self.num_children
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ChildVASP {
    parent_vasp_addr: AccountAddress,
}

impl ChildVASP {
    pub fn parent_vasp_addr(&self) -> AccountAddress {
        self.parent_vasp_addr
    }
}

impl MoveResource for ParentVASP {
    const MODULE_NAME: &'static str = "VASP";
    const STRUCT_NAME: &'static str = "ParentVASP";
}

impl MoveResource for ChildVASP {
    const MODULE_NAME: &'static str = "VASP";
    const STRUCT_NAME: &'static str = "ChildVASP";
}
