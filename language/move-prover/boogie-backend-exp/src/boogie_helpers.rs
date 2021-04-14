// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use boogie_backend::options::BoogieOptions;
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

pub const MAX_MAKE_VEC_ARGS: usize = 12;

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
        "{}#{}",
        field_env.get_name().display(struct_env.symbol_pool()),
        boogie_struct_name(struct_env, inst)
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
        let n = usize::min(args.len(), MAX_MAKE_VEC_ARGS);
        let direct_args = &args[0..n];
        let mut make = format!("MakeVec{}({})", n, direct_args.iter().join(", "));
        for arg in args.iter().skip(n) {
            make = format!("ExtendVec({}, {})", make, arg)
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
            U8 | U64 | U128 | Num | Address | Signer => "int".to_string(),
            Bool => "bool".to_string(),
            TypeValue => "$TypeValue".to_string(),
            _ => panic!("unexpected type"),
        },
        Vector(et) => format!("Vec ({})", boogie_type(env, et)),
        Struct(mid, sid, inst) => boogie_struct_name(&env.get_module(*mid).into_struct(*sid), inst),
        Reference(_, bt) => format!("$Mutation ({})", boogie_type(env, bt)),
        TypeParameter(idx) => boogie_type_param(env, *idx),
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | TypeLocal(..) | Error
        | Var(..) => format!("<<unsupported: {:?}>>", ty),
    }
}

pub fn boogie_type_param(_env: &GlobalEnv, idx: u16) -> String {
    format!("#{}", idx)
}

pub fn boogie_temp(env: &GlobalEnv, ty: &Type, instance: usize) -> String {
    boogie_temp_from_suffix(env, &boogie_type_suffix(env, ty), instance)
}

pub fn boogie_temp_from_suffix(_env: &GlobalEnv, suffix: &str, instance: usize) -> String {
    format!("$temp_{}{}", instance, suffix)
}

pub fn boogie_type_suffix(env: &GlobalEnv, ty: &Type) -> String {
    use PrimitiveType::*;
    use Type::*;
    match ty {
        Primitive(p) => match p {
            U8 | U64 | U128 | Num => "int".to_string(),
            Address | Signer => "addr".to_string(),
            Bool => "bool".to_string(),
            EventStore => "".to_string(),
            _ => format!("<<unsupported {:?}>>", ty),
        },
        Vector(et) => format!("vec{}", boogie_inst_suffix(env, &[et.as_ref().to_owned()])),
        Struct(mid, sid, inst) => {
            boogie_type_suffix_for_struct(&env.get_module(*mid).into_struct(*sid), inst)
        }
        TypeParameter(idx) => boogie_type_param(env, *idx),
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | TypeLocal(..) | Error
        | Var(..) | Reference(..) => format!("<<unsupported {:?}>>", ty),
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
    boogie_well_formed_expr_impl(env, name, ty, 0)
}

fn boogie_well_formed_expr_impl(env: &GlobalEnv, name: &str, ty: &Type, nest: usize) -> String {
    let mut conds = vec![];
    let indent = " ".repeat(usize::saturating_mul(nest, 2));
    let mut add_type_check = |s: String| {
        conds.push(format!("{}{}", indent, s));
    };
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::U8 => add_type_check(format!("$IsValidU8({})", name)),
            PrimitiveType::U64 => add_type_check(format!("$IsValidU64({})", name)),
            PrimitiveType::U128 => add_type_check(format!("$IsValidU128({})", name)),
            PrimitiveType::Num => add_type_check(format!("$IsValidNum({})", name)),
            PrimitiveType::Bool => add_type_check(format!("$IsValidBool({})", name)),
            PrimitiveType::Address | PrimitiveType::Signer => {
                add_type_check(format!("$IsValidAddress({})", name))
            }
            PrimitiveType::Range => add_type_check(format!("$IsValidRange({})", name)),
            PrimitiveType::TypeValue => {}
            PrimitiveType::EventStore => unimplemented!("EventStore"),
        },
        Type::Vector(elem_ty) => {
            add_type_check(format!("$Vector_$is_well_formed({})", name));
            if !matches!(**elem_ty, Type::TypeParameter(..)) {
                let nest_value = &format!("ReadVec({},$${})", name, nest);
                let nest_check = boogie_well_formed_expr_impl(
                    env,
                    nest_value,
                    &elem_ty,
                    usize::saturating_add(nest, 1),
                );
                if !nest_check.is_empty() {
                    add_type_check(format!(
                        "(forall $${}: int :: {{{}}}\n{}   InRangeVec({}, $${}) ==> {})",
                        nest, nest_value, indent, name, nest, nest_check,
                    ));
                }
            }
        }
        Type::Struct(module_idx, struct_idx, targs) => {
            let struct_env = env.get_module(*module_idx).into_struct(*struct_idx);
            if !struct_env.is_native_or_intrinsic() {
                for field in struct_env.get_fields() {
                    let sel = format!("{}({})", boogie_field_sel(&field, targs), name);
                    let ty = field.get_type();
                    let inst_type = ty.instantiate(targs);
                    add_type_check(boogie_well_formed_expr_impl(
                        env,
                        &sel,
                        &inst_type,
                        nest.saturating_add(1),
                    ));
                }
            }
        }
        Type::Reference(true, rtype) => {
            add_type_check(boogie_well_formed_expr_impl(
                env,
                &format!("$Dereference({})", name),
                rtype,
                usize::saturating_add(nest, 1),
            ));
        }
        Type::Reference(false, rtype) => {
            add_type_check(boogie_well_formed_expr_impl(
                env,
                name,
                rtype.skip_reference(),
                nest,
            ));
        }
        // TODO: tuple and functions?
        Type::Fun(_args, _result) => {}
        Type::Tuple(_elems) => {}
        // A type parameter or type value is opaque, so no type check here.
        Type::TypeParameter(..) | Type::TypeLocal(..) => {}
        Type::Error | Type::Var(..) | Type::TypeDomain(..) | Type::ResourceDomain(..) => {
            panic!("unexpected transient type")
        }
    }
    conds.iter().filter(|s| !s.is_empty()).join("\n&& ")
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

pub fn boogie_global_declarator(
    env: &GlobalEnv,
    name: &str,
    param_count: usize,
    ty: &Type,
) -> String {
    assert!(!ty.is_reference());
    if param_count > 0 {
        format!(
            "{} : [{}]{}",
            name,
            (0..param_count).map(|_| "$TypeValue").join(", "),
            boogie_type(env, ty)
        )
    } else {
        format!("{} : {}", name, boogie_type(env, ty))
    }
}

pub fn boogie_byte_blob(_options: &BoogieOptions, val: &[u8]) -> String {
    let args = val.iter().map(|v| format!("{}", *v)).collect_vec();
    if args.is_empty() {
        "$EmptyVec'int'()".to_string()
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
