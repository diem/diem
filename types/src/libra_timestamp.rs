// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    language_storage::StructTag,
};
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static LIBRA_TIMESTAMP_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraTimestamp").unwrap());
static LIBRA_TIMESTAMP_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("CurrentTimeMicroseconds").unwrap());

pub fn libra_timestamp_module_name() -> &'static IdentStr {
    &*LIBRA_TIMESTAMP_MODULE_NAME
}

pub fn libra_timestamp_struct_name() -> &'static IdentStr {
    &*LIBRA_TIMESTAMP_STRUCT_NAME
}

pub fn libra_timestamp_tag() -> StructTag {
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        name: libra_timestamp_struct_name().to_owned(),
        module: libra_timestamp_module_name().to_owned(),
        type_params: vec![],
    }
}

/// The access path where the Validator Set resource is stored.
pub static LIBRA_TIMESTAMP_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&libra_timestamp_tag(), &Accesses::empty()));

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraTimestampResource {
    pub libra_timestamp: LibraTimestamp,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraTimestamp {
    pub microseconds: u64,
}
