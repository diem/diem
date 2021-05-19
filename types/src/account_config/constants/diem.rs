// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use anyhow::{bail, ensure, Result};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
};
use once_cell::sync::Lazy;

pub use move_core_types::vm_status::known_locations::{DIEM_MODULE, DIEM_MODULE_IDENTIFIER};

const COIN_MODULE_IDENTIFIER: &IdentStr = DIEM_MODULE_IDENTIFIER;
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, COIN_MODULE_IDENTIFIER.to_owned()));

// TODO: This imposes a few implied restrictions:
//   1) The currency module must be published under the core code address.
//   2) The module name must be the same as the gas specifier.
//   3) The struct name must be the same as the module name
// We need to consider whether we want to switch to a more or fully qualified name.
pub fn type_tag_for_currency_code(currency_code: Identifier) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: currency_code.clone(),
        name: currency_code,
        type_params: vec![],
    })
}

pub fn currency_code_from_type_tag(type_tag: TypeTag) -> Result<Identifier> {
    let struct_tag = match type_tag {
        TypeTag::Struct(struct_tag) => struct_tag,
        _ => bail!(
            "expected a type for a currency struct, received: {:?}",
            type_tag
        ),
    };

    ensure!(
        struct_tag.address == CORE_CODE_ADDRESS
            && struct_tag.module == struct_tag.name
            && struct_tag.type_params.is_empty(),
        "not a valid currency struct tag: {:?}",
        struct_tag,
    );

    let currency_code = struct_tag.name;

    ensure!(
        allowed_currency_code_string(currency_code.as_str()),
        "currency code can only contain uppercase alphanumeric characters: '{}'",
        currency_code,
    );

    Ok(currency_code)
}

/// In addition to the constraints for valid Move identifiers, currency codes
/// should consist entirely of uppercase alphanumeric characters (e.g., no underscores).
pub fn allowed_currency_code_string(possible_currency_code_string: &str) -> bool {
    possible_currency_code_string
        .chars()
        .all(|chr| matches!(chr, 'A'..='Z' | '0'..='9'))
        && Identifier::is_valid(possible_currency_code_string)
}

pub fn from_currency_code_string(currency_code_string: &str) -> Result<Identifier> {
    if !allowed_currency_code_string(currency_code_string) {
        bail!("Invalid currency code '{}'", currency_code_string)
    }
    Identifier::new(currency_code_string)
}
