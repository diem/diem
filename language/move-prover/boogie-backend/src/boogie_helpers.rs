// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use crate::options::BoogieOptions;
use bytecode::function_target::FunctionTarget;
use itertools::Itertools;
use move_model::{
    ast::{MemoryLabel, TempIndex},
    model::{
        FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, QualifiedInstId, SpecFunId, StructEnv,
        StructId, SCRIPT_MODULE_NAME,
    },
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};

pub const MAX_MAKE_VEC_ARGS: usize = 4;

/// Return boogie name of given module.
pub fn boogie_module_name(env: &ModuleEnv<'_>) -> String {
    let mod_name = env.get_name();
    let mod_sym = env.symbol_pool().string(mod_name.name());
    if mod_sym.as_str() == SCRIPT_MODULE_NAME {
        // <SELF> is not accepted by boogie as a symbol
        "#SELF#".to_string()
    } else {
        // qualify module by address.
        format!("{}_{}", mod_name.addr().to_str_radix(16), mod_sym)
    }
}

/// Return boogie name of given structure.
pub fn boogie_struct_name(struct_env: &StructEnv<'_>, inst: &[Type]) -> String {
    format!(
        "${}_{}{}",
        boogie_module_name(&struct_env.module_env),
        struct_env.get_name().display(struct_env.symbol_pool()),
        boogie_inst_suffix(struct_env.module_env.env, inst)
    )
}

/// Return field selector for given field.
pub fn boogie_field_sel(field_env: &FieldEnv<'_>, inst: &[Type]) -> String {
    let struct_env = &field_env.struct_env;
    format!(
        "${}#{}",
        field_env.get_name().display(struct_env.symbol_pool()),
        boogie_struct_name(struct_env, inst)
    )
}

/// Return field selector for given field.
pub fn boogie_field_update(field_env: &FieldEnv<'_>, inst: &[Type]) -> String {
    let struct_env = &field_env.struct_env;
    let suffix = boogie_type_suffix_for_struct(struct_env, inst);
    format!(
        "$Update'{}'_{}",
        suffix,
        field_env.get_name().display(struct_env.symbol_pool()),
    )
}

/// Return boogie name of given function.
pub fn boogie_function_name(fun_env: &FunctionEnv<'_>, inst: &[Type]) -> String {
    format!(
        "${}_{}{}",
        boogie_module_name(&fun_env.module_env),
        fun_env.get_name().display(fun_env.symbol_pool()),
        boogie_inst_suffix(fun_env.module_env.env, inst)
    )
}

/// Return boogie name of given spec var.
pub fn boogie_spec_var_name(
    module_env: &ModuleEnv<'_>,
    name: Symbol,
    inst: &[Type],
    memory_label: &Option<MemoryLabel>,
) -> String {
    format!(
        "${}_{}{}{}",
        boogie_module_name(module_env),
        name.display(module_env.symbol_pool()),
        boogie_inst_suffix(module_env.env, inst),
        boogie_memory_label(memory_label)
    )
}

/// Return boogie name of given spec function.
pub fn boogie_spec_fun_name(env: &ModuleEnv<'_>, id: SpecFunId, inst: &[Type]) -> String {
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
        "${}_{}{}{}",
        boogie_module_name(env),
        decl.name.display(env.symbol_pool()),
        overload_qualifier,
        boogie_inst_suffix(env.env, inst)
    )
}

/// Return boogie name for function representing a lifted `some` expression.
pub fn boogie_choice_fun_name(id: usize) -> String {
    format!("$choice_{}", id)
}

/// Creates the name of the resource memory domain for any function for the given struct.
/// This variable represents a local variable of the Boogie translation of this function.
pub fn boogie_modifies_memory_name(env: &GlobalEnv, memory: &QualifiedInstId<StructId>) -> String {
    let struct_env = &env.get_struct_qid(memory.to_qualified_id());
    format!("{}_$modifies", boogie_struct_name(struct_env, &memory.inst))
}

