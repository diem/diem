// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    language_storage::{StructTag, TypeTag},
};
use move_core_types::identifier::{IdentStr, Identifier};

pub trait MoveResource {
    const MODULE_NAME: &'static str;
    const STRUCT_NAME: &'static str;

    fn struct_identifier() -> Identifier {
        IdentStr::new(Self::STRUCT_NAME)
            .expect("failed to get IdentStr for Move struct")
            .to_owned()
    }

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            name: Self::struct_identifier(),
            module: IdentStr::new(Self::MODULE_NAME)
                .expect("failed to get IdentStr for Move module")
                .to_owned(),
            type_params: Self::type_params(),
        }
    }

    fn resource_path() -> Vec<u8> {
        AccessPath::resource_access_vec(&Self::struct_tag(), &Accesses::empty())
    }
}
