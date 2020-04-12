// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_config, language_storage::StructTag};
use move_core_types::identifier::IdentStr;

pub trait MoveResource {
    const MODULE_NAME: &'static str;
    const STRUCT_NAME: &'static str;

    fn struct_tag() -> StructTag {
        StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            name: IdentStr::new(Self::STRUCT_NAME)
                .expect("failed to get IdentStr for Move struct")
                .to_owned(),
            module: IdentStr::new(Self::MODULE_NAME)
                .expect("failed to get IdentStr for Move module")
                .to_owned(),
            type_params: vec![],
        }
    }
}
