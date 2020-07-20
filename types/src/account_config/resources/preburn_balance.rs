// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::constants::{lbr_type_tag, CORE_CODE_ADDRESS, LIBRA_MODULE_NAME},
};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

/// The preburn balance held under an account.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct PreburnResource {
    coin: u64,
}

impl PreburnResource {
    pub fn new(coin: u64) -> Self {
        Self { coin }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_currency(currency_typetag: TypeTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: PreburnResource::struct_identifier(),
            module: PreburnResource::module_identifier(),
            type_params: vec![currency_typetag],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(currency_typetag: TypeTag) -> Vec<u8> {
        AccessPath::resource_access_vec(&PreburnResource::struct_tag_for_currency(currency_typetag))
    }
}

impl MoveResource for PreburnResource {
    const MODULE_NAME: &'static str = LIBRA_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Preburn";

    fn type_params() -> Vec<TypeTag> {
        vec![lbr_type_tag()]
    }
}
