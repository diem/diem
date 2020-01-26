// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use crate::env::{FieldEnv, FunctionEnv, GlobalEnv, GlobalType, StructEnv};
use itertools::Itertools;
use vm::file_format::StructDefinitionIndex;

/// Return boogie name of given structure.
pub fn boogie_struct_name(env: &StructEnv<'_>) -> String {
    format!("{}_{}", env.module_env.get_id().name(), env.get_name())
}

/// Return boogie name of given field.
pub fn boogie_field_name(env: &FieldEnv<'_>) -> String {
    format!("{}_{}", boogie_struct_name(env.struct_env), env.get_name())
}

/// Return boogie name of given function.
pub fn boogie_function_name(env: &FunctionEnv<'_>) -> String {
    format!("{}_{}", env.module_env.get_id().name(), env.get_name())
}

/// Create boogie type value from signature token.
pub fn boogie_type_value(env: &GlobalEnv, sig: &GlobalType) -> String {
    match sig {
        GlobalType::Bool => "BooleanType()".to_string(),
        GlobalType::U8 | GlobalType::U64 | GlobalType::U128 => "IntegerType()".to_string(),
        GlobalType::ByteArray => "ByteArrayType()".to_string(),
        GlobalType::Address => "AddressType()".to_string(),
        GlobalType::Reference(t) | GlobalType::MutableReference(t) => {
            format!("ReferenceType({})", boogie_type_value(env, &*t))
        }
        GlobalType::TypeParameter(index) => format!("tv{}", index),
        GlobalType::Struct(module_idx, struct_idx, args) => {
            boogie_struct_type_value(env, *module_idx, struct_idx, args)
        }
    }
}

/// Create boogie type value for a struct with given type actuals.
pub fn boogie_struct_type_value(
    env: &GlobalEnv,
    module_idx: usize,
    struct_idx: &StructDefinitionIndex,
    args: &[GlobalType],
) -> String {
    let module_env = env.get_module(module_idx);
    let struct_env = module_env.get_struct(struct_idx);
    format!(
        "{}_type_value({})",
        boogie_struct_name(&struct_env),
        boogie_type_values(env, args)
    )
}

/// Create boogie type value list, separated by comma.
pub fn boogie_type_values(env: &GlobalEnv, args: &[GlobalType]) -> String {
    args.iter()
        .map(|arg| boogie_type_value(env, arg))
        .join(", ")
}

/// Return boogie type for a local with given signature token.
pub fn boogie_local_type(sig: &GlobalType) -> String {
    if sig.is_reference() {
        "Reference".to_string()
    } else {
        "Value".to_string()
    }
}

/// Create boogie type check assumption.
pub fn boogie_type_check(env: &GlobalEnv, name: &str, sig: &GlobalType) -> String {
    let mut params = name.to_string();
    let mut ret = String::new();
    let check = match sig {
        GlobalType::U8 => "IsValidU8",
        GlobalType::U64 => "IsValidU64",
        GlobalType::U128 => "IsValidU128",
        GlobalType::Bool => "is#Boolean",
        GlobalType::Address => "is#Address",
        GlobalType::ByteArray => "is#ByteArray",
        // Only need to check Struct for top-level; fields will be checked as we extract them.
        GlobalType::Struct(_, _, _) => "is#Vector",
        GlobalType::Reference(rtype) | GlobalType::MutableReference(rtype) => {
            let n = format!("Dereference(__m, {})", params);
            ret = boogie_type_check(env, &n, rtype);
            params = format!("__m, __frame, {}", params);
            "IsValidReferenceParameter"
        }
        // Otherwise it is a type parameter which is opaque
        GlobalType::TypeParameter(_) => "",
    };
    let ret2 = if check.is_empty() {
        "".to_string()
    } else {
        format!("assume {}({});\n", check, params)
    };
    ret + &ret2
}
