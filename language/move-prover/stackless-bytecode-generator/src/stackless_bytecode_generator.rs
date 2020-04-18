// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    annotations::Annotations,
    function_target::FunctionTargetData,
    stackless_bytecode::{
        AssignKind, AttrId, BranchCond,
        Bytecode::{self},
        Constant, Label, Operation, SpecBlockId, TempIndex,
    },
};
use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use move_vm_types::values::Value as VMValue;
use spec_lang::{
    env::{FunctionEnv, Loc, ModuleEnv, StructId},
    ty::{PrimitiveType, Type},
};
use std::{collections::BTreeMap, matches};
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode as MoveBytecode, CodeOffset, CompiledModule, FieldHandleIndex, SignatureIndex,
    },
    views::{FunctionHandleView, ViewInternals},
};

pub struct StacklessBytecodeGenerator<'a> {
    func_env: &'a FunctionEnv<'a>,
    module: &'a CompiledModule,
    temp_count: usize,
    temp_stack: Vec<usize>,
    local_types: Vec<Type>,
    code: Vec<Bytecode>,
    location_table: BTreeMap<AttrId, Loc>,
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn new(func_env: &'a FunctionEnv<'a>) -> Self {
        let local_types = (0..func_env.get_local_count())
            .map(|i| func_env.get_local_type(i))
            .collect_vec();
        StacklessBytecodeGenerator {
            func_env,
            module: func_env.module_env.get_verified_module(),
            temp_count: local_types.len(),
            temp_stack: vec![],
            local_types,
            code: vec![],
            location_table: BTreeMap::new(),
        }
    }

    pub fn generate_function(mut self) -> FunctionTargetData {
        let original_code = self.func_env.get_bytecode();
        let mut label_map = BTreeMap::new();

        // Generate labels.
        for bytecode in original_code {
            if let MoveBytecode::BrTrue(code_offset)
            | MoveBytecode::BrFalse(code_offset)
            | MoveBytecode::Branch(code_offset) = bytecode
            {
                let label = Label::new(label_map.len());
                label_map.insert(*code_offset as CodeOffset, label);
            }
        }
        // Generate bytecode.
        let mut given_spec_blocks = BTreeMap::new();
        for (code_offset, bytecode) in original_code.iter().enumerate() {
            self.generate_bytecode(
                bytecode,
                code_offset as CodeOffset,
                &label_map,
                &mut given_spec_blocks,
            );
        }

        FunctionTargetData {
            code: self.code,
            local_types: self.local_types,
            return_types: self.func_env.get_return_types(),
            locations: self.location_table,
            annotations: Annotations::default(),
            given_spec_blocks,
            generated_spec_blocks: BTreeMap::new(),
        }
    }

    /// Create a new attribute id and populate location table.
    fn new_loc_attr(&mut self, code_offset: CodeOffset) -> AttrId {
        let loc = self.func_env.get_bytecode_loc(code_offset);
        let attr = AttrId::new(self.location_table.len());
        self.location_table.insert(attr.clone(), loc);
        attr
    }

    fn get_field_info(&self, field_handle_index: FieldHandleIndex) -> (StructId, usize, Type) {
        let field_handle = self.module.field_handle_at(field_handle_index);
        let struct_id = self.func_env.module_env.get_struct_id(field_handle.owner);
        let struct_env = self.func_env.module_env.get_struct(struct_id);
        let field_env = struct_env.get_field_by_offset(field_handle.field as usize);
        (struct_id, field_handle.field as usize, field_env.get_type())
    }

    fn get_type_params(&self, type_params_index: SignatureIndex) -> Vec<Type> {
        self.func_env
            .module_env
            .get_type_actuals(Some(type_params_index))
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn generate_bytecode(
        &mut self,
        bytecode: &MoveBytecode,
        code_offset: CodeOffset,
        label_map: &BTreeMap<CodeOffset, Label>,
        spec_blocks: &mut BTreeMap<SpecBlockId, CodeOffset>,
    ) {
        // Add label if defined at this code offset.
        if let Some(label) = label_map.get(&code_offset) {
            let label_attr_id = self.new_loc_attr(code_offset);
            self.code.push(Bytecode::Label(label_attr_id, *label));
        }

        // Add spec block if defined at this code offset.
        if self
            .func_env
            .get_specification_on_impl(code_offset)
            .is_some()
        {
            let block_id = SpecBlockId::new(spec_blocks.len());
            spec_blocks.insert(block_id, code_offset);
            let spec_block_attr_id = self.new_loc_attr(code_offset);
            self.code
                .push(Bytecode::SpecBlock(spec_block_attr_id, block_id));

            // If the current instruction is just a Nop, skip it. It has been generated tp support
            // spec blocks.
            if matches!(bytecode, MoveBytecode::Nop) {
                return;
            }
        }

        let attr_id = self.new_loc_attr(code_offset);

        let mk_call = |op: Operation, dsts: Vec<usize>, srcs: Vec<usize>| -> Bytecode {
            Bytecode::Call(attr_id, dsts, op, srcs)
        };
        let mk_unary = |op: Operation, dst: usize, src: usize| -> Bytecode {
            Bytecode::Call(attr_id, vec![dst], op, vec![src])
        };
        let mk_binary = |op: Operation, dst: usize, src1: usize, src2: usize| -> Bytecode {
            Bytecode::Call(attr_id, vec![dst], op, vec![src1, src2])
        };

        match bytecode {
            MoveBytecode::Pop => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(mk_call(Operation::Destroy, vec![], vec![temp_index]));
            }
            MoveBytecode::BrTrue(target) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Branch(
                    attr_id,
                    *label_map.get(target).unwrap(),
                    BranchCond::True(temp_index),
                ));
            }

            MoveBytecode::BrFalse(target) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Branch(
                    attr_id,
                    *label_map.get(target).unwrap(),
                    BranchCond::False(temp_index),
                ));
            }

            MoveBytecode::Abort => {
                let error_code_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(mk_call(Operation::Abort, vec![], vec![error_code_index]));
            }

            MoveBytecode::StLoc(idx) => {
                let operand_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Assign(
                    attr_id,
                    *idx as TempIndex,
                    operand_index,
                    AssignKind::Store,
                ));
            }

            MoveBytecode::Ret => {
                let mut return_temps = vec![];
                for _ in 0..self.func_env.get_return_count() {
                    let return_temp_index = self.temp_stack.pop().unwrap();
                    return_temps.push(return_temp_index);
                }
                return_temps.reverse();
                self.code.push(Bytecode::Ret(attr_id, return_temps));
            }

            MoveBytecode::Branch(target) => {
                self.code.push(Bytecode::Branch(
                    attr_id,
                    *label_map.get(target).unwrap(),
                    BranchCond::Always,
                ));
            }

            MoveBytecode::FreezeRef => {
                let mutable_ref_index = self.temp_stack.pop().unwrap();
                let mutable_ref_sig = self.local_types[mutable_ref_index].clone();
                if let Type::Reference(is_mut, signature) = mutable_ref_sig {
                    if is_mut {
                        let immutable_ref_index = self.temp_count;
                        self.temp_stack.push(immutable_ref_index);
                        self.local_types.push(Type::Reference(false, signature));
                        self.code.push(mk_call(
                            Operation::FreezeRef,
                            vec![immutable_ref_index],
                            vec![mutable_ref_index],
                        ));
                        self.temp_count += 1;
                    }
                }
            }

            MoveBytecode::ImmBorrowField(field_handle_index)
            | MoveBytecode::MutBorrowField(field_handle_index) => {
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let (struct_id, field_offset, field_type) =
                    self.get_field_info(*field_handle_index);
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(mk_call(
                    Operation::BorrowField(
                        self.func_env.module_env.get_id(),
                        struct_id,
                        vec![],
                        field_offset,
                    ),
                    vec![field_ref_index],
                    vec![struct_ref_index],
                ));
                self.temp_count += 1;
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowField(..));
                self.local_types
                    .push(Type::Reference(is_mut, Box::new(field_type)));
            }

            MoveBytecode::ImmBorrowFieldGeneric(field_inst_index)
            | MoveBytecode::MutBorrowFieldGeneric(field_inst_index) => {
                let field_inst = self.module.field_instantiation_at(*field_inst_index);
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let (struct_id, field_offset, field_signature) =
                    self.get_field_info(field_inst.handle);
                let type_sigs = self
                    .func_env
                    .module_env
                    .globalize_signatures(&self.module.signature_at(field_inst.type_parameters).0);
                let field_type = match field_signature {
                    Type::TypeParameter(i) => type_sigs[i as usize].clone(),
                    _ => field_signature,
                };
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(mk_call(
                    Operation::BorrowField(
                        self.func_env.module_env.get_id(),
                        struct_id,
                        type_sigs,
                        field_offset,
                    ),
                    vec![field_ref_index],
                    vec![struct_ref_index],
                ));
                self.temp_count += 1;
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowFieldGeneric(..));
                self.local_types
                    .push(Type::Reference(is_mut, Box::new(field_type)));
            }

            MoveBytecode::LdU8(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U8));
                self.code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U8(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU64(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U64));
                self.code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U64(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU128(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U128));
                self.code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U128(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::CastU8 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U8));
                self.code
                    .push(mk_unary(Operation::CastU8, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU64 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U64));
                self.code
                    .push(mk_unary(Operation::CastU64, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU128 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U128));
                self.code
                    .push(mk_unary(Operation::CastU128, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::LdConst(idx) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                let constant = self.func_env.module_env.get_constant(*idx);
                let ty = self
                    .func_env
                    .module_env
                    .globalize_signature(&constant.type_);
                let value = Self::translate_value(
                    &ty,
                    self.func_env.module_env.get_constant_value(constant),
                );
                self.local_types.push(ty);
                self.code.push(Bytecode::Load(attr_id, temp_index, value));
                self.temp_count += 1;
            }

            MoveBytecode::LdTrue => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::Bool(true)));
                self.temp_count += 1;
            }

            MoveBytecode::LdFalse => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::Bool(false)));
                self.temp_count += 1;
            }

            MoveBytecode::CopyLoc(idx) => {
                let signature = self.func_env.get_local_type(*idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(Bytecode::Assign(
                    attr_id,
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Copy,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MoveLoc(idx) => {
                let signature = self.func_env.get_local_type(*idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(Bytecode::Assign(
                    attr_id,
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Move,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MutBorrowLoc(idx) => {
                let signature = self.func_env.get_local_type(*idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Reference(true, Box::new(signature)));
                self.code.push(mk_unary(
                    Operation::BorrowLoc,
                    temp_index,
                    *idx as TempIndex,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::ImmBorrowLoc(idx) => {
                let signature = self.func_env.get_local_type(*idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Reference(false, Box::new(signature)));
                self.code.push(mk_unary(
                    Operation::BorrowLoc,
                    temp_index,
                    *idx as TempIndex,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::Call(idx) => {
                let function_handle = self.module.function_handle_at(*idx);
                let function_handle_view = FunctionHandleView::new(self.module, function_handle);

                let mut arg_temp_indices = vec![];
                let mut return_temp_indices = vec![];
                for _ in function_handle_view.arg_tokens() {
                    let arg_temp_index = self.temp_stack.pop().unwrap();
                    arg_temp_indices.push(arg_temp_index);
                }
                for return_type_view in function_handle_view.return_tokens() {
                    let return_temp_index = self.temp_count;
                    let return_type = self
                        .func_env
                        .module_env
                        .globalize_signature(&return_type_view.as_inner());
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    self.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                return_temp_indices.reverse();
                let callee_env = self.func_env.module_env.get_called_function(*idx);
                self.code.push(mk_call(
                    Operation::Function(
                        callee_env.module_env.get_id(),
                        callee_env.get_id(),
                        vec![],
                    ),
                    return_temp_indices,
                    arg_temp_indices,
                ))
            }
            MoveBytecode::CallGeneric(idx) => {
                let func_instantiation = self.module.function_instantiation_at(*idx);

                let type_sigs = self.get_type_params(func_instantiation.type_parameters);
                let function_handle = self.module.function_handle_at(func_instantiation.handle);
                let function_handle_view = FunctionHandleView::new(self.module, function_handle);

                let mut arg_temp_indices = vec![];
                let mut return_temp_indices = vec![];
                for _ in function_handle_view.arg_tokens() {
                    let arg_temp_index = self.temp_stack.pop().unwrap();
                    arg_temp_indices.push(arg_temp_index);
                }
                for return_type_view in function_handle_view.return_tokens() {
                    let return_temp_index = self.temp_count;
                    // instantiate type parameters
                    let return_type = self
                        .func_env
                        .module_env
                        .globalize_signature(&return_type_view.as_inner())
                        .instantiate(&type_sigs);
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    self.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                return_temp_indices.reverse();
                let callee_env = self
                    .func_env
                    .module_env
                    .get_called_function(func_instantiation.handle);
                self.code.push(mk_call(
                    Operation::Function(
                        callee_env.module_env.get_id(),
                        callee_env.get_id(),
                        type_sigs,
                    ),
                    return_temp_indices,
                    arg_temp_indices,
                ))
            }

            MoveBytecode::Pack(idx) => {
                let struct_env = self.func_env.module_env.get_struct_by_def_idx(*idx);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_env.get_fields() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                self.local_types.push(Type::Struct(
                    struct_env.module_env.get_id(),
                    struct_env.get_id(),
                    vec![],
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.code.push(mk_call(
                    Operation::Pack(struct_env.module_env.get_id(), struct_env.get_id(), vec![]),
                    vec![struct_temp_index],
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::PackGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                let struct_env = self
                    .func_env
                    .module_env
                    .get_struct_by_def_idx(struct_instantiation.def);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_env.get_fields() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                self.local_types.push(Type::Struct(
                    struct_env.module_env.get_id(),
                    struct_env.get_id(),
                    actuals.clone(),
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.code.push(mk_call(
                    Operation::Pack(struct_env.module_env.get_id(), struct_env.get_id(), actuals),
                    vec![struct_temp_index],
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::Unpack(idx) => {
                let struct_env = self.func_env.module_env.get_struct_by_def_idx(*idx);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_env in struct_env.get_fields() {
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types.push(field_env.get_type());
                    self.temp_count += 1;
                }
                self.code.push(mk_call(
                    Operation::Unpack(struct_env.module_env.get_id(), struct_env.get_id(), vec![]),
                    field_temp_indices,
                    vec![struct_temp_index],
                ));
            }

            MoveBytecode::UnpackGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                let struct_env = self
                    .func_env
                    .module_env
                    .get_struct_by_def_idx(struct_instantiation.def);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_env in struct_env.get_fields() {
                    let field_type = field_env.get_type().instantiate(&actuals);
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types.push(field_type);
                    self.temp_count += 1;
                }
                self.code.push(mk_call(
                    Operation::Unpack(struct_env.module_env.get_id(), struct_env.get_id(), actuals),
                    field_temp_indices,
                    vec![struct_temp_index],
                ));
            }

            MoveBytecode::ReadRef => {
                let operand_index = self.temp_stack.pop().unwrap();
                let operand_sig = self.local_types[operand_index].clone();
                let temp_index = self.temp_count;
                if let Type::Reference(_, signature) = operand_sig {
                    self.local_types.push(*signature);
                }
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code
                    .push(mk_unary(Operation::ReadRef, temp_index, operand_index));
            }

            MoveBytecode::WriteRef => {
                let ref_operand_index = self.temp_stack.pop().unwrap();
                let val_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(mk_unary(
                    Operation::WriteRef,
                    ref_operand_index,
                    val_operand_index,
                ));
            }

            MoveBytecode::Add
            | MoveBytecode::Sub
            | MoveBytecode::Mul
            | MoveBytecode::Mod
            | MoveBytecode::Div
            | MoveBytecode::BitOr
            | MoveBytecode::BitAnd
            | MoveBytecode::Xor
            | MoveBytecode::Shl
            | MoveBytecode::Shr => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let operand_type = self.local_types[operand1_index].clone();
                let temp_index = self.temp_count;
                self.local_types.push(operand_type);
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                match bytecode {
                    MoveBytecode::Add => {
                        self.code.push(mk_binary(
                            Operation::Add,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Sub => {
                        self.code.push(mk_binary(
                            Operation::Sub,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mul => {
                        self.code.push(mk_binary(
                            Operation::Mul,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mod => {
                        self.code.push(mk_binary(
                            Operation::Mod,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Div => {
                        self.code.push(mk_binary(
                            Operation::Div,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitOr => {
                        self.code.push(mk_binary(
                            Operation::BitOr,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitAnd => {
                        self.code.push(mk_binary(
                            Operation::BitAnd,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Xor => {
                        self.code.push(mk_binary(
                            Operation::Xor,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shl => {
                        self.code.push(mk_binary(
                            Operation::Shl,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shr => {
                        self.code.push(mk_binary(
                            Operation::Shr,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            MoveBytecode::Or => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_binary(
                    Operation::Or,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            MoveBytecode::And => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_binary(
                    Operation::And,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            MoveBytecode::Not => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code
                    .push(mk_unary(Operation::Not, temp_index, operand_index));
            }
            MoveBytecode::Eq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_binary(
                    Operation::Eq,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            MoveBytecode::Neq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_binary(
                    Operation::Neq,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            MoveBytecode::Lt | MoveBytecode::Gt | MoveBytecode::Le | MoveBytecode::Ge => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                match bytecode {
                    MoveBytecode::Lt => {
                        self.code.push(mk_binary(
                            Operation::Lt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Gt => {
                        self.code.push(mk_binary(
                            Operation::Gt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Le => {
                        self.code.push(mk_binary(
                            Operation::Le,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Ge => {
                        self.code.push(mk_binary(
                            Operation::Ge,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            MoveBytecode::Exists(struct_index) => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_unary(
                    Operation::Exists(
                        self.func_env.module_env.get_id(),
                        self.func_env.module_env.get_struct_id(*struct_index),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::ExistsGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(mk_unary(
                    Operation::Exists(
                        self.func_env.module_env.get_id(),
                        self.func_env
                            .module_env
                            .get_struct_id(struct_instantiation.def),
                        self.get_type_params(struct_instantiation.type_parameters),
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MutBorrowGlobal(idx) | MoveBytecode::ImmBorrowGlobal(idx) => {
                let struct_env = self.func_env.module_env.get_struct_by_def_idx(*idx);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobal(..));
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(Type::Struct(
                        struct_env.module_env.get_id(),
                        struct_env.get_id(),
                        vec![],
                    )),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code.push(mk_unary(
                    Operation::BorrowGlobal(
                        self.func_env.module_env.get_id(),
                        self.func_env.module_env.get_struct_id(*idx),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MutBorrowGlobalGeneric(idx)
            | MoveBytecode::ImmBorrowGlobalGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobalGeneric(..));
                let struct_env = self
                    .func_env
                    .module_env
                    .get_struct_by_def_idx(struct_instantiation.def);

                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                self.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(Type::Struct(
                        struct_env.module_env.get_id(),
                        struct_env.get_id(),
                        actuals.clone(),
                    )),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code.push(mk_unary(
                    Operation::BorrowGlobal(
                        self.func_env.module_env.get_id(),
                        self.func_env
                            .module_env
                            .get_struct_id(struct_instantiation.def),
                        actuals,
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveFrom(idx) => {
                let struct_env = self.func_env.module_env.get_struct_by_def_idx(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Struct(
                    struct_env.module_env.get_id(),
                    struct_env.get_id(),
                    vec![],
                ));
                self.temp_count += 1;
                self.code.push(mk_unary(
                    Operation::MoveFrom(
                        self.func_env.module_env.get_id(),
                        self.func_env.module_env.get_struct_id(*idx),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveFromGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let struct_env = self
                    .func_env
                    .module_env
                    .get_struct_by_def_idx(struct_instantiation.def);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                self.local_types.push(Type::Struct(
                    struct_env.module_env.get_id(),
                    struct_env.get_id(),
                    actuals.clone(),
                ));
                self.temp_count += 1;
                self.code.push(mk_unary(
                    Operation::MoveFrom(
                        self.func_env.module_env.get_id(),
                        self.func_env
                            .module_env
                            .get_struct_id(struct_instantiation.def),
                        actuals,
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveToSender(idx) => {
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(mk_call(
                    Operation::MoveToSender(
                        self.func_env.module_env.get_id(),
                        self.func_env.module_env.get_struct_id(*idx),
                        vec![],
                    ),
                    vec![],
                    vec![value_operand_index],
                ));
            }

            MoveBytecode::MoveToSenderGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(mk_call(
                    Operation::MoveToSender(
                        self.func_env.module_env.get_id(),
                        self.func_env
                            .module_env
                            .get_struct_id(struct_instantiation.def),
                        self.get_type_params(struct_instantiation.type_parameters),
                    ),
                    vec![],
                    vec![value_operand_index],
                ));
            }

            MoveBytecode::GetTxnSenderAddress => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Primitive(PrimitiveType::Address));
                self.code.push(Bytecode::Load(
                    attr_id,
                    temp_index,
                    Constant::TxnSenderAddress,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::GetTxnGasUnitPrice
            | MoveBytecode::GetTxnMaxGasUnits
            | MoveBytecode::GetGasRemaining
            | MoveBytecode::GetTxnSequenceNumber
            | MoveBytecode::GetTxnPublicKey => panic!(
                "MoveBytecode {:?} is deprecated and will be removed soon",
                bytecode
            ),
            MoveBytecode::Nop => self.code.push(Bytecode::Nop(attr_id)),
        }
    }

    fn translate_value(ty: &Type, value: VMValue) -> Constant {
        match ty {
            Type::Vector(inner) => {
                let vs = value.value_as::<Vec<VMValue>>().unwrap();
                let b = vs
                    .into_iter()
                    .map(|v| match Self::translate_value(inner, v) {
                        Constant::U8(u) => u,
                        _ => unimplemented!("Not yet supported constant vector type: {:?}", ty),
                    })
                    .collect::<Vec<u8>>();
                Constant::ByteArray(b)
            }
            Type::Primitive(PrimitiveType::Bool) => {
                Constant::Bool(value.value_as::<bool>().unwrap())
            }
            Type::Primitive(PrimitiveType::U8) => Constant::U8(value.value_as::<u8>().unwrap()),
            Type::Primitive(PrimitiveType::U64) => Constant::U64(value.value_as::<u64>().unwrap()),
            Type::Primitive(PrimitiveType::U128) => {
                Constant::U128(value.value_as::<u128>().unwrap())
            }
            Type::Primitive(PrimitiveType::Address) => {
                let a = value.value_as::<AccountAddress>().unwrap();
                Constant::Address(ModuleEnv::addr_to_big_uint(&a))
            }
            _ => panic!("Unexpected (and possibly invalid) constant type: {:?}", ty),
        }
    }
}
