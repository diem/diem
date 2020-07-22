// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use heck::CamelCase;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_reflection::{ContainerFormat, Format, Named, VariantFormat};
use std::collections::BTreeMap;

/// Useful error message.
pub(crate) fn type_not_allowed(type_tag: &TypeTag) -> ! {
    panic!(
        "Transaction scripts cannot take arguments of type {}.",
        type_tag
    );
}

/// Clean up doc comments extracter by the Move prover.
pub(crate) fn prepare_doc_string(doc: &str) -> String {
    doc.replace("\n ", "\n").trim().to_string()
}

fn quote_type_as_format(type_tag: &TypeTag) -> Format {
    use TypeTag::*;
    match type_tag {
        Bool => Format::Bool,
        U8 => Format::U8,
        U64 => Format::U64,
        U128 => Format::U128,
        Address => Format::TypeName("AccountAddress".into()),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => Format::Bytes,
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn quote_type_parameter_as_field(ty_arg: &TypeArgumentABI) -> Named<Format> {
    Named {
        name: ty_arg.name().to_string(),
        value: Format::TypeName("TypeTag".into()),
    }
}

fn quote_parameter_as_field(arg: &ArgumentABI) -> Named<Format> {
    Named {
        name: arg.name().to_string(),
        value: quote_type_as_format(arg.type_tag()),
    }
}

pub(crate) fn make_abi_enum_container(abis: &[ScriptABI]) -> ContainerFormat {
    let mut variants = BTreeMap::new();
    for (index, abi) in abis.iter().enumerate() {
        let mut fields = Vec::new();
        for ty_arg in abi.ty_args() {
            fields.push(quote_type_parameter_as_field(ty_arg));
        }
        for arg in abi.args() {
            fields.push(quote_parameter_as_field(arg));
        }
        let format = VariantFormat::Struct(fields);
        variants.insert(
            index as u32,
            Named {
                name: abi.name().to_camel_case(),
                value: format,
            },
        );
    }
    ContainerFormat::Enum(variants)
}
