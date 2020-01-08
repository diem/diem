// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{
    account_address::AccountAddress,
    account_config::core_code_address,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
    libra_resource::LibraResource,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    // LibraSystem
    static ref LIBRA_SYSTEM_MODULE_NAME: Identifier = Identifier::new("LibraSystem").unwrap();
    static ref BLOCK_METADATA_STRUCT_NAME: Identifier = Identifier::new("BlockMetadata").unwrap();

}

pub fn libra_system_module_name() -> &'static IdentStr {
    &*LIBRA_SYSTEM_MODULE_NAME
}
pub fn block_metadata_module_name() -> &'static IdentStr {
    &*BLOCK_METADATA_STRUCT_NAME
}

pub fn block_metadata_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: libra_system_module_name().to_owned(),
        name: block_metadata_module_name().to_owned(),
        type_params: vec![],
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockMetaResource {
    pub height: u64,
    pub timestamp: u64,
    pub id: Vec<u8>,
    pub proposer: AccountAddress,
}

impl LibraResource for BlockMetaResource {
    fn struct_tag() -> StructTag {
        block_metadata_struct_tag()
    }
}
