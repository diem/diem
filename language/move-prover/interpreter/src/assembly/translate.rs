// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use bytecode::{
    function_target::FunctionTarget,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{
        AssignKind, BorrowEdge, BorrowNode, Bytecode, HavocKind, Operation, PropKind, StrongEdge,
    },
};
use move_model::{
    ast::{MemoryLabel, TempIndex},
    model::{GlobalEnv, ModuleId, StructId},
    ty as MT,
};

use crate::{
    assembly::{
        ast::{Function, GlobalSlot, Instruction, LocalSlot, Program},
        status,
        ty::{BaseType, StructField, StructInstantiation, Type},
    },
    shared::ident::{FunctionIdent, StructIdent},
};

//**************************************************************************************************
// Program
//**************************************************************************************************

pub fn translate(env: &GlobalEnv, holder: &FunctionTargetsHolder) -> Option<Program> {
    let mut functions = BTreeMap::new();
    let mut global_slots = BTreeMap::new();
    let toposort = FunctionTargetPipeline::sort_targets_in_topological_order(env, holder);
    for func_env in toposort {
        for (_, target) in holder.get_targets(&func_env) {
            let func_ident = FunctionIdent::new(&target);
            if !target.data.code.is_empty() {
                let func_body = match function(holder, &target, &mut global_slots) {
                    None => {
                        assert!(env.has_errors());
                        return None;
                    }
                    Some(f) => f,
                };
                if functions.contains_key(&func_ident) {
                    status::error_duplicated_function(&target, &func_ident);
                    return None;
                }
                functions.insert(func_ident, func_body);
            }
        }
    }
    let program = Program {
        functions,
        global_slots,
    };
    Some(program)
}

//**************************************************************************************************
// Function
//**************************************************************************************************

fn function(
    holder: &FunctionTargetsHolder,
    target: &FunctionTarget,
    global_slots: &mut BTreeMap<MemoryLabel, GlobalSlot>,
) -> Option<Function> {
    let env = target.global_env();

    // discover and validate local slots
    let param_decls = target.func_env.get_parameters();
    assert!(param_decls.len() <= target.get_local_count());

    let mut local_slots = vec![];
    for (i, local_ty) in target.data.local_types.iter().enumerate() {
        let is_arg = i < target.get_parameter_count();
        let name = env
            .symbol_pool()
            .string(target.get_local_name(i))
            .to_string();
        if is_arg {
            // check that types for local slots is compatible with the declared parameter type
            let param_decl_ty = &param_decls.get(i).unwrap().1;
            let type_matches = if local_ty == param_decl_ty {
                true
            } else {
                match param_decl_ty {
                    MT::Type::Reference(false, base_ty) => local_ty == base_ty.as_ref(),
                    _ => false,
                }
            };
            if !type_matches {
                status::error_local_model_type_mismatches_declared_parameter_type(
                    target,
                    param_decl_ty,
                    local_ty,
                    i,
                    &name,
                );
                return None;
            }
        }
        match local_type(env, local_ty, None) {
            None => {
                status::error_local_slot_type_is_invalid(target, local_ty, i, &name);
                return None;
            }
            Some(ty) => {
                let slot = LocalSlot { ty, is_arg, name };
                local_slots.push(slot);
            }
        };
    }

    // iterate over bytecode
    let mut instructions = vec![];
    for (i, bytecode) in target.get_bytecode().iter().enumerate() {
        match instruction(holder, target, &local_slots, bytecode, global_slots) {
            None => {
                status::error_bytecode_is_invalid(target, i, bytecode);
                return None;
            }
            Some(inst) => {
                instructions.push(inst);
            }
        };
    }

    // translate function body
    let func_body = Function {
        local_slots,
        instructions,
    };
    Some(func_body)
}

//**************************************************************************************************
// Typing
//**************************************************************************************************

fn local_type(env: &GlobalEnv, ty: &MT::Type, subst: Option<&[BaseType]>) -> Option<Type> {
    let new_ty = match ty {
        MT::Type::Primitive(..)
        | MT::Type::Vector(..)
        | MT::Type::Struct(..)
        | MT::Type::TypeParameter(..) => Type::Base(base_type(env, ty, subst)?),
        MT::Type::Reference(is_mut, base_ty) => {
            base_type(env, base_ty, subst)?.into_ref_type(*is_mut)
        }
        _ => {
            return None;
        }
    };
    Some(new_ty)
}

