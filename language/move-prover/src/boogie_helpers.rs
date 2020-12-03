// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use crate::cli::Options;
use itertools::Itertools;
use spec_lang::{
    env::{
        FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, ModuleId, QualifiedId, SpecFunId, StructEnv,
        StructId, SCRIPT_MODULE_NAME,
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
    if name == "$DiemAccount_save_account" && env.get_type_parameters().len() == 1 {
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
            PrimitiveType::Bool => "$BooleanType()".to_string(),
            PrimitiveType::U8 | PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::Num => {
                "$IntegerType()".to_string()
            }
            PrimitiveType::Address => "$AddressType()".to_string(),
            // TODO fix this for a real boogie type
            PrimitiveType::Signer => "$AddressType()".to_string(),
            PrimitiveType::Range => "$RangeType()".to_string(),
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
        Type::Error => panic!("unexpected error type"),
        Type::Var(..) => panic!("unexpected type variable"),
        Type::TypeDomain(..) => panic!("unexpected transient type"),
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

/// Creates the name of the resource memory domain for the caller of any function for the given struct.
/// This variable represents an input parameter of the Boogie translation of this function.
pub fn boogie_caller_resource_memory_domain_name(
    env: &GlobalEnv,
    memory: QualifiedId<StructId>,
) -> String {
    let struct_env = env.get_module(memory.module_id).into_struct(memory.id);
    format!("{}_$CallerDomain", boogie_struct_name(&struct_env))
}

/// Creates the name of the resource memory domain for any function for the given struct.
/// This variable represents a local variable of the Boogie translation of this function.
pub fn boogie_self_resource_memory_domain_name(
    env: &GlobalEnv,
    memory: QualifiedId<StructId>,
) -> String {
    let struct_env = env.get_module(memory.module_id).into_struct(memory.id);
    format!("{}_$SelfDomain", boogie_struct_name(&struct_env))
}

/// Creates the name of the resource memory for the given struct.
pub fn boogie_resource_memory_name(env: &GlobalEnv, memory: QualifiedId<StructId>) -> String {
    let struct_env = env.get_module(memory.module_id).into_struct(memory.id);
    format!("{}_$memory", boogie_struct_name(&struct_env))
}

/// For global update invariants, creates the name where the last resource memory is stored.
pub fn boogie_saved_resource_memory_name(env: &GlobalEnv, memory: QualifiedId<StructId>) -> String {
    format!("{}_$old", boogie_resource_memory_name(env, memory))
}

/// Create boogie type value list, separated by comma.
pub fn boogie_type_values(env: &GlobalEnv, args: &[Type]) -> String {
    args.iter()
        .map(|arg| boogie_type_value(env, arg))
        .join(", ")
}

/// Creates a type value array for given types.
pub fn boogie_type_value_array(env: &GlobalEnv, args: &[Type]) -> String {
    let args = args
        .iter()
        .map(|ty| boogie_type_value(env, ty))
        .collect_vec();
    boogie_type_value_array_from_strings(&args)
}

/// Creates a type value array for types given as strings.
pub fn boogie_type_value_array_from_strings(args: &[String]) -> String {
    if args.is_empty() {
        return "$EmptyTypeValueArray".to_string();
    }
    let mut map = String::from("$MapConstTypeValue($DefaultTypeValue())");
    for (i, arg) in args.iter().enumerate() {
        map = format!("{}[{} := {}]", map, i, arg);
    }
    format!("$TypeValueArray({}, {})", map, args.len())
}

/// Return boogie type for a local with given signature token.
pub fn boogie_local_type(ty: &Type) -> String {
    if ty.is_reference() {
        "$Mutation".to_string()
    } else {
        "$Value".to_string()
    }
}

/// A value indicating how to perform well-formed checks.
#[derive(Clone, Copy, PartialEq)]
pub enum WellFormedMode {
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
    boogie_well_formed_expr_impl(env, name, ty, true, mode, 0)
}

/// Create boogie invariant check boolean expression.
pub fn boogie_inv_expr(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    boogie_well_formed_expr_impl(env, name, ty, false, WellFormedMode::WithInvariant, 0)
}

fn boogie_well_formed_expr_impl(
    env: &GlobalEnv,
    name: &str,
    ty: &Type,
    with_types: bool,
    mode: WellFormedMode,
    nest: usize,
) -> String {
    let mut conds = vec![];
    let mut add_type_check = |s: String| {
        if with_types {
            conds.push(s);
        }
    };
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::U8 => add_type_check(format!("$IsValidU8({})", name)),
            PrimitiveType::U64 => add_type_check(format!("$IsValidU64({})", name)),
            PrimitiveType::U128 => add_type_check(format!("$IsValidU128({})", name)),
            PrimitiveType::Num => add_type_check(format!("$IsValidNum({})", name)),
            PrimitiveType::Bool => add_type_check(format!("is#$Boolean({})", name)),
            PrimitiveType::Address => add_type_check(format!("is#$Address({})", name)),
            // TODO fix this for a real boogie check
            PrimitiveType::Signer => add_type_check(format!("is#$Address({})", name)),
            PrimitiveType::Range => add_type_check(format!("$IsValidRange({})", name)),
            PrimitiveType::TypeValue => add_type_check(format!("is#$Type({})", name)),
        },
        Type::Vector(elem_ty) => {
            add_type_check(format!("$Vector_$is_well_formed({})", name));
            if !matches!(**elem_ty, Type::TypeParameter(..)) {
                let nest_value = &format!("$select_vector({},$${})", name, nest);
                let elem_expr = boogie_well_formed_expr_impl(
                    env,
                    nest_value,
                    &elem_ty,
                    with_types,
                    mode,
                    nest + 1,
                );
                if !elem_expr.is_empty() {
                    conds.push(format!(
                        "(forall $${}: int :: {{{}}} $${} >= 0 && $${} < $vlen({}) ==> {})",
                        nest, nest_value, nest, nest, name, elem_expr,
                    ));
                }
            }
        }
        Type::Struct(module_idx, struct_idx, _) => {
            let struct_env = env.get_module(*module_idx).into_struct(*struct_idx);
            let well_formed_name = if !with_types {
                "$invariant_holds"
            } else if mode == WellFormedMode::WithoutInvariant {
                "$is_well_typed"
            } else {
                "$is_well_formed"
            };
            conds.push(format!(
                "{}_{}({})",
                boogie_struct_name(&struct_env),
                well_formed_name,
                name
            ))
        }
        Type::Reference(_, rtype) => {
            conds.push(boogie_well_formed_expr_impl(
                env,
                &format!("$Dereference({})", name),
                rtype,
                with_types,
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
        let assert_kind = if type_requires_str.starts_with("free") {
            "assume"
        } else {
            "assert"
        };
        format!("{} {};\n", assert_kind, expr)
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
        let type_check =
            boogie_well_formed_expr(env, &var_selector, ty, WellFormedMode::WithInvariant);
        format!(
            "var {} where (forall {} :: {});",
            declarator,
            (0..param_count)
                .map(|i| format!("$tv{}: $TypeValue", i))
                .join(", "),
            type_check
        )
    } else {
        format!(
            "var {} where {};",
            declarator,
            boogie_well_formed_expr(env, name, ty, WellFormedMode::WithInvariant)
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
            "{} : [{}]$Value",
            name,
            (0..param_count).map(|_| "$TypeValue").join(", ")
        )
    } else {
        format!("{} : $Value", name)
    }
}

pub fn boogie_byte_blob(options: &Options, val: &[u8]) -> String {
    if options.backend.vector_using_sequences {
        // Use concatenation.
        let mut res = "$mk_vector()".to_string();
        for b in val {
            res = format!("$push_back_vector({}, $Integer({}))", res, b);
        }
        res
    } else {
        // Repeated push backs very expensive in map representation, so construct the value
        // array directly.
        let mut ctor_expr = "$MapConstValue($DefaultValue())".to_owned();
        for (i, b) in val.iter().enumerate() {
            ctor_expr = format!("{}[{} := $Integer({})]", ctor_expr, i, *b);
        }
        format!("$Vector($ValueArray({}, {}))", ctor_expr, val.len())
    }
}

/// Construct a statement to debug track a local based on the function table approach. This
/// works without specific Boogie support.
pub fn boogie_debug_track_local_via_function(
    file_idx: &str,
    pos: &str,
    var_idx: &str,
    value: &str,
) -> String {
    ensure_trace_info(format!(
        "assume $DebugTrackLocal({}, {}, {}, {});",
        file_idx, pos, var_idx, value
    ))
}

/// Construct a statement to debug track a local based on the Boogie attribute approach.
pub fn boogie_debug_track_local_via_attrib(
    file_idx: &str,
    pos: &str,
    var_idx: &str,
    value: &str,
) -> String {
    ensure_trace_info(format!(
        "$trace_temp := {};\n\
        assume {{:print \"$track_local({},{},{}):\", $trace_temp}} true;",
        value, file_idx, pos, var_idx,
    ))
}

/// Construct a statement to debug track a local based on the Boogie attribute approach, with
/// dynamically (via variables) provided parameters.
pub fn boogie_debug_track_local_via_attrib_dynamic(
    file_idx: &str,
    pos: &str,
    var_idx: &str,
    value: &str,
) -> String {
    ensure_trace_info(format!(
        "assume {{:print \"$track_local(\",{},\",\",{},\",\",{},\"):\", {}}} true;",
        file_idx, pos, var_idx, value,
    ))
}

/// Construct a statement to debug track an abort. This works without specific Boogie support.
pub fn boogie_debug_track_abort_via_function(
    file_idx: &str,
    pos: &str,
    abort_code: &str,
) -> String {
    ensure_trace_info(format!(
        "assume $DebugTrackAbort({}, {}, {});",
        file_idx, pos, abort_code
    ))
}

/// Construct a statement to debug track an abort using the Boogie attribute approach.
pub fn boogie_debug_track_abort_via_attrib(file_idx: &str, pos: &str, abort_code: &str) -> String {
    ensure_trace_info(format!(
        "$trace_abort_temp := {};\n\
        assume {{:print \"$track_abort({},{}):\", $trace_abort_temp}} true;",
        abort_code, file_idx, pos,
    ))
}

/// Wraps a statement such that Boogie does not elimiate it in execution traces. Boogie
/// does not seem to account for `assume` statements in execution traces, so in order to
/// let our tracking statements above not be forgotten, this trick ensures that they always
/// appear in the trace.
fn ensure_trace_info(s: String) -> String {
    format!("if (true) {{\n {}\n}}", s.replace("\n", "\n  "))
}
