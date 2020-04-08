// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stackless_bytecode::{
    AssignKind, BinaryOp, BranchCond,
    Bytecode::{self, *},
    Constant, TempIndex, UnaryOp,
};
use spec_lang::{
    env::ModuleEnv,
    ty::{PrimitiveType, Type},
};
use std::{collections::BTreeMap, matches};
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode as MoveBytecode, CodeOffset, CompiledModule, FieldHandleIndex, FunctionDefinition,
        SignatureIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
    },
    views::{FunctionDefinitionView, FunctionHandleView, StructDefinitionView, ViewInternals},
};

pub struct StacklessFunction {
    pub local_types: Vec<Type>,
    pub code: Vec<Bytecode>,
}

pub struct StacklessBytecodeGenerator<'a> {
    module_env: &'a ModuleEnv<'a>,
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    temp_count: usize,
    temp_stack: Vec<usize>,
    local_types: Vec<Type>,
    code: Vec<Bytecode>,
}

pub struct StacklessModuleGenerator<'a> {
    module_env: &'a ModuleEnv<'a>,
    module: &'a CompiledModule,
}

impl<'a> StacklessModuleGenerator<'a> {
    pub fn new(module_env: &'a ModuleEnv<'a>, module: &'a CompiledModule) -> Self {
        StacklessModuleGenerator { module_env, module }
    }

    pub fn generate_module(self) -> Vec<StacklessFunction> {
        self.module
            .function_defs()
            .iter()
            .map(move |function_definition| {
                StacklessBytecodeGenerator::new(self.module_env, self.module, function_definition)
                    .generate_function()
            })
            .collect()
    }
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn new(
        module_env: &'a ModuleEnv<'a>,
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
    ) -> Self {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let mut temp_count = 0;
        let mut local_types = vec![];
        if !function_definition_view.is_native() {
            let locals_signature_view = function_definition_view.locals_signature();
            temp_count = locals_signature_view.len();
            for (_, parameter_view) in locals_signature_view.tokens().enumerate() {
                local_types.push(module_env.globalize_signature(&parameter_view.as_inner()));
            }
        }
        StacklessBytecodeGenerator {
            module_env,
            module,
            function_definition_view,
            temp_count,
            temp_stack: vec![],
            local_types,
            code: vec![],
        }
    }

    pub fn generate_function(mut self) -> StacklessFunction {
        let original_code = &self.function_definition_view.code().code;
        for bytecode in original_code {
            self.generate_bytecode(bytecode);
        }

        StacklessFunction {
            code: self.code,
            local_types: self.local_types,
        }
    }

    fn get_field_info(
        &self,
        field_handle_index: FieldHandleIndex,
    ) -> (StructDefinitionIndex, usize, Type) {
        let field_handle = self.module.field_handle_at(field_handle_index);
        let struct_def = self.module.struct_def_at(field_handle.owner);
        if let StructFieldInformation::Declared(fields) = &struct_def.field_information {
            return (
                field_handle.owner,
                field_handle.field as usize,
                self.module_env
                    .globalize_signature(&fields[field_handle.field as usize].signature.0),
            );
        }
        unreachable!("struct must have fields")
    }

    fn get_type_params(&self, type_params_index: SignatureIndex) -> Vec<Type> {
        self.module_env.get_type_actuals(Some(type_params_index))
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn generate_bytecode(&mut self, bytecode: &MoveBytecode) {
        match bytecode {
            MoveBytecode::Pop => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Destroy(temp_index));
            }
            MoveBytecode::BrTrue(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(Bytecode::Branch(*code_offset, BranchCond::True(temp_index)));
            }

            MoveBytecode::BrFalse(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Branch(
                    *code_offset,
                    BranchCond::False(temp_index),
                ));
            }

