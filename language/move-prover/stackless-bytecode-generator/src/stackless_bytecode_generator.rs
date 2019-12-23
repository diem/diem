// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stackless_bytecode::StacklessBytecode;
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode, CompiledModule, CompiledProgram, FieldDefinitionIndex, FunctionDefinition,
        LocalsSignatureIndex, SignatureToken,
    },
    views::{
        FieldDefinitionView, FunctionDefinitionView, FunctionSignatureView, StructDefinitionView,
        ViewInternals,
    },
};

pub struct StacklessFunction {
    pub local_types: Vec<SignatureToken>,
    pub code: Vec<StacklessBytecode>,
}

pub struct StacklessProgram {
    pub module_functions: Vec<Vec<StacklessFunction>>,
    pub compiled_modules: Vec<CompiledModule>,
}

struct StacklessBytecodeGenerator<'a> {
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    temp_count: usize,
    temp_stack: Vec<usize>,
    local_types: Vec<SignatureToken>,
    code: Vec<StacklessBytecode>,
}

pub struct StacklessProgramGenerator {
    program: CompiledProgram,
}

impl StacklessProgramGenerator {
    pub fn new(program: CompiledProgram) -> Self {
        StacklessProgramGenerator { program }
    }

    pub fn generate_program(self) -> StacklessProgram {
        let script_module = self.program.script.into_module();
        let mut compiled_modules = self.program.modules;
        compiled_modules.push(script_module);
        let module_functions = compiled_modules
            .iter()
            .map(move |module| StacklessModuleGenerator::new(&module).generate_module())
            .collect();
        StacklessProgram {
            module_functions,
            compiled_modules,
        }
    }
}

pub struct StacklessModuleGenerator<'a> {
    module: &'a CompiledModule,
}

