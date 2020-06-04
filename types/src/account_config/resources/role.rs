// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::{
        constants::{ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS},
        resources::{ChildVASP, ParentVASP},
    },
};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountRole {
    ParentVASP(ParentVASP),
    ChildVASP(ChildVASP),
    Empty,
    Unhosted,
    Unknown,
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

    pub fn is_empty(&self) -> bool {
        match self {
            AccountRole::Empty => true,
            _ => false,
        }
    }

    pub fn access_path_for<Role: MoveResource>() -> Vec<u8> {
        let role_type_tag = TypeTag::Struct(Role::struct_tag());
        let tag = StructTag {
            address: CORE_CODE_ADDRESS,
            name: AccountRole::struct_identifier(),
            module: AccountRole::module_identifier(),
            type_params: vec![role_type_tag],
        };
        AccessPath::resource_access_vec(&tag)
    }
}

impl MoveResource for AccountRole {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Role";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentVASPRole {
    pub role: ParentVASP,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildVASPRole {
    pub role: ChildVASP,
}

// Empty Move resources have a dummy boolean field
#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyRole {
    _dummy: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnhostedRole {
    _dummy: bool,
}

impl MoveResource for EmptyRole {
    const MODULE_NAME: &'static str = "Empty";
    const STRUCT_NAME: &'static str = "Empty";
}

impl MoveResource for UnhostedRole {
    const MODULE_NAME: &'static str = "Unhosted";
    const STRUCT_NAME: &'static str = "Unhosted";
}
