// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode::{function_target::FunctionTarget, stackless_bytecode::Bytecode};
use move_model::{
    ast::{MemoryLabel, TempIndex},
    ty as MT,
};
use std::collections::BTreeMap;

use crate::{
    assembly::{
        ast::GlobalSlot,
        ty::{StructInstantiation, Type},
    },
    shared::ident::FunctionIdent,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

pub fn error_duplicated_function(target: &FunctionTarget, ident: &FunctionIdent) {
    let env = target.global_env();
    env.error(
        &target.get_loc(),
        &format!("Duplicated function: {}", ident),
    );
}

//**************************************************************************************************
// Type
//**************************************************************************************************

pub fn error_local_model_type_mismatches_declared_parameter_type(
    target: &FunctionTarget,
    param_decl_ty: &MT::Type,
    local_ty: &MT::Type,
    local_index: TempIndex,
    local_name: &str,
) {
    let env = target.global_env();
    let ctxt = target.func_env.get_type_display_ctxt();
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Local type does not match declared parameter type {} - {}: {} vs {}",
            local_index,
            local_name,
            local_ty.display(&ctxt),
            param_decl_ty.display(&ctxt),
        ),
    );
}

pub fn error_local_slot_type_is_invalid(
    target: &FunctionTarget,
    local_ty: &MT::Type,
    local_index: TempIndex,
    local_name: &str,
) {
    let env = target.global_env();
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Invalid type for local slot {} - {}: {}",
            local_index,
            local_name,
            local_ty.display(&target.func_env.get_type_display_ctxt())
        ),
    );
}

pub fn error_global_slot_type_mismatch(
    target: &FunctionTarget,
    global_slots: &BTreeMap<MemoryLabel, GlobalSlot>,
    mem_label: MemoryLabel,
    expected_inst: &StructInstantiation,
) {
    let env = target.global_env();
    let tokens: Vec<_> = global_slots
        .get(&mem_label)
        .unwrap()
        .insts
        .iter()
        .map(|inst| inst.to_string())
        .collect();
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Global slot type does not match with the instantiation {}: [{}] vs {}",
            mem_label,
            tokens.join(" | "),
            expected_inst
        ),
    );
}

pub fn error_return_model_type_is_invalid(
    target: &FunctionTarget,
    ret_ty: &MT::Type,
    ret_index: TempIndex,
) {
    let env = target.global_env();
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Invalid type for return value {}: {}",
            ret_index,
            ret_ty.display(&target.func_env.get_type_display_ctxt())
        ),
    );
}

pub fn error_return_type_mismatches_declared_return_type(
    target: &FunctionTarget,
    ret_index: TempIndex,
    ret_ty: &Type,
    ret_decl_ty: &Type,
) {
    let env = target.global_env();
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Return slot type does not match with declared return type {}: {} vs {}",
            ret_index, ret_ty, ret_decl_ty,
        ),
    );
}

//**************************************************************************************************
// Bytecode
//**************************************************************************************************

pub fn error_bytecode_is_invalid(target: &FunctionTarget, offset: usize, bytecode: &Bytecode) {
    let env = target.global_env();
    let label_offsets = Bytecode::label_offsets(target.get_bytecode());
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Invalid bytecode at offset {}: {}",
            offset,
            bytecode.display(target, &label_offsets)
        ),
    );
}

pub fn error_bytecode_local_slot_access_out_of_bound(
    target: &FunctionTarget,
    bytecode: &Bytecode,
    index: TempIndex,
) {
    let env = target.global_env();
    let label_offsets = Bytecode::label_offsets(target.get_bytecode());
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Out-of-bound access to local slot {} in bytecode {}",
            index,
            bytecode.display(target, &label_offsets)
        ),
    );
}

pub fn error_bytecode_operand_type_incompatible<'a>(
    target: &FunctionTarget,
    bytecode: &Bytecode,
    operand_tys: impl IntoIterator<Item = &'a Type>,
) {
    let env = target.global_env();
    let label_offsets = Bytecode::label_offsets(target.get_bytecode());
    let tys_repr = operand_tys
        .into_iter()
        .map(|ty| ty.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Incompatible operand type(s) ({}) for bytecode {}",
            tys_repr,
            bytecode.display(target, &label_offsets)
        ),
    );
}

pub fn error_bytecode_operand_count_mismatch(
    target: &FunctionTarget,
    bytecode: &Bytecode,
    srcs_len: usize,
    dsts_len: usize,
) {
    let env = target.global_env();
    let label_offsets = Bytecode::label_offsets(target.get_bytecode());
    env.error(
        &target.func_env.get_loc(),
        &format!(
            "Incompatible operand count(s) ({}) -> {} for bytecode {}",
            srcs_len,
            dsts_len,
            bytecode.display(target, &label_offsets)
        ),
    );
}
