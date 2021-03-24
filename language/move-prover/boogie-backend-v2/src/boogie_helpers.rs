// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helpers for emitting Boogie code.

use crate::options::BoogieOptions;
use bytecode::function_target::FunctionTarget;
use itertools::Itertools;
use move_model::{
    ast::{MemoryLabel, TempIndex},
    model::{
        FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, ModuleId, QualifiedId, SpecFunId, StructEnv,
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
pub fn boogie_spec_var_name(
    env: &ModuleEnv<'_>,
    name: Symbol,
    memory_label: &Option<MemoryLabel>,
) -> String {
    format!(
        "${}_{}{}",
        boogie_module_name(env),
        name.display(env.symbol_pool()),
        boogie_memory_label(memory_label)
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
            PrimitiveType::EventStore => unimplemented!("EventStore"),
        },
        Type::Vector(t) => format!("$Vector_type_value({})", boogie_type_value(env, t)),
        Type::Reference(_, t) => format!("ReferenceType({})", boogie_type_value(env, t)),
        Type::TypeParameter(index) => format!("$tv{}", index),
        Type::TypeLocal(s) => s.display(env.symbol_pool()).to_string(),
        Type::Struct(module_id, struct_id, args) => {
            boogie_struct_type_value(env, *module_id, *struct_id, args)
        }
        // TODO: function and tuple types?
        Type::Tuple(_args) => "Tuple_type_value()".to_string(),
        Type::Fun(_args, _result) => "Function_type_value()".to_string(),
        Type::Error => panic!("unexpected error type"),
        Type::Var(..) => panic!("unexpected type variable"),
        Type::TypeDomain(..) | Type::ResourceDomain(..) => panic!("unexpected transient type"),
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

/// Creates the name of the resource memory domain for any function for the given struct.
/// This variable represents a local variable of the Boogie translation of this function.
pub fn boogie_modifies_memory_name(env: &GlobalEnv, memory: QualifiedId<StructId>) -> String {
    let struct_env = env.get_module(memory.module_id).into_struct(memory.id);
    format!("{}_$Modifies", boogie_struct_name(&struct_env))
}

/// Creates the name of the resource memory for the given struct.
pub fn boogie_resource_memory_name(
    env: &GlobalEnv,
    memory: QualifiedId<StructId>,
    memory_label: &Option<MemoryLabel>,
) -> String {
    let struct_env = env.get_module(memory.module_id).into_struct(memory.id);
    format!(
        "{}_$memory{}",
        boogie_struct_name(&struct_env),
        boogie_memory_label(memory_label)
    )
}

fn boogie_memory_label(memory_label: &Option<MemoryLabel>) -> String {
    if let Some(l) = memory_label {
        format!("#{}", l.as_usize())
    } else {
        "".to_string()
    }
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
    boogie_make_vec_from_strings(args)
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
pub fn boogie_local_type(ty: &Type) -> String {
    use PrimitiveType::*;
    use Type::*;
    match ty {
        Primitive(p) => match p {
            U8 | U64 | U128 | Num | Address | Signer => "int".to_string(),
            Bool => "bool".to_string(),
            TypeValue => "$TypeValue".to_string(),
            _ => panic!("unexpected type"),
        },
        Vector(..) | Struct(..) => "Vec $Value".to_string(),
        Reference(..) => "$Mutation".to_string(),
        TypeParameter(..) => "$Value".to_string(),
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | TypeLocal(..) | Error
        | Var(..) => format!("<<unsupported: {:?}>>", ty),
    }
}

pub fn boogie_declare_temps() -> String {
    "var $$t_bool: bool;\nvar $$t_int: int;\nvar $$t_addr: int;\nvar $$t_type: $TypeValue;\
    \nvar $$t, $$t1, $$t2, $$t3: $Value;\nvar $$t_vec: Vec $Value;"
        .to_string()
}

pub fn boogie_temp(ty: &Type) -> String {
    format!("$$t{}", boogie_type_suffix(ty))
}

pub fn boogie_type_suffix(ty: &Type) -> &'static str {
    use PrimitiveType::*;
    use Type::*;
    match ty {
        Primitive(p) => match p {
            U8 | U64 | U128 | Num | Signer => "_int",
            Address => "_addr",
            Bool => "_bool",
            TypeValue => "_type",
            EventStore => "",
            _ => "<<unsupported>>",
        },
        Vector(..) | Struct(..) => "_vec",
        TypeParameter(..) => "",
        Fun(..) | Tuple(..) | TypeDomain(..) | ResourceDomain(..) | TypeLocal(..) | Error
        | Var(..) | Reference(..) => "<<unsupported>>",
    }
}

pub fn boogie_equality_for_type(eq: bool, ty: &Type) -> String {
    format!(
        "{}{}",
        if eq { "$IsEqual" } else { "!$IsEqual" },
        boogie_type_suffix(ty)
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
            let elem_ty_value = boogie_type_value(env, elem_ty);
            add_type_check(format!(
                "$Vector_$is_well_formed({}, {})",
                elem_ty_value, name
            ));
            if !matches!(**elem_ty, Type::TypeParameter(..)) {
                let suffix = boogie_type_suffix(elem_ty);
                let nest_value = &format!("ReadVec({},$${})", name, nest);
                let mut check = format!("$IsValidBox{}({})", suffix, nest_value);
                let nest_value_unboxed = &format!("$Unbox{}({})", suffix, nest_value);
                let nest_check = boogie_well_formed_expr_impl(
                    env,
                    nest_value_unboxed,
                    &elem_ty,
                    usize::saturating_add(nest, 1),
                );
                if !nest_check.is_empty() {
                    check = format!("{} &&\n  {}", check, nest_check);
                }
                add_type_check(format!(
                    "(forall $${}: int :: {{{}}}\n{}    $InRangeVec({}, $${}) ==> {})",
                    nest, nest_value, indent, name, nest, check,
                ));
            }
        }
        Type::Struct(module_idx, struct_idx, targs) => {
            let struct_env = env.get_module(*module_idx).into_struct(*struct_idx);
            add_type_check(format!(
                "$Tag{}({}) && $IsValidU64(LenVec({}))",
                boogie_struct_name(&struct_env),
                name,
                name
            ));
            add_type_check(format!(
                "LenVec({}) == {}",
                name,
                struct_env.get_field_count()
            ));
            let field_var = format!("$$f{}", nest);
            for field_env in struct_env.get_fields() {
                let ty = field_env.get_type().instantiate(targs);
                let suffix = boogie_type_suffix(&ty);
                let field_value = format!("ReadVec({}, {})", name, boogie_field_name(&field_env));
                add_type_check(format!("$IsValidBox{}({})", suffix, field_value));
                let nest_check = boogie_well_formed_expr_impl(
                    env,
                    &field_var,
                    &ty,
                    usize::saturating_add(nest, 1),
                );
                if !nest_check.is_empty() {
                    add_type_check(format!(
                        "(var {} := $Unbox{}({});\n{}   {})",
                        field_var, suffix, field_value, indent, nest_check
                    ));
                }
            }
        }
        Type::Reference(true, rtype) => {
            let suffix = boogie_type_suffix(rtype);
            add_type_check(format!("$IsValidBox{}($Dereference({}))", suffix, name));
            add_type_check(boogie_well_formed_expr_impl(
                env,
                &format!("$Unbox{}($Dereference({}))", suffix, name),
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
pub fn boogie_declare_global(env: &GlobalEnv, name: &str, param_count: usize, ty: &Type) -> String {
    let declarator = boogie_global_declarator(env, name, param_count, ty);
    assert!(!ty.is_reference());
    if param_count > 0 {
        let var_selector = format!(
            "{}[{}]",
            name,
            (0..param_count).map(|i| format!("$tv{}", i)).join(", ")
        );
        let type_check = boogie_well_formed_expr(env, &var_selector, ty);
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
            boogie_well_formed_expr(env, name, ty,)
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
            "{} : [{}]{}",
            name,
            (0..param_count).map(|_| "$TypeValue").join(", "),
            boogie_local_type(ty)
        )
    } else {
        format!("{} : {}", name, boogie_local_type(ty))
    }
}

pub fn boogie_byte_blob(_options: &BoogieOptions, val: &[u8]) -> String {
    let args = val
        .iter()
        .map(|v| format!("$Box_int({})", *v))
        .collect_vec();
    boogie_make_vec_from_strings(&args)
}

/// Construct a statement to debug track a local based on the Boogie attribute approach.
pub fn boogie_debug_track_local(
    fun_target: &FunctionTarget<'_>,
    origin_idx: TempIndex,
    idx: TempIndex,
) -> String {
    boogie_debug_track(fun_target, "$track_local", origin_idx, idx)
}

fn boogie_debug_track(
    fun_target: &FunctionTarget<'_>,
    track_tag: &str,
    tracked_idx: usize,
    idx: TempIndex,
) -> String {
    let fun_def_idx = fun_target.func_env.get_def_idx();
    let ty = fun_target.get_local_type(idx);
    let value = format!("$t{}", idx);
    if ty.is_reference() {
        let temp_name = boogie_temp(ty.skip_reference());
        let suffix = boogie_type_suffix(ty.skip_reference());
        format!(
            "{} := $Unbox{}($Dereference({}));\n\
             assume {{:print \"{}({},{},{}):\", {}}} {} == {};",
            temp_name,
            suffix,
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
) -> String {
    boogie_debug_track(fun_target, "$track_return", ret_idx, idx)
}