/// Creates the name of the resource memory for the given struct.
pub fn boogie_resource_memory_name(
    env: &GlobalEnv,
    memory: &QualifiedInstId<StructId>,
    memory_label: &Option<MemoryLabel>,
) -> String {
    let struct_env = env.get_struct_qid(memory.to_qualified_id());
    format!(
        "{}_$memory{}",
        boogie_struct_name(&struct_env, &memory.inst),
        boogie_memory_label(memory_label)
    )
}

/// Creates a string for a memory label.
fn boogie_memory_label(memory_label: &Option<MemoryLabel>) -> String {
    if let Some(l) = memory_label {
        format!("#{}", l.as_usize())
    } else {
        "".to_string()
    }
}

/// Creates a vector from the given list of arguments.
pub fn boogie_make_vec_from_strings(args: &[String]) -> String {
    if args.is_empty() {
        "EmptyVec()".to_string()
    } else {
        let mut make = "".to_owned();
        let mut at = 0;
        loop {
            let n = usize::min(args.len() - at, MAX_MAKE_VEC_ARGS);
            let m = format!("MakeVec{}({})", n, args[at..at + n].iter().join(", "));
            make = if make.is_empty() {
                m
            } else {
                format!("ConcatVec({}, {})", make, m)
            };
            at += n;
            if at >= args.len() {
                break;
            }
        }
        make
    }
}

/// Return boogie type for a local with given signature token.
pub fn boogie_type(env: &GlobalEnv, ty: &Type) -> String {
    use PrimitiveType::*;
    use Type::*;
    match ty {
        Primitive(p) => match p {
            U8 | U64 | U128 | Num | Address => "int".to_string(),
            Signer => "$signer".to_string(),
            Bool => "bool".to_string(),
            _ => panic!("unexpected type"),
        },
        Vector(et) => format!("Vec ({})", boogie_type(env, et)),
        Struct(mid, sid, inst) => boogie_struct_name(&env.get_module(*mid).into_struct(*sid), inst),
        Reference(_, bt) => format!("$Mutation ({})", boogie_type(env, bt)),
        TypeParameter(idx) => boogie_type_param(env, *idx),
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | Error | Var(..) => {
            format!("<<unsupported: {:?}>>", ty)
        }
    }
}

pub fn boogie_type_param(_env: &GlobalEnv, idx: u16) -> String {
    format!("#{}", idx)
}

pub fn boogie_temp(env: &GlobalEnv, ty: &Type, instance: usize) -> String {
    boogie_temp_from_suffix(env, &boogie_type_suffix(env, ty), instance)
}

pub fn boogie_temp_from_suffix(_env: &GlobalEnv, suffix: &str, instance: usize) -> String {
    format!("$temp_{}'{}'", instance, suffix)
}

/// Returns the suffix to specialize a name for the given type instance.
pub fn boogie_type_suffix(env: &GlobalEnv, ty: &Type) -> String {
    use PrimitiveType::*;
    use Type::*;
    match ty {
        Primitive(p) => match p {
            U8 => "u8".to_string(),
            U64 => "u64".to_string(),
            U128 => "u128".to_string(),
            Num => "num".to_string(),
            Address => "address".to_string(),
            Signer => "signer".to_string(),
            Bool => "bool".to_string(),
            Range => "range".to_string(),
            _ => format!("<<unsupported {:?}>>", ty),
        },
        Vector(et) => format!("vec{}", boogie_inst_suffix(env, &[et.as_ref().to_owned()])),
        Struct(mid, sid, inst) => {
            boogie_type_suffix_for_struct(&env.get_module(*mid).into_struct(*sid), inst)
        }
        TypeParameter(idx) => boogie_type_param(env, *idx),
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | Error | Var(..)
        | Reference(..) => format!("<<unsupported {:?}>>", ty),
    }
}

pub fn boogie_type_suffix_for_struct(struct_env: &StructEnv<'_>, inst: &[Type]) -> String {
    boogie_struct_name(struct_env, inst)
}

pub fn boogie_inst_suffix(env: &GlobalEnv, inst: &[Type]) -> String {
    if inst.is_empty() {
        "".to_owned()
    } else {
        format!(
            "'{}'",
            inst.iter().map(|ty| boogie_type_suffix(env, ty)).join("_")
        )
    }
}