fn base_type(env: &GlobalEnv, ty: &MT::Type, subst: Option<&[BaseType]>) -> Option<BaseType> {
    let new_ty = match ty {
        MT::Type::Primitive(MT::PrimitiveType::Bool) => BaseType::mk_bool(),
        MT::Type::Primitive(MT::PrimitiveType::U8) => BaseType::mk_u8(),
        MT::Type::Primitive(MT::PrimitiveType::U64) => BaseType::mk_u64(),
        MT::Type::Primitive(MT::PrimitiveType::U128) => BaseType::mk_u128(),
        MT::Type::Primitive(MT::PrimitiveType::Num) => BaseType::mk_num(),
        MT::Type::Primitive(MT::PrimitiveType::Address) => BaseType::mk_address(),
        MT::Type::Primitive(MT::PrimitiveType::Signer) => BaseType::mk_signer(),
        MT::Type::Vector(elem_ty) => BaseType::mk_vector(base_type(env, elem_ty, subst)?),
        MT::Type::Struct(module_id, struct_id, ty_insts) => {
            BaseType::mk_struct(struct_type(env, *module_id, *struct_id, ty_insts, subst)?)
        }
        MT::Type::TypeParameter(index) => match subst {
            None => BaseType::mk_parameter(*index),
            Some(ty_args) => match ty_args.get(*index as usize) {
                None => {
                    return None;
                }
                Some(ty_arg) => ty_arg.clone(),
            },
        },
        MT::Type::TypeLocal(symbol) => {
            BaseType::mk_binding(env.symbol_pool().string(*symbol).to_string())
        }
        _ => {
            return None;
        }
    };
    Some(new_ty)
}

fn struct_type(
    env: &GlobalEnv,
    module_id: ModuleId,
    struct_id: StructId,
    ty_insts: &[MT::Type],
    subst: Option<&[BaseType]>,
) -> Option<StructInstantiation> {
    // derive struct identity
    let struct_env = env.get_struct(module_id.qualified(struct_id));
    let ident = StructIdent::new(&struct_env);

    // convert instantiations
    let struct_ty_params = struct_env.get_type_parameters();

    // check type arguments
    // TODO (mengxu) verify type constraints
    assert_eq!(struct_ty_params.len(), ty_insts.len());
    let new_ty_insts = ty_insts
        .iter()
        .map(|ty_inst| base_type(env, ty_inst, subst))
        .collect::<Option<Vec<_>>>()?;

    // collect fields
    let fields = struct_env
        .get_fields()
        .map(|field_env| {
            let field_name = env.symbol_pool().string(field_env.get_name()).to_string();
            let field_ty_opt = base_type(env, &field_env.get_type(), Some(&new_ty_insts));
            field_ty_opt.map(|field_ty| StructField {
                name: field_name,
                ty: field_ty,
            })
        })
        .collect::<Option<Vec<_>>>()?;

    // return the information for constructing the struct type
    Some(StructInstantiation { ident, fields })
}

//**************************************************************************************************
// Instruction
//**************************************************************************************************

