// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use itertools::Itertools;
use spec_lang::{
    env::{
        FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, ModuleId, SpecFunId, StructEnv, StructId,
        SCRIPT_MODULE_NAME,
    },
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};

/// Return boogie name of given module.
pub fn boogie_module_name(env: &ModuleEnv<'_>) -> String {
    let name = env.symbol_pool().string(env.get_name().name());
    if name.as_str() == SCRIPT_MODULE_NAME {
        // <SELF> is not accepted by boogie as a symbol
        "#SELF#".to_string()
    } else {
        name.to_string()
    }
}

/// Return boogie name of given structure.
pub fn boogie_struct_name(env: &StructEnv<'_>) -> String {
    format!(
        "${}_{}",
        boogie_module_name(&env.module_env),
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
    let name = format!(
        "${}_{}",
        boogie_module_name(&env.module_env),
        env.get_name().display(env.symbol_pool())
    );
    // TODO: hack to deal with similar native functions in old/new library. We identify
    // whether the old or new version of the function is referenced by the number of type
    // parameters.
    if name == "$LibraAccount_save_account" && env.get_type_parameters().len() == 1 {
        name + "_OLD"
    } else {
        name
    }
}

/// Return boogie name of given spec var.
pub fn boogie_spec_var_name(env: &ModuleEnv<'_>, name: Symbol) -> String {
    format!(
        "${}_{}",
        boogie_module_name(env),
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
        boogie_module_name(env),
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
            PrimitiveType::Address => "AddressType()".to_string(),
            // TODO fix this for a real boogie type
            PrimitiveType::Signer => "AddressType()".to_string(),
            PrimitiveType::Range => "RangeType()".to_string(),
            PrimitiveType::TypeValue => "$TypeType()".to_string(),
        },
        Type::Vector(t) => format!("$Vector_type_value({})", boogie_type_value(env, t)),
        Type::Reference(_, t) => format!("ReferenceType({})", boogie_type_value(env, t)),
        Type::TypeParameter(index) => format!("$tv{}", index),
        Type::TypeLocal(s) => format!("t#$Type({})", s.display(env.symbol_pool())),
        Type::Struct(module_id, struct_id, args) => {
            boogie_struct_type_value(env, *module_id, *struct_id, args)
        }
        // TODO: function and tuple types?
        Type::Tuple(_args) => "Tuple_type_value()".to_string(),
        Type::Fun(_args, _result) => "Function_type_value()".to_string(),
        Type::Error | Type::Var(..) | Type::TypeDomain(..) => panic!("unexpected transient type"),
    }
}