pub fn boogie_equality_for_type(env: &GlobalEnv, eq: bool, ty: &Type) -> String {
    format!(
        "{}'{}'",
        if eq { "$IsEqual" } else { "!$IsEqual" },
        boogie_type_suffix(env, ty)
    )
}

/// Create boogie well-formed boolean expression.
pub fn boogie_well_formed_expr(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    let target = if ty.is_reference() {
        format!("$Dereference({})", name)
    } else {
        name.to_owned()
    };
    let suffix = boogie_type_suffix(env, ty.skip_reference());
    format!("$IsValid'{}'({})", suffix, target)
}

/// Create boogie well-formed check. The result will be either an empty string or a
/// newline-terminated assume statement.
pub fn boogie_well_formed_check(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    let expr = boogie_well_formed_expr(env, name, ty);
    if !expr.is_empty() {
        format!("assume {};", expr)
    } else {
        "".to_string()
    }
}

/// Create boogie global variable with type constraint. No references allowed.
pub fn boogie_declare_global(env: &GlobalEnv, name: &str, ty: &Type) -> String {
    assert!(!ty.is_reference());
    format!(
        "var {} : {} where {};",
        name,
        boogie_type(env, ty),
        // TODO: boogie crash boogie_well_formed_expr(env, name, ty)
        // boogie_well_formed_expr(env, name, ty)"
        "true"
    )
}

pub fn boogie_byte_blob(_options: &BoogieOptions, val: &[u8]) -> String {
    let args = val.iter().map(|v| format!("{}", *v)).collect_vec();
    if args.is_empty() {
        "$EmptyVec'u8'()".to_string()
    } else {
        boogie_make_vec_from_strings(&args)
    }
}

/// Construct a statement to debug track a local based on the Boogie attribute approach.
pub fn boogie_debug_track_local(
    fun_target: &FunctionTarget<'_>,
    origin_idx: TempIndex,
    idx: TempIndex,
    ty: &Type,
) -> String {
    boogie_debug_track(fun_target, "$track_local", origin_idx, idx, ty)
}

fn boogie_debug_track(
    fun_target: &FunctionTarget<'_>,
    track_tag: &str,
    tracked_idx: usize,
    idx: TempIndex,
    ty: &Type,
) -> String {
    let fun_def_idx = fun_target.func_env.get_def_idx();
    let value = format!("$t{}", idx);
    if ty.is_reference() {
        let temp_name = boogie_temp(fun_target.global_env(), ty.skip_reference(), 0);
        format!(
            "{} := $Dereference({});\n\
             assume {{:print \"{}({},{},{}):\", {}}} {} == {};",
            temp_name,
            value,
            track_tag,
            fun_target.func_env.module_env.get_id().to_usize(),
            fun_def_idx,
            tracked_idx,
            temp_name,
            temp_name,
            temp_name
        )
    } else {
        format!(
            "assume {{:print \"{}({},{},{}):\", {}}} {} == {};",
            track_tag,
            fun_target.func_env.module_env.get_id().to_usize(),
            fun_def_idx,
            tracked_idx,
            value,
            value,
            value
        )
    }
}

/// Construct a statement to debug track an abort.
pub fn boogie_debug_track_abort(fun_target: &FunctionTarget<'_>, abort_code: &str) -> String {
    let fun_def_idx = fun_target.func_env.get_def_idx();
    format!(
        "assume {{:print \"$track_abort({},{}):\", {}}} {} == {};",
        fun_target.func_env.module_env.get_id().to_usize(),
        fun_def_idx,
        abort_code,
        abort_code,
        abort_code,
    )
}

/// Construct a statement to debug track a return value.
pub fn boogie_debug_track_return(
    fun_target: &FunctionTarget<'_>,
    ret_idx: usize,
    idx: TempIndex,
    ty: &Type,
) -> String {
    boogie_debug_track(fun_target, "$track_return", ret_idx, idx, ty)
}