impl<'a> StacklessModuleGenerator<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        StacklessModuleGenerator { module }
    }

    pub fn generate_module(self) -> Vec<StacklessFunction> {
        self.module
            .function_defs()
            .iter()
            .map(move |function_definition| {
                StacklessBytecodeGenerator::new(self.module, function_definition)
                    .generate_function()
            })
            .collect()
    }
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn new(module: &'a CompiledModule, function_definition: &'a FunctionDefinition) -> Self {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let mut temp_count = 0;
        let mut local_types = vec![];
        if !function_definition_view.is_native() {
            let locals_signature_view = function_definition_view.locals_signature();
            temp_count = locals_signature_view.len();
            for (_, arg_type_view) in locals_signature_view.tokens().enumerate() {
                local_types.push(arg_type_view.as_inner().clone());
            }
        }
        StacklessBytecodeGenerator {
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

    fn get_field_signature(&self, field_definition_index: FieldDefinitionIndex) -> SignatureToken {
        let field_definition = self.module.field_def_at(field_definition_index);
        let field_definition_view = FieldDefinitionView::new(self.module, field_definition);
        field_definition_view
            .type_signature()
            .token()
            .as_inner()
            .clone()
    }

    fn get_type_params(&self, type_params_index: LocalsSignatureIndex) -> Vec<SignatureToken> {
        self.module.locals_signature_at(type_params_index).0.clone()
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn generate_bytecode(&mut self, bytecode: &Bytecode) {
        match bytecode {
            Bytecode::Pop => {
                self.temp_stack.pop();
                self.code.push(StacklessBytecode::NoOp);
            }
            Bytecode::BrTrue(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(StacklessBytecode::BrTrue(*code_offset, temp_index));
            }

            Bytecode::BrFalse(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(StacklessBytecode::BrFalse(*code_offset, temp_index));
            }

            Bytecode::Abort => {
                let error_code_index = self.temp_stack.pop().unwrap();
                self.code.push(StacklessBytecode::Abort(error_code_index));
            }

            Bytecode::StLoc(idx) => {
                let operand_index = self.temp_stack.pop().unwrap();
                self.code
                    .push(StacklessBytecode::StLoc(*idx, operand_index));
            }

            Bytecode::Ret => {
                let mut return_temps = vec![];
                for _ in self.function_definition_view.signature().return_tokens() {
                    let return_temp_index = self.temp_stack.pop().unwrap();
                    return_temps.push(return_temp_index);
                }
                return_temps.reverse();
                self.code.push(StacklessBytecode::Ret(return_temps));
            }

            Bytecode::Branch(code_offset) => {
                self.code.push(StacklessBytecode::Branch(*code_offset));
            }

            Bytecode::FreezeRef => {
                let mutable_ref_index = self.temp_stack.pop().unwrap();
                let mutable_ref_sig = self.local_types[mutable_ref_index].clone();
                if let SignatureToken::MutableReference(signature) = mutable_ref_sig {
                    let immutable_ref_index = self.temp_count;
                    self.temp_stack.push(immutable_ref_index);
                    self.local_types.push(SignatureToken::Reference(signature));
                    self.code.push(StacklessBytecode::FreezeRef(
                        immutable_ref_index,
                        mutable_ref_index,
                    ));
                    self.temp_count += 1;
                }
            }

            Bytecode::MutBorrowField(field_definition_index) => {
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let field_signature = self.get_field_signature(*field_definition_index);
                let parent_type = self.local_types[struct_ref_index].clone();
                let type_sigs = match parent_type {
                    SignatureToken::MutableReference(b) => match *b {
                        SignatureToken::Struct(_, v) => v,
                        _ => panic!("not a struct in BorrowField"),
                    },
                    _ => panic!("not a reference in BorrowField"),
                };
                let field_type = match field_signature {
                    SignatureToken::TypeParameter(i) => type_sigs[i as usize].clone(),
                    _ => field_signature,
                };
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(StacklessBytecode::BorrowField(
                    field_ref_index,
                    struct_ref_index,
                    *field_definition_index,
                ));
                self.temp_count += 1;
                self.local_types
                    .push(SignatureToken::MutableReference(Box::new(field_type)));
            }

            Bytecode::ImmBorrowField(field_definition_index) => {
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let field_signature = self.get_field_signature(*field_definition_index);
                let parent_type = self.local_types[struct_ref_index].clone();
                let type_sigs = match parent_type {
                    SignatureToken::Reference(b) | SignatureToken::MutableReference(b) => {
                        match *b {
                            SignatureToken::Struct(_, v) => v,
                            _ => panic!("not a struct in BorrowField"),
                        }
                    }
                    _ => {
                        println!("{:?},{:?}", bytecode, parent_type);
                        panic!("not a reference in BorrowField")
                    }
                };
                let field_type = match field_signature {
                    SignatureToken::TypeParameter(i) => type_sigs[i as usize].clone(),
                    _ => field_signature,
                };
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.code.push(StacklessBytecode::BorrowField(
                    field_ref_index,
                    struct_ref_index,
                    *field_definition_index,
                ));
                self.temp_count += 1;
                self.local_types
                    .push(SignatureToken::Reference(Box::new(field_type)));
            }

            Bytecode::LdU8(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U8);
                self.code.push(StacklessBytecode::LdU8(temp_index, *number));
                self.temp_count += 1;
            }

            Bytecode::LdU64(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.code
                    .push(StacklessBytecode::LdU64(temp_index, *number));
                self.temp_count += 1;
            }

            Bytecode::LdU128(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U128);
                self.code
                    .push(StacklessBytecode::LdU128(temp_index, *number));
                self.temp_count += 1;
            }

            Bytecode::CastU8 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U8);
                self.code
                    .push(StacklessBytecode::CastU8(temp_index, operand_index));
                self.temp_count += 1;
            }

            Bytecode::CastU64 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U8);
                self.code
                    .push(StacklessBytecode::CastU64(temp_index, operand_index));
                self.temp_count += 1;
            }

            Bytecode::CastU128 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U8);
                self.code
                    .push(StacklessBytecode::CastU128(temp_index, operand_index));
                self.temp_count += 1;
            }

            Bytecode::LdAddr(address_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Address);
                self.code
                    .push(StacklessBytecode::LdAddr(temp_index, *address_pool_index));
                self.temp_count += 1;
            }

            Bytecode::LdByteArray(byte_array_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::ByteArray);
                self.code.push(StacklessBytecode::LdByteArray(
                    temp_index,
                    *byte_array_pool_index,
                ));
                self.temp_count += 1;
            }

            Bytecode::LdTrue => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Bool);
                self.code.push(StacklessBytecode::LdTrue(temp_index));
                self.temp_count += 1;
            }

            Bytecode::LdFalse => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Bool);
                self.code.push(StacklessBytecode::LdFalse(temp_index));
                self.temp_count += 1;
            }

            Bytecode::CopyLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(StacklessBytecode::CopyLoc(temp_index, *idx));
                self.temp_count += 1;
            }

            Bytecode::MoveLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.code.push(StacklessBytecode::MoveLoc(temp_index, *idx));
                self.temp_count += 1;
            }

            Bytecode::MutBorrowLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(SignatureToken::MutableReference(Box::new(signature)));
                self.code
                    .push(StacklessBytecode::BorrowLoc(temp_index, *idx));
                self.temp_count += 1;
            }

            Bytecode::ImmBorrowLoc(idx) => {
                let locals_signature_view = self.function_definition_view.locals_signature();
                let signature = locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(SignatureToken::Reference(Box::new(signature)));
                self.code
                    .push(StacklessBytecode::BorrowLoc(temp_index, *idx));
                self.temp_count += 1;
            }

            Bytecode::Call(idx, type_params) => {
                let type_sigs = self.get_type_params(*type_params);
                let function_handle = self.module.function_handle_at(*idx);
                let function_signature =
                    self.module.function_signature_at(function_handle.signature);
                let function_signature_view =
                    FunctionSignatureView::new(self.module, function_signature);

                let mut arg_temp_indices = vec![];
                let mut return_temp_indices = vec![];
                for _ in function_signature.arg_types.iter() {
                    let arg_temp_index = self.temp_stack.pop().unwrap();
                    arg_temp_indices.push(arg_temp_index);
                }
                for return_type_view in function_signature_view.return_tokens() {
                    let return_temp_index = self.temp_count;
                    // instantiate type parameters
                    let return_type =
                        self.instantiate_type_params(return_type_view.as_inner(), &type_sigs);
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    self.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                return_temp_indices.reverse();
                self.code.push(StacklessBytecode::Call(
                    return_temp_indices,
                    *idx,
                    *type_params,
                    arg_temp_indices,
                ))
            }

            Bytecode::Pack(idx, type_params) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);

                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_definition_view.fields().unwrap() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                self.local_types.push(SignatureToken::Struct(
                    struct_definition.struct_handle,
                    self.get_type_params(*type_params),
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.code.push(StacklessBytecode::Pack(
                    struct_temp_index,
                    *idx,
                    *type_params,
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            Bytecode::Unpack(idx, type_params) => {
                let type_sigs = self.get_type_params(*type_params);
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_definition_view in struct_definition_view.fields().unwrap() {
                    let field_signature_view = field_definition_view.type_signature();
                    let field_type = match field_signature_view.token().as_inner() {
                        SignatureToken::TypeParameter(i) => type_sigs[*i as usize].clone(),
                        _ => field_signature_view.token().as_inner().clone(),
                    };
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types.push(field_type);
                    self.temp_count += 1;
                }
                self.code.push(StacklessBytecode::Unpack(
                    field_temp_indices,
                    *idx,
                    *type_params,
                    struct_temp_index,
                ));
            }
            Bytecode::ReadRef => {
                let operand_index = self.temp_stack.pop().unwrap();
                let operand_sig = self.local_types[operand_index].clone();
                let temp_index = self.temp_count;
                match operand_sig {
                    SignatureToken::Reference(signature)
                    | SignatureToken::MutableReference(signature) => {
                        self.local_types.push(*signature);
                    }
                    _ => {}
                }
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code
                    .push(StacklessBytecode::ReadRef(temp_index, operand_index));
            }

            Bytecode::WriteRef => {
                let ref_operand_index = self.temp_stack.pop().unwrap();
                let val_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(StacklessBytecode::WriteRef(
                    ref_operand_index,
                    val_operand_index,
                ));
            }

            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor
            | Bytecode::Shl
            | Bytecode::Shr => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let operand_type = self.local_types[operand1_index].clone();
                let temp_index = self.temp_count;
                self.local_types.push(operand_type);
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                match bytecode {
                    Bytecode::Add => {
                        self.code.push(StacklessBytecode::Add(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Sub => {
                        self.code.push(StacklessBytecode::Sub(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Mul => {
                        self.code.push(StacklessBytecode::Mul(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Mod => {
                        self.code.push(StacklessBytecode::Mod(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Div => {
                        self.code.push(StacklessBytecode::Div(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::BitOr => {
                        self.code.push(StacklessBytecode::BitOr(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::BitAnd => {
                        self.code.push(StacklessBytecode::BitAnd(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Xor => {
                        self.code.push(StacklessBytecode::Xor(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Shl => {
                        self.code.push(StacklessBytecode::Shl(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Shr => {
                        self.code.push(StacklessBytecode::Shr(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            Bytecode::Or => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(StacklessBytecode::Or(
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            Bytecode::And => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(StacklessBytecode::And(
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            Bytecode::Not => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code
                    .push(StacklessBytecode::Not(temp_index, operand_index));
            }
            Bytecode::Eq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(StacklessBytecode::Eq(
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            Bytecode::Neq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(StacklessBytecode::Neq(
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                match bytecode {
                    Bytecode::Lt => {
                        self.code.push(StacklessBytecode::Lt(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Gt => {
                        self.code.push(StacklessBytecode::Gt(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Le => {
                        self.code.push(StacklessBytecode::Le(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    Bytecode::Ge => {
                        self.code.push(StacklessBytecode::Ge(
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            Bytecode::Exists(struct_index, type_params) => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.code.push(StacklessBytecode::Exists(
                    temp_index,
                    operand_index,
                    *struct_index,
                    *type_params,
                ));
            }
            Bytecode::MutBorrowGlobal(idx, type_params)
            | Bytecode::ImmBorrowGlobal(idx, type_params) => {
                let struct_definition = self.module.struct_def_at(*idx);

                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types
                    .push(SignatureToken::MutableReference(Box::new(
                        SignatureToken::Struct(
                            struct_definition.struct_handle,
                            self.get_type_params(*type_params),
                        ),
                    )));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.code.push(StacklessBytecode::BorrowGlobal(
                    temp_index,
                    operand_index,
                    *idx,
                    *type_params,
                ));
            }
            Bytecode::MoveFrom(idx, type_params) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Struct(
                    struct_definition.struct_handle,
                    self.get_type_params(*type_params),
                ));
                self.temp_count += 1;
                self.code.push(StacklessBytecode::MoveFrom(
                    temp_index,
                    operand_index,
                    *idx,
                    *type_params,
                ));
            }
            Bytecode::MoveToSender(idx, type_params) => {
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.code.push(StacklessBytecode::MoveToSender(
                    value_operand_index,
                    *idx,
                    *type_params,
                ));
            }
            Bytecode::GetTxnGasUnitPrice => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.code
                    .push(StacklessBytecode::GetTxnGasUnitPrice(temp_index));
                self.temp_count += 1;
            }
            Bytecode::GetTxnMaxGasUnits => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.code
                    .push(StacklessBytecode::GetTxnMaxGasUnits(temp_index));
                self.temp_count += 1;
            }
            Bytecode::GetGasRemaining => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.code
                    .push(StacklessBytecode::GetGasRemaining(temp_index));
                self.temp_count += 1;
            }
            Bytecode::GetTxnSequenceNumber => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.code
                    .push(StacklessBytecode::GetTxnSequenceNumber(temp_index));
                self.temp_count += 1;
            }

            Bytecode::GetTxnSenderAddress => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Address);
                self.code
                    .push(StacklessBytecode::GetTxnSenderAddress(temp_index));
                self.temp_count += 1;
            }

            Bytecode::GetTxnPublicKey => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::ByteArray);
                self.code
                    .push(StacklessBytecode::GetTxnPublicKey(temp_index));
                self.temp_count += 1;
            }
        }
    }

    fn instantiate_type_params(
        &self,
        sig: &SignatureToken,
        actuals: &[SignatureToken],
    ) -> SignatureToken {
        match sig {
            SignatureToken::TypeParameter(i) => actuals[*i as usize].clone(),
            SignatureToken::Reference(b) => {
                SignatureToken::Reference(Box::new(self.instantiate_type_params(&**b, actuals)))
            }
            SignatureToken::MutableReference(b) => SignatureToken::MutableReference(Box::new(
                self.instantiate_type_params(&**b, actuals),
            )),
            SignatureToken::Struct(handle_index, type_args) => SignatureToken::Struct(
                *handle_index,
                type_args
                    .iter()
                    .map(|a| self.instantiate_type_params(a, actuals))
                    .collect(),
            ),
            _ => sig.clone(),
        }
    }
}