/// Create boogie type value for a struct with given type actuals.
pub fn boogie_struct_type_value(
    env: &GlobalEnv,
    module_id: ModuleId,
    struct_id: StructId,
    args: &[Type],
) -> String {
    let struct_env = env.get_module(module_id).into_struct(struct_id);
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

/// A value indicating how to perform well-formed checks.
#[derive(Clone, Copy, PartialEq)]
pub enum WellFormedMode {
    /// Assume types and invariants in auto mode. If the type is a mutable reference, invariants
    /// will not be assumed.
    Default,
    /// Assume types and invariants.
    WithInvariant,
    /// Assume types only.
    WithoutInvariant,
}

/// Create boogie well-formed boolean expression.
pub fn boogie_well_formed_expr(
    env: &GlobalEnv,
    name: &str,
    ty: &Type,
    mode: WellFormedMode,
) -> String {
    boogie_well_formed_expr_impl(env, name, ty, mode, 0)
}

fn boogie_well_formed_expr_impl(
    env: &GlobalEnv,
    name: &str,
    ty: &Type,
    mode: WellFormedMode,
    nest: usize,
) -> String {
    let mut conds = vec![];
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::U8 => conds.push(format!("$IsValidU8({})", name)),
            PrimitiveType::U64 => conds.push(format!("$IsValidU64({})", name)),
            PrimitiveType::U128 => conds.push(format!("$IsValidU128({})", name)),
            PrimitiveType::Num => conds.push(format!("$IsValidNum({})", name)),
            PrimitiveType::Bool => conds.push(format!("is#Boolean({})", name)),
            PrimitiveType::Address => conds.push(format!("is#Address({})", name)),
            // TODO fix this for a real boogie check
            PrimitiveType::Signer => conds.push(format!("is#Address({})", name)),
            PrimitiveType::Range => conds.push(format!("$IsValidRange({})", name)),
            PrimitiveType::TypeValue => conds.push(format!("is#$Type({})", name)),
        },
        Type::Vector(elem_ty) => {
            conds.push(format!("$Vector_is_well_formed({})", name));
            if !matches!(**elem_ty, Type::TypeParameter(..)) {
                let nest_value = &format!("$vmap({})[$${}]", name, nest);
                conds.push(format!(
                    "(forall $${}: int :: {{{}}} $${} >= 0 && $${} < $vlen({}) ==> {})",
                    nest,
                    nest_value,
                    nest,
                    nest,
                    name,
                    boogie_well_formed_expr_impl(env, nest_value, &elem_ty, mode, nest + 1)
                ));
            }
        }
        Type::Struct(module_idx, struct_idx, _) => {
            let struct_env = env.get_module(*module_idx).into_struct(*struct_idx);
            let well_formed_name = if mode == WellFormedMode::WithoutInvariant {
                "is_well_formed_types"
            } else {
                "is_well_formed"
            };
            conds.push(format!(
                "{}_{}({})",
                boogie_struct_name(&struct_env),
                well_formed_name,
                name
            ))
        }
        Type::Reference(is_mut, rtype) => {
            let mode = if *is_mut && mode == WellFormedMode::Default {
                WellFormedMode::WithoutInvariant
            } else {
                mode
            };
            conds.push(boogie_well_formed_expr_impl(
                env,
                &format!("$Dereference({})", name),
                rtype,
                mode,
                nest + 1,
            ));
        }
        // TODO: tuple and functions?
        Type::Fun(_args, _result) => {}
        Type::Tuple(_elems) => {}
        // A type parameter or type value is opaque, so no type check here.
        Type::TypeParameter(..) | Type::TypeLocal(..) => {}
        Type::Error | Type::Var(..) | Type::TypeDomain(..) => panic!("unexpected transient type"),
    }
    conds.iter().filter(|s| !s.is_empty()).join(" && ")
}

/// Create boogie well-formed check. The result will be either an empty string or a
/// newline-terminated assume statement.
pub fn boogie_well_formed_check(
    env: &GlobalEnv,
    name: &str,
    ty: &Type,
    mode: WellFormedMode,
) -> String {
    let expr = boogie_well_formed_expr(env, name, ty, mode);
    if !expr.is_empty() {
        format!("assume {};\n", expr)
    } else {
        "".to_string()
    }
}

/// Create boogie well-formed preconditions. The result will be either an empty
/// string or a newline-terminated requires statement.
pub fn boogie_requires_well_formed(
    env: &GlobalEnv,
    name: &str,
    ty: &Type,
    mode: WellFormedMode,
    type_requires_str: &str,
) -> String {
    let expr = boogie_well_formed_expr(env, name, ty, mode);
    if !expr.is_empty() {
        format!("{} {};\n", type_requires_str, expr)
    } else {
        "".to_string()
    }
}

/// Create boogie global variable with type constraint. No references allowed.
pub fn boogie_declare_global(env: &GlobalEnv, name: &str, param_count: usize, ty: &Type) -> String {
    let declarator = boogie_global_declarator(env, name, param_count, ty);
    assert!(!ty.is_reference());
    if param_count > 0 {
        let var_selector = format!(
            "{}[{}]",
            name,
            (0..param_count).map(|i| format!("$tv{}", i)).join(", ")
        );
        let type_check = boogie_well_formed_expr(env, &var_selector, ty, WellFormedMode::Default);
        format!(
            "var {} where (forall {} :: {});",
            declarator,
            (0..param_count)
                .map(|i| format!("$tv{}: TypeValue", i))
                .join(", "),
            type_check
        )
    } else {
        format!(
            "var {} where {};",
            declarator,
            boogie_well_formed_expr(env, name, ty, WellFormedMode::Default)
        )
    }
}

pub fn boogie_global_declarator(
    _env: &GlobalEnv,
    name: &str,
    param_count: usize,
    ty: &Type,
) -> String {
    assert!(!ty.is_reference());
    if param_count > 0 {
        format!(
            "{} : [{}]Value",
            name,
            (0..param_count).map(|_| "TypeValue").join(", ")
        )
    } else {
        format!("{} : Value", name)
    }
}

pub fn boogie_byte_blob(val: &[u8]) -> String {
    let mut res = "$mk_vector()".to_string();
    for b in val {
        res = format!("$push_back_vector({}, Integer({}))", res, b);
    }
    res
}
