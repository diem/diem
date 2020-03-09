// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use itertools::Itertools;
use spec_lang::env::{
    FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, ModuleId, SpecFunId, StructEnv, StructId,
};
use spec_lang::symbol::Symbol;
use spec_lang::ty::{PrimitiveType, Type};

/// Return boogie name of given structure.
pub fn boogie_struct_name(env: &StructEnv<'_>) -> String {
    format!(
        "${}_{}",
        env.module_env.get_name().name().display(env.symbol_pool()),
        env.get_name().display(env.symbol_pool())
    )
}

/// Return boogie name of given field.
pub fn boogie_field_name(env: &FieldEnv<'_>) -> String {
    format!(
        "{}_{}",
        boogie_struct_name(&env.struct_env),
        env.get_name().display(env.struct_env.symbol_pool())
    )
}

/// Return boogie name of given function.
pub fn boogie_function_name(env: &FunctionEnv<'_>) -> String {
    format!(
        "${}_{}",
        env.module_env.get_name().name().display(env.symbol_pool()),
        env.get_name().display(env.symbol_pool())
    )
}

/// Return boogie name of given spec var.
pub fn boogie_spec_var_name(env: &ModuleEnv<'_>, name: Symbol) -> String {
    format!(
        "${}_{}",
        env.get_name().name().display(env.symbol_pool()),
        name.display(env.symbol_pool())
    )
}

/// Return boogie name of given spec function.
pub fn boogie_spec_fun_name(env: &ModuleEnv<'_>, id: SpecFunId) -> String {
    let decl = env.get_spec_fun(id);
    let pos = env
        .get_spec_funs_of_name(decl.name)
        .position(|(overload_id, _)| &id == overload_id)
        .expect("spec fun env inconsistent");
    let overload_qualifier = if pos > 0 {
        format!("_{}", pos)
    } else {
        "".to_string()
    };
    format!(
        "${}_{}{}",
        env.get_name().name().display(env.symbol_pool()),
        decl.name.display(env.symbol_pool()),
        overload_qualifier
    )
}

/// Create boogie type value from signature token.
pub fn boogie_type_value(env: &GlobalEnv, ty: &Type) -> String {
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::Bool => "BooleanType()".to_string(),
            PrimitiveType::U8 | PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::Num => {
                "IntegerType()".to_string()
            }
            PrimitiveType::ByteArray => "ByteArrayType()".to_string(),
            PrimitiveType::Address => "AddressType()".to_string(),
            PrimitiveType::Range => "RangeType()".to_string(),
        },
        Type::Vector(t) => format!("$Vector_type_value({})", boogie_type_value(env, t)),
        Type::Reference(_, t) => format!("ReferenceType({})", boogie_type_value(env, t)),
        Type::TypeParameter(index) => format!("$tv{}", index),
        Type::Struct(module_id, struct_id, args) => {
            boogie_struct_type_value(env, *module_id, *struct_id, args)
        }
        // TODO: function and tuple types?
        Type::Tuple(_args) => "Tuple_type_value()".to_string(),
        Type::Fun(_args, _result) => "Function_type_value()".to_string(),
        _ => panic!("unexpected transient type"),
    }
}

/// Create boogie type value for a struct with given type actuals.
pub fn boogie_struct_type_value(
    env: &GlobalEnv,
    module_id: ModuleId,
    struct_id: StructId,
    args: &[Type],
) -> String {
    let module_env = env.get_module(module_id);
    let struct_env = module_env.get_struct(struct_id);
    format!(
        "{}_type_value({})",
        boogie_struct_name(&struct_env),
        boogie_type_values(env, args)
    )
}

/// Create boogie type value list, separated by comma.
pub fn boogie_type_values(env: &GlobalEnv, args: &[Type]) -> String {
    args.iter()
        .map(|arg| boogie_type_value(env, arg))
        .join(", ")
}

/// Return boogie type for a local with given signature token.
pub fn boogie_local_type(ty: &Type) -> String {
    if ty.is_reference() {
        "Reference".to_string()
    } else {
        "Value".to_string()
    }
}

/// Create boogie type check boolean expression.
pub fn boogie_type_check_expr(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    let mut conds = vec![];
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::U8 => conds.push(format!("$IsValidU8({})", name)),
            PrimitiveType::U64 => conds.push(format!("$IsValidU64({})", name)),
            PrimitiveType::U128 => conds.push(format!("$IsValidU128({})", name)),
            PrimitiveType::Num => conds.push(format!("$IsValidNum({})", name)),
            PrimitiveType::Bool => conds.push(format!("is#Boolean({})", name)),
            PrimitiveType::Address => conds.push(format!("is#Address({})", name)),
            PrimitiveType::ByteArray => conds.push(format!("is#ByteArray({})", name)),
            PrimitiveType::Range => conds.push(format!("$IsValidRange({})", name)),
        },
        Type::Vector(_) => conds.push(format!("$Vector_is_well_formed({})", name)),
        Type::Struct(module_idx, struct_idx, _) => {
            let struct_env = env.get_module(*module_idx).into_struct(*struct_idx);
            conds.push(format!(
                "${}_is_well_formed({})",
                boogie_struct_name(&struct_env),
                name
            ))
        }
        Type::Reference(_is_mut, rtype) => {
            conds.push(boogie_type_check_expr(
                env,
                &format!("$Dereference($m, {})", name),
                rtype,
            ));
            conds.push(format!(
                "$IsValidReferenceParameter($m, $local_counter, {})",
                name
            ));
        }
        // TODO: tuple and functions?
        Type::Fun(_args, _result) => {}
        Type::Tuple(_elems) => {}
        // A type parameter is opaque, so no type check here.
        Type::TypeParameter(_) => {}
        _ => panic!("unexpected transient type"),
    }
    conds.iter().filter(|s| !s.is_empty()).join(" && ")
}

/// Create boogie type check assumption. The result will be either an empty string or a
/// newline-terminated assume statement.
pub fn boogie_type_check(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    let expr = boogie_type_check_expr(env, name, ty);
    if !expr.is_empty() {
        format!("assume {};\n", expr)
    } else {
        "".to_string()
    }
}

/// Create boogie global variable with type constraint. No references allowed.
pub fn boogie_declare_global(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    assert!(!ty.is_reference());
    format!(
        "var {} : Value where {};",
        name,
        boogie_type_check_expr(env, name, ty)
    )
}

pub fn boogie_var_before_borrow(idx: usize) -> String {
    format!("$before_borrow_{}", idx)
}
