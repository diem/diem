// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{StructTag, TypeTag},
};
use serde::de::DeserializeOwned;

pub trait MoveStructType {
    const ADDRESS: AccountAddress = crate::language_storage::CORE_CODE_ADDRESS;
    const MODULE_NAME: &'static IdentStr;
    const STRUCT_NAME: &'static IdentStr;

    fn module_identifier() -> Identifier {
        Self::MODULE_NAME.to_owned()
    }

    fn struct_identifier() -> Identifier {
        Self::STRUCT_NAME.to_owned()
    }

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_params: Self::type_params(),
        }
    }
}

pub trait MoveResource: MoveStructType + DeserializeOwned {
    fn resource_path() -> Vec<u8> {
        Self::struct_tag().access_vector()
    }
}
