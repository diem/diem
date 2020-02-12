// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use crate::env::{FieldEnv, FunctionEnv, GlobalEnv, GlobalType, ModuleEnv, StructEnv};
use itertools::Itertools;
use vm::file_format::StructDefinitionIndex;

/// Return boogie name of given structure.
pub fn boogie_struct_name(env: &StructEnv<'_>) -> String {
    format!("{}_{}", env.module_env.get_id().name(), env.get_name())
}

/// Return boogie name of given field.
pub fn boogie_field_name(env: &FieldEnv<'_>) -> String {
    format!("{}_{}", boogie_struct_name(&env.struct_env), env.get_name())
}

/// Return boogie name of given function.
pub fn boogie_function_name(env: &FunctionEnv<'_>) -> String {
    format!("{}_{}", env.module_env.get_id().name(), env.get_name())
}

/// Return boogie name of given function.
pub fn boogie_synthetic_name(env: &ModuleEnv<'_>, name: &str) -> String {
    format!("__{}_{}", env.get_id().name(), name)
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
        // TODO: Need a boogie type for subrange?
        GlobalType::Subrange => "Unimplemented boogie subrange".to_string(),
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

/// Create boogie type check boolean expression.
pub fn boogie_type_check_expr(env: &GlobalEnv, name: &str, sig: &GlobalType) -> String {
    let mut conds = vec![];
    //let mut params = name.to_string();
    //let mut ret = String::new();
    match sig {
        GlobalType::U8 => conds.push(format!("IsValidU8({})", name)),
        GlobalType::U64 => conds.push(format!("IsValidU64({})", name)),
        GlobalType::U128 => conds.push(format!("IsValidU128({})", name)),
        GlobalType::Bool => conds.push(format!("is#Boolean({})", name)),
        GlobalType::Address => conds.push(format!("is#Address({})", name)),
        GlobalType::ByteArray => conds.push(format!("is#ByteArray({})", name)),
        GlobalType::Struct(module_idx, struct_idx, _) => {
            let struct_env = env.get_module(*module_idx).into_get_struct(struct_idx);
            conds.push(format!(
                "${}_is_well_formed({})",
                boogie_struct_name(&struct_env),
                name
            ))
        }
        GlobalType::Reference(rtype) | GlobalType::MutableReference(rtype) => {
            conds.push(boogie_type_check_expr(
                env,
                &format!("Dereference(__m, {})", name),
                rtype,
            ));
            conds.push(format!(
                "IsValidReferenceParameter(__m, __local_counter, {})",
                name
            ));
        }
        // TODO: IsValidSubrange in Boogie should check that args are U64's.
        // Ok if lb > ub -- subrange is empty in that case.
        GlobalType::Subrange => {
            format!("IsValidSubrange({})", name);
        }
        // Otherwise it is a type parameter which is opaque
        GlobalType::TypeParameter(_) => {}
    }
    conds.iter().filter(|s| !s.is_empty()).join(" && ")
}

/// Create boogie type check assumption. The result will be either an empty string or a
/// newline-terminated assume statement.
pub fn boogie_type_check(env: &GlobalEnv, name: &str, sig: &GlobalType) -> String {
    let expr = boogie_type_check_expr(env, name, sig);
    if !expr.is_empty() {
        format!("assume {};\n", expr)
    } else {
        "".to_string()
    }
}

/// Create boogie global variable with type constraint. No references allowed.
pub fn boogie_declare_global(env: &GlobalEnv, name: &str, sig: &GlobalType) -> String {
    assert!(!sig.is_reference());
    format!(
        "var {} : Value where {};",
        name,
        boogie_type_check_expr(env, name, sig)
    )
}

/// Returns the name of the invariant target value.
pub fn boogie_invariant_target(for_old: bool) -> String {
    if for_old {
        "__inv_target_old".to_string()
    } else {
        "__inv_target".to_string()
    }
}

pub fn boogie_var_before_borrow(idx: usize) -> String {
    format!("__before_borrow_{}", idx)
}