            MoveBytecode::Abort => {
                let error_code_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Abort(error_code_index));
            }

            MoveBytecode::StLoc(idx) => {
                let operand_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::Assign(
                    *idx as TempIndex,
                    operand_index,
                    AssignKind::Store,
                ));
            }

            MoveBytecode::Ret => {
                let mut return_temps = vec![];
                for _ in self.function_definition_view.return_tokens() {
                    let return_temp_index = self.temp_stack.pop().unwrap();
                    return_temps.push(return_temp_index);
                }
                return_temps.reverse();
                self.code.push(Bytecode::Ret(return_temps));
            }

            MoveBytecode::Branch(code_offset) => {
                self.code
                    .push(Bytecode::Branch(*code_offset, BranchCond::Always));
            }

            MoveBytecode::FreezeRef => {
                let mutable_ref_index = self.temp_stack.pop().unwrap();
                let mutable_ref_sig = self.local_types[mutable_ref_index].clone();
                if let Type::Reference(is_mut, signature) = mutable_ref_sig {
                    if is_mut {
                        let immutable_ref_index = self.temp_count;
                        self.temp_stack.push(immutable_ref_index);
                        self.local_types.push(Type::Reference(false, signature));
                        self.code
                            .push(Bytecode::FreezeRef(immutable_ref_index, mutable_ref_index));
                        self.temp_count += 1;
                    }
                }
            }

            MoveBytecode::ImmBorrowField(field_handle_index)
            | MoveBytecode::MutBorrowField(field_handle_index) => {
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let (struct_def_idx, field_offset, field_type) =
                    self.get_field_info(*field_handle_index);
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(Bytecode::BorrowField(
                    field_ref_index,
                    struct_ref_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_def_idx),
                    field_offset,
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
                let (struct_def_idx, field_offset, field_signature) =
                    self.get_field_info(field_inst.handle);
                let type_sigs = self
                    .module_env
                    .globalize_signatures(&self.module.signature_at(field_inst.type_parameters).0);
                let field_type = match field_signature {
                    Type::TypeParameter(i) => type_sigs[i as usize].clone(),
                    _ => field_signature,
                };
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(Bytecode::BorrowField(
                    field_ref_index,
                    struct_ref_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_def_idx),
                    field_offset,
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
                    .push(Bytecode::Load(temp_index, Constant::U8(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU64(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U64));
                self.code
                    .push(Bytecode::Load(temp_index, Constant::U64(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU128(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U128));
                self.code
                    .push(Bytecode::Load(temp_index, Constant::U128(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::CastU8 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U8));
                self.code
                    .push(Bytecode::Unary(UnaryOp::CastU8, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU64 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U64));
                self.code
                    .push(Bytecode::Unary(UnaryOp::CastU64, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU128 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::U128));
                self.code.push(Bytecode::Unary(
                    UnaryOp::CastU128,
                    temp_index,
                    operand_index,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::LdAddr(address_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Primitive(PrimitiveType::Address));
                self.code.push(Bytecode::Load(
                    temp_index,
                    Constant::Address(self.module_env.get_address(*address_pool_index)),
                ));
                self.temp_count += 1;
            }

            MoveBytecode::LdByteArray(byte_array_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Vector(Box::new(Type::Primitive(PrimitiveType::U8))));
                self.code.push(Bytecode::Load(
                    temp_index,
                    Constant::ByteArray(
                        self.module_env
                            .get_byte_blob(*byte_array_pool_index)
                            .to_vec(),
                    ),
                ));
                self.temp_count += 1;
            }

            MoveBytecode::LdTrue => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.code
                    .push(Bytecode::Load(temp_index, Constant::Bool(true)));
                self.temp_count += 1;
            }

            MoveBytecode::LdFalse => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.code
                    .push(Bytecode::Load(temp_index, Constant::Bool(false)));
                self.temp_count += 1;
            }

            MoveBytecode::CopyLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = self
                    .module_env
                    .globalize_signature(&locals_signature_view.token_at(*idx).as_inner());
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(Bytecode::Assign(
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Copy,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MoveLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = self
                    .module_env
                    .globalize_signature(&locals_signature_view.token_at(*idx).as_inner());
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(Bytecode::Assign(
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Move,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MutBorrowLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = self
                    .module_env
                    .globalize_signature(&locals_signature_view.token_at(*idx).as_inner());
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Reference(true, Box::new(signature)));
                self.code
                    .push(Bytecode::BorrowLoc(temp_index, *idx as TempIndex));
                self.temp_count += 1;
            }

            MoveBytecode::ImmBorrowLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = self
                    .module_env
                    .globalize_signature(&locals_signature_view.token_at(*idx).as_inner());
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Reference(false, Box::new(signature)));
                self.code
                    .push(Bytecode::BorrowLoc(temp_index, *idx as TempIndex));
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
                        .module_env
                        .globalize_signature(&return_type_view.as_inner());
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    self.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                return_temp_indices.reverse();
                let callee_env = self.module_env.get_called_function(*idx);
                self.code.push(Bytecode::Call(
                    return_temp_indices,
                    callee_env.module_env.get_id(),
                    callee_env.get_id(),
                    vec![],
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
                    .module_env
                    .get_called_function(func_instantiation.handle);
                self.code.push(Bytecode::Call(
                    return_temp_indices,
                    callee_env.module_env.get_id(),
                    callee_env.get_id(),
                    type_sigs,
                    arg_temp_indices,
                ))
            }

            MoveBytecode::Pack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);

                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_definition_view.fields().unwrap() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                self.local_types.push(
                    self.module_env.globalize_signature(&SignatureToken::Struct(
                        struct_definition.struct_handle,
                    )),
                );
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.code.push(Bytecode::Pack(
                    struct_temp_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*idx),
                    vec![],
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::PackGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let struct_definition = self.module.struct_def_at(struct_instantiation.def);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);

                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_definition_view.fields().unwrap() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                self.local_types.push(
                    self.module_env
                        .globalize_signature(&SignatureToken::Struct(
                            struct_definition.struct_handle,
                        ))
                        .replace_struct_instantiation(&actuals),
                );
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.code.push(Bytecode::Pack(
                    struct_temp_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    actuals,
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::Unpack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_definition_view in struct_definition_view.fields().unwrap() {
                    let field_signature_view = field_definition_view.type_signature();
                    let field_type = self
                        .module_env
                        .globalize_signature(&field_signature_view.token().as_inner());
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types.push(field_type);
                    self.temp_count += 1;
                }
                self.code.push(Bytecode::Unpack(
                    field_temp_indices,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*idx),
                    vec![],
                    struct_temp_index,
                ));
            }

            MoveBytecode::UnpackGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let type_sigs = self.get_type_params(struct_instantiation.type_parameters);
                let struct_definition = self.module.struct_def_at(struct_instantiation.def);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_definition_view in struct_definition_view.fields().unwrap() {
                    let field_signature_view = field_definition_view.type_signature();
                    let field_type = self
                        .module_env
                        .globalize_signature(&field_signature_view.token().as_inner())
                        .instantiate(&type_sigs);
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types.push(field_type);
                    self.temp_count += 1;
                }
                self.code.push(Bytecode::Unpack(
                    field_temp_indices,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    type_sigs,
                    struct_temp_index,
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
                self.code.push(Bytecode::ReadRef(temp_index, operand_index));
            }

            MoveBytecode::WriteRef => {
                let ref_operand_index = self.temp_stack.pop().unwrap();
                let val_operand_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(Bytecode::WriteRef(ref_operand_index, val_operand_index));
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
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Add,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Sub => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Sub,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mul => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Mul,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mod => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Mod,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Div => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Div,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitOr => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::BitOr,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitAnd => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::BitAnd,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Xor => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Xor,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shl => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Shl,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shr => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Shr,
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
                self.code.push(Bytecode::Binary(
                    BinaryOp::Or,
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
                self.code.push(Bytecode::Binary(
                    BinaryOp::And,
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
                    .push(Bytecode::Unary(UnaryOp::Not, temp_index, operand_index));
            }
            MoveBytecode::Eq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(Bytecode::Binary(
                    BinaryOp::Eq,
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
                self.code.push(Bytecode::Binary(
                    BinaryOp::Neq,
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
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Lt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Gt => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Gt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Le => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Le,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Ge => {
                        self.code.push(Bytecode::Binary(
                            BinaryOp::Ge,
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
                self.code.push(Bytecode::Exists(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*struct_index),
                    vec![],
                ));
            }

            MoveBytecode::ExistsGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(Bytecode::Exists(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    self.get_type_params(struct_instantiation.type_parameters),
                ));
            }

            MoveBytecode::MutBorrowGlobal(idx) | MoveBytecode::ImmBorrowGlobal(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobal(..));

                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(self.module_env.globalize_signature(&SignatureToken::Struct(
                        struct_definition.struct_handle,
                    ))),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code.push(Bytecode::BorrowGlobal(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*idx),
                    vec![],
                ));
            }

            MoveBytecode::MutBorrowGlobalGeneric(idx)
            | MoveBytecode::ImmBorrowGlobalGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let struct_definition = self.module.struct_def_at(struct_instantiation.def);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobalGeneric(..));

                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                self.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(
                        self.module_env
                            .globalize_signature(&SignatureToken::Struct(
                                struct_definition.struct_handle,
                            ))
                            .instantiate(&actuals),
                    ),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code.push(Bytecode::BorrowGlobal(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    actuals,
                ));
            }

            MoveBytecode::MoveFrom(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(
                    self.module_env.globalize_signature(&SignatureToken::Struct(
                        struct_definition.struct_handle,
                    )),
                );
                self.temp_count += 1;
                self.code.push(Bytecode::MoveFrom(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*idx),
                    vec![],
                ));
            }

            MoveBytecode::MoveFromGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let struct_definition = self.module.struct_def_at(struct_instantiation.def);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                self.local_types.push(
                    self.module_env
                        .globalize_signature(&SignatureToken::Struct(
                            struct_definition.struct_handle,
                        ))
                        .instantiate(&actuals),
                );
                self.temp_count += 1;
                self.code.push(Bytecode::MoveFrom(
                    temp_index,
                    operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    actuals,
                ));
            }

            MoveBytecode::MoveToSender(idx) => {
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::MoveToSender(
                    value_operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(*idx),
                    vec![],
                ));
            }

            MoveBytecode::MoveToSenderGeneric(idx) => {
                let struct_instantiation = self.module.struct_instantiation_at(*idx);
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(Bytecode::MoveToSender(
                    value_operand_index,
                    self.module_env.get_id(),
                    self.module_env.get_struct_id(struct_instantiation.def),
                    self.get_type_params(struct_instantiation.type_parameters),
                ));
            }

            MoveBytecode::GetTxnSenderAddress => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(Type::Primitive(PrimitiveType::Address));
                self.code
                    .push(Bytecode::Load(temp_index, Constant::TxnSenderAddress));
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
            MoveBytecode::Nop => (), // simply discard
        }
    }

    /// Remove MoveLoc, StLoc and CopyLoc from Stackless MoveBytecode
    pub fn simplify_bytecode(code: &[Bytecode]) -> Vec<Bytecode> {
        let mut new_code = vec![];
        let mut new_offsets = BTreeMap::new(); // Used later to decide the new offset to branch to
        let mut equiv_temps = BTreeMap::new(); // Stores temp->local mappings
        let mut new_offset = 0;

        for (i, bytecode) in code.iter().enumerate() {
            match bytecode {
                Bytecode::Assign(dest, src, _) => {
                    equiv_temps.insert(*dest, *src);
                    new_offsets.insert(i as CodeOffset, new_offset); // jump to the next instruction
                }
                _ => {
                    new_offsets.insert(i as CodeOffset, new_offset);
                    new_offset += 1;
                }
            }
        }

        let temp_to_local = |t: &TempIndex| -> TempIndex { *equiv_temps.get(t).unwrap_or(t) };

        for bytecode in code {
            match bytecode {
                BorrowLoc(dest, src) => {
                    new_code.push(BorrowLoc(temp_to_local(dest), temp_to_local(src)));
                }
                ReadRef(dest, src) => {
                    new_code.push(ReadRef(temp_to_local(dest), temp_to_local(src)));
                }
                WriteRef(dest, src) => {
                    new_code.push(WriteRef(temp_to_local(dest), temp_to_local(src)));
                }
                FreezeRef(dest, src) => {
                    new_code.push(FreezeRef(temp_to_local(dest), temp_to_local(src)));
                }
                Call(dest_vec, m, f, l, src_vec) => {
                    let new_dest_vec = dest_vec.iter().map(|t| temp_to_local(t)).collect();
                    let new_src_vec = src_vec.iter().map(|t| temp_to_local(t)).collect();
                    new_code.push(Call(new_dest_vec, *m, *f, l.clone(), new_src_vec));
                }
                Ret(v) => {
                    new_code.push(Ret(v.iter().map(|t| temp_to_local(t)).collect()));
                }
                Pack(dest, m, s, l, src_vec) => {
                    let new_src_vec = src_vec.iter().map(|t| temp_to_local(t)).collect();
                    new_code.push(Pack(temp_to_local(dest), *m, *s, l.clone(), new_src_vec));
                }
                Unpack(dest_vec, m, s, l, src) => {
                    let new_dest_vec = dest_vec.iter().map(|t| temp_to_local(t)).collect();
                    new_code.push(Unpack(new_dest_vec, *m, *s, l.clone(), temp_to_local(src)));
                }
                BorrowField(dest, src, m, s, offset) => {
                    new_code.push(BorrowField(
                        temp_to_local(dest),
                        temp_to_local(src),
                        *m,
                        *s,
                        *offset,
                    ));
                }
                MoveToSender(t, m, s, l) => {
                    new_code.push(MoveToSender(temp_to_local(t), *m, *s, l.clone()));
                }
                MoveFrom(dest, a, m, s, l) => {
                    new_code.push(MoveFrom(
                        temp_to_local(dest),
                        temp_to_local(a),
                        *m,
                        *s,
                        l.clone(),
                    ));
                }
                BorrowGlobal(dest, a, m, s, l) => {
                    new_code.push(BorrowGlobal(
                        temp_to_local(dest),
                        temp_to_local(a),
                        *m,
                        *s,
                        l.clone(),
                    ));
                }
                Exists(dest, a, m, s, l) => {
                    new_code.push(Exists(
                        temp_to_local(dest),
                        temp_to_local(a),
                        *m,
                        *s,
                        l.clone(),
                    ));
                }
                Load(t, c) => {
                    new_code.push(Load(temp_to_local(t), c.clone()));
                }
                Unary(op, dest, src) => {
                    new_code.push(Unary(*op, temp_to_local(dest), temp_to_local(src)));
                }
                Binary(op, dest, src1, src2) => {
                    new_code.push(Binary(
                        *op,
                        temp_to_local(dest),
                        temp_to_local(src1),
                        temp_to_local(src2),
                    ));
                }
                Branch(offset, cond) => {
                    new_code.push(Branch(*new_offsets.get(offset).unwrap(), *cond));
                }
                Abort(t) => {
                    new_code.push(Abort(temp_to_local(t)));
                }
                Destroy(t) => {
                    new_code.push(Destroy(temp_to_local(t)));
                }
                _ => {}
            }
        }
        new_code
    }
}