fn instruction(
    holder: &FunctionTargetsHolder,
    target: &FunctionTarget,
    local_slots: &[LocalSlot],
    bytecode: &Bytecode,
    global_slots: &mut BTreeMap<MemoryLabel, GlobalSlot>,
) -> Option<Instruction> {
    let env = target.global_env();

    // utility function pointers
    let util_is_index_oob = |index: TempIndex| {
        let invalid = index >= local_slots.len();
        if invalid {
            status::error_bytecode_local_slot_access_out_of_bound(target, bytecode, index);
        }
        invalid
    };
    let util_err_operand_type = |slots: Vec<&LocalSlot>| {
        status::error_bytecode_operand_type_incompatible(
            target,
            bytecode,
            slots.into_iter().map(|slot| &slot.ty),
        );
    };

    // match against each case of the bytecode
    let instruction = match bytecode {
        Bytecode::Assign(_, dst, src, kind) => {
            let dst = *dst;
            let src = *src;

            // check bounds
            if util_is_index_oob(dst) {
                return None;
            }
            if util_is_index_oob(src) {
                return None;
            }

            // check types
            let dst_slot = local_slots.get(dst).unwrap();
            let src_slot = local_slots.get(src).unwrap();
            if !dst_slot.ty.is_compatible_for_assign(&src_slot.ty) {
                util_err_operand_type(vec![dst_slot, src_slot]);
                return None;
            }

            // build the instruction
            match kind {
                AssignKind::Move | AssignKind::Store => Instruction::Move { dst, src },
                AssignKind::Copy => Instruction::Copy { dst, src },
            }
        }
        Bytecode::Load(_, dst, val) => {
            let dst = *dst;

            // check bounds
            if util_is_index_oob(dst) {
                return None;
            }

            // check types
            let dst_slot = local_slots.get(dst).unwrap();
            if !dst_slot.ty.is_compatible_for_constant(val) {
                util_err_operand_type(vec![dst_slot]);
                return None;
            }

            // build the instruction
            Instruction::Load {
                dst,
                val: val.clone(),
            }
        }

        Bytecode::Call(_, dsts, op, srcs, on_abort) => {
            // check bounds
            for index in dsts.iter().chain(srcs.iter()).copied() {
                if util_is_index_oob(index) {
                    return None;
                }
            }

            // check abort action validity
            match on_abort {
                None => (),
                Some(action) => {
                    assert!(op.can_abort());
                    let index = action.1;
                    if util_is_index_oob(index) {
                        return None;
                    }
                    let index_slot = local_slots.get(index).unwrap();
                    if !index_slot.ty.is_compatible_for_abort_code() {
                        util_err_operand_type(vec![index_slot]);
                        return None;
                    }
                }
            };

            // collect src and dst slots
            let dst_slots: Vec<_> = dsts
                .iter()
                .map(|index| local_slots.get(*index).unwrap())
                .collect();
            let src_slots: Vec<_> = srcs
                .iter()
                .map(|index| local_slots.get(*index).unwrap())
                .collect();

            // utility function pointers
            let util_operand_counts_mismatch =
                |expected_src_count: usize, expected_dst_count: usize| {
                    let invalid = src_slots.len() != expected_src_count
                        || dst_slots.len() != expected_dst_count;
                    if invalid {
                        status::error_bytecode_operand_count_mismatch(
                            target,
                            bytecode,
                            src_slots.len(),
                            dst_slots.len(),
                        );
                    }
                    invalid
                };
            let util_err_call_operand_type_ext = |slots: Vec<&LocalSlot>| {
                status::error_bytecode_operand_type_incompatible(
                    target,
                    bytecode,
                    dst_slots
                        .iter()
                        .chain(src_slots.iter())
                        .chain(slots.iter())
                        .map(|slot| &slot.ty),
                );
            };
            let util_err_call_operand_type = || util_err_call_operand_type_ext(vec![]);

            // branch based on the operation
            match op {
                // user function
                Operation::Function(callee_module, callee_func, callee_ty_args) => {
                    let callee_env = env.get_function(callee_module.qualified(*callee_func));
                    // TODO (mengxu): might need to call a different function variant?
                    let callee_target = holder.get_target(&callee_env, &FunctionVariant::Baseline);

                    // check type arguments
                    let callee_ty_params = callee_target.get_type_parameters();
                    // TODO (mengxu) verify type constraints
                    assert_eq!(callee_ty_params.len(), callee_ty_args.len());
                    let callee_ty_insts = callee_ty_args
                        .iter()
                        .map(|callee_ty_arg| base_type(env, callee_ty_arg, None))
                        .collect::<Option<Vec<_>>>()?;

                    // check operand bounds
                    if util_operand_counts_mismatch(
                        callee_target.get_parameter_count(),
                        callee_target.get_return_count(),
                    ) {
                        return None;
                    }

                    // check operand types
                    for (i, slot) in src_slots.iter().enumerate() {
                        // NOTE: unwrap is safe because the functions are processed in topological order
                        let callee_local_type = local_type(
                            env,
                            callee_target.get_local_type(i),
                            Some(&callee_ty_insts),
                        )
                        .unwrap();
                        if slot.ty != callee_local_type {
                            util_err_call_operand_type();
                            return None;
                        }
                    }
                    for (i, slot) in dst_slots.iter().enumerate() {
                        // NOTE: unwrap is safe because the functions are processed in topological order
                        let callee_local_type = local_type(
                            env,
                            callee_target.get_return_type(i),
                            Some(&callee_ty_insts),
                        )
                        .unwrap();
                        if slot.ty != callee_local_type {
                            util_err_call_operand_type();
                            return None;
                        }
                    }

                    // build the instruction
                    Instruction::Call {
                        ident: FunctionIdent::new(&callee_target),
                        ty_args: callee_ty_insts,
                        args: srcs.clone(),
                        rets: dsts.clone(),
                        on_abort: on_abort.clone(),
                    }
                }

                // opaque
                Operation::OpaqueCallBegin(callee_module, callee_func, callee_ty_args) => {
                    let callee_env = env.get_function(callee_module.qualified(*callee_func));
                    // TODO (mengxu): might need to call a different function variant?
                    let callee_target = holder.get_target(&callee_env, &FunctionVariant::Baseline);

                    // check type arguments
                    let callee_ty_params = callee_target.get_type_parameters();
                    // TODO (mengxu) verify type constraints
                    assert_eq!(callee_ty_params.len(), callee_ty_args.len());
                    let callee_ty_insts = callee_ty_args
                        .iter()
                        .map(|callee_ty_arg| base_type(env, callee_ty_arg, None))
                        .collect::<Option<Vec<_>>>()?;

                    // check operand bounds
                    if util_operand_counts_mismatch(0, 0) {
                        return None;
                    }

                    // build the instruction
                    Instruction::OpaqueCallInit {
                        ident: FunctionIdent::new(&callee_target),
                        ty_args: callee_ty_insts,
                    }
                }
                Operation::OpaqueCallEnd(callee_module, callee_func, callee_ty_args) => {
                    let callee_env = env.get_function(callee_module.qualified(*callee_func));
                    // TODO (mengxu): might need to call a different function variant?
                    let callee_target = holder.get_target(&callee_env, &FunctionVariant::Baseline);

                    // check type arguments
                    let callee_ty_params = callee_target.get_type_parameters();
                    // TODO (mengxu) verify type constraints
                    assert_eq!(callee_ty_params.len(), callee_ty_args.len());
                    let callee_ty_insts = callee_ty_args
                        .iter()
                        .map(|callee_ty_arg| base_type(env, callee_ty_arg, None))
                        .collect::<Option<Vec<_>>>()?;

                    // check operand bounds
                    if util_operand_counts_mismatch(0, 0) {
                        return None;
                    }

                    // build the instruction
                    Instruction::OpaqueCallFini {
                        ident: FunctionIdent::new(&callee_target),
                        ty_args: callee_ty_insts,
                    }
                }

                // struct
                Operation::Pack(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(struct_inst.fields.len(), 1) {
                        return None;
                    }

                    // check operand types
                    for (slot, field) in src_slots.iter().zip(struct_inst.fields.iter()) {
                        if !slot.ty.is_base_of(&field.ty) {
                            util_err_call_operand_type();
                            return None;
                        }
                    }
                    if !dst_slots.get(0).unwrap().ty.is_struct_of(&struct_inst) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Pack {
                        inst: struct_inst,
                        fields: srcs.clone(),
                        packed: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Unpack(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, struct_inst.fields.len()) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_struct_of(&struct_inst) {
                        util_err_call_operand_type();
                        return None;
                    }
                    for (slot, field) in dst_slots.iter().zip(struct_inst.fields.iter()) {
                        if !slot.ty.is_base_of(&field.ty) {
                            util_err_call_operand_type();
                            return None;
                        }
                    }

                    // build the instruction
                    Instruction::Unpack {
                        inst: struct_inst,
                        fields: dsts.clone(),
                        packed: *srcs.get(0).unwrap(),
                    }
                }
                Operation::GetField(module_id, struct_id, struct_ty_args, field_num) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;
                    let field_num = *field_num;
                    assert!(field_num < struct_inst.fields.len());
                    let field = struct_inst.fields.get(field_num).unwrap();

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    let src_slot = src_slots.get(0).unwrap();
                    if !src_slot.ty.is_struct_of(&struct_inst)
                        && !src_slot.ty.is_ref_struct_of(&struct_inst, None)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_base_of(&field.ty) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::GetField {
                        inst: struct_inst,
                        field: *dsts.get(0).unwrap(),
                        packed: *srcs.get(0).unwrap(),
                    }
                }
                Operation::BorrowField(module_id, struct_id, struct_ty_args, field_num) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;
                    let field_num = *field_num;
                    assert!(field_num < struct_inst.fields.len());
                    let field = struct_inst.fields.get(field_num).unwrap();

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    let src_slot = src_slots.get(0).unwrap();
                    if !src_slot.ty.is_ref_struct_of(&struct_inst, None) {
                        util_err_call_operand_type();
                        return None;
                    }
                    let src_ref_is_mut = src_slot.ty.get_ref_type().unwrap().0;
                    if !dst_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_ref_of(&field.ty, if src_ref_is_mut { None } else { Some(false) })
                    {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BorrowField {
                        inst: struct_inst,
                        field: *dsts.get(0).unwrap(),
                        packed: *srcs.get(0).unwrap(),
                    }
                }
                Operation::MoveTo(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(2, 0) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_struct_of(&struct_inst) {
                        util_err_call_operand_type();
                        return None;
                    }
                    let signer_slot = src_slots.get(1).unwrap();
                    if !signer_slot.ty.is_signer() && !signer_slot.ty.is_ref_of_signer(Some(false))
                    {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::MoveTo {
                        inst: struct_inst,
                        addr: *srcs.get(1).unwrap(),
                        src: *srcs.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::MoveFrom(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_address() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_struct_of(&struct_inst) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::MoveFrom {
                        inst: struct_inst,
                        addr: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::GetGlobal(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_address() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_struct_of(&struct_inst) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::GetGlobal {
                        inst: struct_inst,
                        addr: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::BorrowGlobal(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_address() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_ref_struct_of(&struct_inst, None)
                    {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BorrowGlobal {
                        inst: struct_inst,
                        addr: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::Exists(module_id, struct_id, struct_ty_args) => {
                    let struct_inst =
                        struct_type(env, *module_id, *struct_id, struct_ty_args, None)?;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_address() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::ExistsGlobal {
                        inst: struct_inst,
                        addr: *srcs.get(0).unwrap(),
                        res: *dsts.get(0).unwrap(),
                    }
                }

                // scope
                Operation::PackRef => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    let src_slot = src_slots.get(0).unwrap();
                    if !src_slot.ty.is_struct() && !src_slot.ty.is_ref_of_struct(Some(true)) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::StructScopeFini {
                        val: *srcs.get(0).unwrap(),
                    }
                }
                Operation::UnpackRef => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    let src_slot = src_slots.get(0).unwrap();
                    if !src_slot.ty.is_struct() && !src_slot.ty.is_ref_of_struct(Some(true)) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::StructScopeInit {
                        val: *srcs.get(0).unwrap(),
                    }
                }
                Operation::WriteBack(BorrowNode::GlobalRoot(inst_id), BorrowEdge::Strong(edge)) => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    let struct_inst =
                        struct_type(env, inst_id.module_id, inst_id.id, &inst_id.inst, None)?;

                    let src_slot = src_slots.get(0).unwrap();
                    match edge {
                        StrongEdge::Direct => {
                            if !src_slot.ty.is_ref_struct_of(&struct_inst, Some(true)) {
                                util_err_call_operand_type();
                                return None;
                            }

                            // build the instruction
                            Instruction::WriteBackGlobalStruct {
                                inst: struct_inst,
                                src: *srcs.get(0).unwrap(),
                            }
                        }
                        StrongEdge::Field(edge_inst_id, field_num) => {
                            let field_num = *field_num;
                            assert_eq!(inst_id, edge_inst_id);
                            assert!(field_num < struct_inst.fields.len());
                            let struct_field = struct_inst.fields.get(field_num).unwrap();
                            if !src_slot.ty.is_ref_of(&struct_field.ty, Some(true)) {
                                util_err_call_operand_type();
                                return None;
                            }

                            // build the instruction
                            Instruction::WriteBackGlobalField {
                                inst: struct_inst,
                                field_num,
                                src: *srcs.get(0).unwrap(),
                            }
                        }
                        StrongEdge::FieldUnknown => {
                            // cannot have vector type because the global must be a struct type
                            util_err_call_operand_type();
                            return None;
                        }
                    }
                }
                Operation::WriteBack(
                    BorrowNode::LocalRoot(dst_index),
                    BorrowEdge::Strong(edge),
                ) => {
                    let dst_index = *dst_index;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }
                    if util_is_index_oob(dst_index) {
                        return None;
                    }

                    // check operand types
                    let dst_slot = local_slots.get(dst_index).unwrap();
                    let src_slot = src_slots.get(0).unwrap();
                    match edge {
                        StrongEdge::Direct => match dst_slot.ty.get_base_type() {
                            None => {
                                util_err_call_operand_type_ext(vec![dst_slot]);
                                return None;
                            }
                            Some(base_ty) => {
                                if !src_slot.ty.is_ref_of(base_ty, Some(true)) {
                                    util_err_call_operand_type_ext(vec![dst_slot]);
                                    return None;
                                }

                                // build the instruction
                                Instruction::WriteBackLocalValue {
                                    src: *srcs.get(0).unwrap(),
                                    dst: dst_index,
                                }
                            }
                        },
                        StrongEdge::Field(edge_inst_id, field_num) => {
                            match dst_slot.ty.get_struct_instantiation() {
                                None => {
                                    util_err_call_operand_type_ext(vec![dst_slot]);
                                    return None;
                                }
                                Some(struct_inst) => {
                                    let field_num = *field_num;
                                    let edge_struct_inst = struct_type(
                                        env,
                                        edge_inst_id.module_id,
                                        edge_inst_id.id,
                                        &edge_inst_id.inst,
                                        None,
                                    )?;
                                    assert_eq!(struct_inst, &edge_struct_inst);
                                    assert!(field_num < struct_inst.fields.len());
                                    let struct_field = struct_inst.fields.get(field_num).unwrap();
                                    if !src_slot.ty.is_ref_of(&struct_field.ty, Some(true)) {
                                        util_err_call_operand_type_ext(vec![dst_slot]);
                                        return None;
                                    }

                                    // build the instruction
                                    Instruction::WriteBackLocalField {
                                        inst: edge_struct_inst,
                                        field_num,
                                        src: *srcs.get(0).unwrap(),
                                        dst: dst_index,
                                    }
                                }
                            }
                        }
                        StrongEdge::FieldUnknown => match dst_slot.ty.get_vector_element_type() {
                            None => {
                                util_err_call_operand_type_ext(vec![dst_slot]);
                                return None;
                            }
                            Some(elem_ty) => {
                                if !src_slot.ty.is_ref_of(elem_ty, Some(true)) {
                                    util_err_call_operand_type_ext(vec![dst_slot]);
                                    return None;
                                }

                                // build the instruction
                                Instruction::WriteBackLocalVecElement {
                                    src: *srcs.get(0).unwrap(),
                                    dst: dst_index,
                                }
                            }
                        },
                    }
                }
                Operation::WriteBack(
                    BorrowNode::Reference(dst_index),
                    BorrowEdge::Strong(edge),
                ) => {
                    let dst_index = *dst_index;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }
                    if util_is_index_oob(dst_index) {
                        return None;
                    }

                    // check operand types
                    let dst_slot = local_slots.get(dst_index).unwrap();
                    match dst_slot.ty.get_ref_type() {
                        None => {
                            util_err_call_operand_type_ext(vec![dst_slot]);
                            return None;
                        }
                        Some((false, _)) => {
                            util_err_call_operand_type_ext(vec![dst_slot]);
                            return None;
                        }
                        Some((_, base_ty)) => {
                            let src_slot = src_slots.get(0).unwrap();
                            match edge {
                                StrongEdge::Direct => {
                                    if !src_slot.ty.is_ref_of(base_ty, Some(true)) {
                                        util_err_call_operand_type_ext(vec![dst_slot]);
                                        return None;
                                    }

                                    // build the instruction
                                    Instruction::WriteBackRefValue {
                                        src: *srcs.get(0).unwrap(),
                                        dst: dst_index,
                                    }
                                }
                                StrongEdge::Field(edge_inst_id, field_num) => {
                                    match base_ty.get_struct_instantiation() {
                                        None => {
                                            util_err_call_operand_type_ext(vec![dst_slot]);
                                            return None;
                                        }
                                        Some(struct_inst) => {
                                            let field_num = *field_num;
                                            let edge_struct_inst = struct_type(
                                                env,
                                                edge_inst_id.module_id,
                                                edge_inst_id.id,
                                                &edge_inst_id.inst,
                                                None,
                                            )?;
                                            assert_eq!(struct_inst, &edge_struct_inst);
                                            assert!(field_num < struct_inst.fields.len());
                                            let struct_field =
                                                struct_inst.fields.get(field_num).unwrap();
                                            if !src_slot.ty.is_ref_of(&struct_field.ty, Some(true))
                                            {
                                                util_err_call_operand_type_ext(vec![dst_slot]);
                                                return None;
                                            }

                                            // build the instruction
                                            Instruction::WriteBackRefField {
                                                inst: edge_struct_inst,
                                                field_num,
                                                src: *srcs.get(0).unwrap(),
                                                dst: dst_index,
                                            }
                                        }
                                    }
                                }
                                StrongEdge::FieldUnknown => {
                                    match base_ty.get_vector_element_type() {
                                        None => {
                                            util_err_call_operand_type_ext(vec![dst_slot]);
                                            return None;
                                        }
                                        Some(elem_ty) => {
                                            if !src_slot.ty.is_ref_of(elem_ty, Some(true)) {
                                                util_err_call_operand_type_ext(vec![dst_slot]);
                                                return None;
                                            }

                                            // build the instruction
                                            Instruction::WriteBackRefVecElement {
                                                src: *srcs.get(0).unwrap(),
                                                dst: dst_index,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Operation::Splice(..)
                | Operation::PackRefDeep
                | Operation::UnpackRefDeep
                | Operation::WriteBack(_, BorrowEdge::Weak) => {
                    // deprecated
                    return None;
                }

                // references
                Operation::BorrowLoc => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    match src_slots.get(0).unwrap().ty.get_base_type() {
                        None => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some(base_ty) => {
                            if !dst_slots.get(0).unwrap().ty.is_ref_of(base_ty, None) {
                                util_err_call_operand_type();
                                return None;
                            }
                        }
                    }

                    // build the instruction
                    Instruction::BorrowLocal {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::ReadRef => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    match src_slots.get(0).unwrap().ty.get_ref_type() {
                        None => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some((_, base_ty)) => {
                            if !dst_slots.get(0).unwrap().ty.is_base_of(base_ty) {
                                util_err_call_operand_type();
                                return None;
                            }
                        }
                    }

                    // build the instruction
                    Instruction::ReadRef {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::WriteRef => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 0) {
                        return None;
                    }

                    // check operand types
                    match src_slots.get(0).unwrap().ty.get_ref_type() {
                        None => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some((false, _)) => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some((_, base_ty)) => {
                            if !src_slots.get(1).unwrap().ty.is_base_of(base_ty) {
                                util_err_call_operand_type();
                                return None;
                            }
                        }
                    }

                    // build the instruction
                    Instruction::WriteRef {
                        src: *srcs.get(1).unwrap(),
                        dst: *srcs.get(0).unwrap(),
                    }
                }
                Operation::FreezeRef => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    match src_slots.get(0).unwrap().ty.get_ref_type() {
                        None => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some((false, _)) => {
                            util_err_call_operand_type();
                            return None;
                        }
                        Some((_, base_ty)) => {
                            if !dst_slots.get(0).unwrap().ty.is_ref_of(base_ty, Some(false)) {
                                util_err_call_operand_type();
                                return None;
                            }
                        }
                    }

                    // build the instruction
                    Instruction::FreezeRef {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // built-in
                Operation::Destroy => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // build the instruction
                    Instruction::Destroy {
                        index: *srcs.get(0).unwrap(),
                    }
                }
                Operation::Havoc(kind) => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    let src_slot = src_slots.get(0).unwrap();
                    match kind {
                        HavocKind::Value => {
                            if !src_slot.ty.is_base() {
                                util_err_call_operand_type();
                                return None;
                            }

                            // build the instruction
                            Instruction::HavocValue {
                                index: *srcs.get(0).unwrap(),
                            }
                        }
                        HavocKind::MutationValue => match src_slot.ty.get_ref_type() {
                            None => {
                                util_err_call_operand_type();
                                return None;
                            }
                            Some((false, _)) => {
                                util_err_call_operand_type();
                                return None;
                            }
                            _ => {
                                // build the instruction
                                Instruction::HavocDeref {
                                    index: *srcs.get(0).unwrap(),
                                }
                            }
                        },
                        HavocKind::MutationAll => match src_slot.ty.get_ref_type() {
                            None => {
                                util_err_call_operand_type();
                                return None;
                            }
                            Some((false, _)) => {
                                util_err_call_operand_type();
                                return None;
                            }
                            _ => {
                                // build the instruction
                                Instruction::HavocMutation {
                                    index: *srcs.get(0).unwrap(),
                                }
                            }
                        },
                    }
                }
                Operation::Stop(label) => {
                    // NOTE: Stop is an operation created for loop-to-DAG transformation. It holds
                    // a label of the original loop header (i.e., where the back-edge goes to)
                    Instruction::Stop { label: *label }
                }

                // cast
                Operation::CastU8 => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_int() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_u8() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::CastU8 {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::CastU64 => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_int() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_u64() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::CastU64 {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::CastU128 => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_int() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_u128() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::CastU128 {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }

                // unary
                Operation::Not => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Not {
                        src: *srcs.get(0).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // binary (arithmetic)
                Operation::Add => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_arithmetic(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Add {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::Sub => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_arithmetic(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Sub {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::Mul => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_arithmetic(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Mul {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::Div => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_arithmetic(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Div {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }
                Operation::Mod => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_arithmetic(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Mod {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                        on_abort: on_abort.clone(),
                    }
                }

                // binary (bitwise)
                Operation::BitAnd => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_bitwise(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BitAnd {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::BitOr => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_bitwise(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BitOr {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Xor => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_compatible_for_bitwise(
                        &src_slots.get(0).unwrap().ty,
                        &src_slots.get(1).unwrap().ty,
                    ) {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BitXor {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Shl => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_bitshift(&src_slots.get(0).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !src_slots.get(1).unwrap().ty.is_u8() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BitShl {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Shr => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_bitshift(&src_slots.get(0).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !src_slots.get(1).unwrap().ty.is_u8() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::BitShr {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // binary (comparison)
                Operation::Lt => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_comparison(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Lt {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Le => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_comparison(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Le {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Ge => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_comparison(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Ge {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Gt => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_comparison(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Gt {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // binary (equality)
                Operation::Eq => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_equality(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Eq {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Neq => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !src_slots
                        .get(0)
                        .unwrap()
                        .ty
                        .is_compatible_for_equality(&src_slots.get(1).unwrap().ty)
                    {
                        util_err_call_operand_type();
                        return None;
                    }
                    if !dst_slots.get(0).unwrap().ty.is_bool() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Neq {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // binary (boolean)
                Operation::And => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_bool()
                        || !src_slots.get(0).unwrap().ty.is_bool()
                        || !src_slots.get(1).unwrap().ty.is_bool()
                    {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::And {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }
                Operation::Or => {
                    // check operand bounds
                    if util_operand_counts_mismatch(2, 1) {
                        return None;
                    }

                    // check operand types
                    if !dst_slots.get(0).unwrap().ty.is_bool()
                        || !src_slots.get(0).unwrap().ty.is_bool()
                        || !src_slots.get(1).unwrap().ty.is_bool()
                    {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Or {
                        lhs: *srcs.get(0).unwrap(),
                        rhs: *srcs.get(1).unwrap(),
                        dst: *dsts.get(0).unwrap(),
                    }
                }

                // Debugging
                Operation::TraceLocal(index) => {
                    let index = *index;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }
                    if util_is_index_oob(index) {
                        return None;
                    }

                    // check operand types
                    let trace_slot = local_slots.get(index).unwrap();
                    if trace_slot.ty != src_slots.get(0).unwrap().ty {
                        util_err_call_operand_type_ext(vec![trace_slot]);
                        return None;
                    }

                    // build the instruction
                    Instruction::Nop
                }
                Operation::TraceReturn(index) => {
                    let index = *index;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    if index >= target.get_return_count() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Nop
                }
                Operation::TraceAbort => {
                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    if !src_slots.get(0).unwrap().ty.is_compatible_for_abort_code() {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Nop
                }
                Operation::TraceExp(node_id) => {
                    let node_id = *node_id;

                    // check operand bounds
                    if util_operand_counts_mismatch(1, 0) {
                        return None;
                    }

                    // check operand types
                    let node_ty = local_type(env, &env.get_node_type(node_id), None)?;
                    if src_slots.get(0).unwrap().ty != node_ty {
                        util_err_call_operand_type();
                        return None;
                    }

                    // build the instruction
                    Instruction::Nop
                }

                // event (TODO: not supported yet)
                Operation::EmitEvent | Operation::EventStoreDiverge => Instruction::Nop,
            }
        }

        Bytecode::Label(_, label) => {
            // TODO (mengxu): ideally, should at least assert that the labels in jump and branch
            // are valid labels, leave this as a TODO for now.
            Instruction::Label { label: *label }
        }
        Bytecode::Jump(_, label) => Instruction::Jump { label: *label },
        Bytecode::Branch(_, then_label, else_label, cond) => {
            let cond = *cond;

            // check bounds
            if util_is_index_oob(cond) {
                return None;
            }

            // check types
            let cond_slot = local_slots.get(cond).unwrap();
            if !cond_slot.ty.is_bool() {
                util_err_operand_type(vec![cond_slot]);
                return None;
            }

            // build the instruction
            Instruction::Branch {
                then_label: *then_label,
                else_label: *else_label,
                cond,
            }
        }

        Bytecode::Abort(_, index) => {
            let index = *index;

            // check bounds
            if util_is_index_oob(index) {
                return None;
            }

            // check types
            let index_slot = local_slots.get(index).unwrap();
            if !index_slot.ty.is_compatible_for_abort_code() {
                util_err_operand_type(vec![index_slot]);
                return None;
            }

            // build the instruction
            Instruction::Abort { index }
        }
        Bytecode::Ret(_, tuple) => {
            assert_eq!(tuple.len(), target.get_return_count());

            // check bounds
            for index in tuple.iter().copied() {
                if util_is_index_oob(index) {
                    return None;
                }
            }

            // check types
            for (index, ret_decl_ty) in tuple.iter().copied().zip(target.get_return_types()) {
                let ret_slot = local_slots.get(index).unwrap();
                match local_type(env, ret_decl_ty, None) {
                    None => {
                        status::error_return_model_type_is_invalid(target, ret_decl_ty, index);
                        return None;
                    }
                    Some(ret_ty) => {
                        if ret_slot.ty != ret_ty {
                            status::error_return_type_mismatches_declared_return_type(
                                target,
                                index,
                                &ret_slot.ty,
                                &ret_ty,
                            );
                            return None;
                        }
                    }
                }
            }

            // build the instruction
            Instruction::Return {
                tuple: tuple.clone(),
            }
        }

        Bytecode::SaveMem(_, mem_label, inst_id) => {
            let mem_label = *mem_label;

            // check operand types
            let struct_inst = struct_type(env, inst_id.module_id, inst_id.id, &inst_id.inst, None)?;
            if !global_slots
                .entry(mem_label)
                .or_default()
                .insts
                .insert(struct_inst.clone())
            {
                status::error_global_slot_type_mismatch(
                    target,
                    global_slots,
                    mem_label,
                    &struct_inst,
                );
                return None;
            }

            // build the instruction
            Instruction::SaveGlobal {
                label: mem_label,
                inst: struct_inst,
            }
        }

        // TODO (mengxu) handle expression-based havocing
        Bytecode::Prop(_, PropKind::Modifies, _) => Instruction::Nop,

        Bytecode::Prop(_, kind, exp) => {
            match local_type(env, &env.get_node_type(exp.node_id()), None) {
                None => {
                    util_err_operand_type(vec![]);
                    return None;
                }
                Some(exp_ty) => {
                    if !exp_ty.is_bool() {
                        util_err_operand_type(vec![]);
                        return None;
                    }
                }
            };
            // TODO (mengxu) check and convert expressions
            let exp = exp.clone();
            match kind {
                PropKind::Assume => Instruction::Assume { exp },
                PropKind::Assert => Instruction::Assert { exp },
                PropKind::Modifies => unreachable!(),
            }
        }

        Bytecode::Nop(_) => Instruction::Nop,

        // deprecated bytecode
        Bytecode::SpecBlock(..) | Bytecode::SaveSpecVar(..) => {
            return None;
        }
    };
    Some(instruction)
}
