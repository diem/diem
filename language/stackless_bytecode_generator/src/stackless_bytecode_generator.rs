use crate::stackless_bytecode::StacklessBytecode;
use bytecode_verifier::control_flow_graph::{
    BasicBlock, BlockId, ControlFlowGraph, VMControlFlowGraph,
};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode, CodeOffset, CompiledModule, FieldDefinitionIndex, FunctionDefinition,
        SignatureToken,
    },
    views::{
        FieldDefinitionView, FunctionDefinitionView, FunctionSignatureView, LocalsSignatureView,
        StructDefinitionView, ViewInternals,
    },
};

pub struct StacklessFunction {
    pub local_types: Vec<SignatureToken>,
    pub code: Vec<StacklessBytecode>,
}

struct StacklessBytecodeGenerator<'a> {
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    locals_signature_view: LocalsSignatureView<'a, CompiledModule>,
    cfg: &'a VMControlFlowGraph,
    temp_count: usize,
    temp_stack: Vec<usize>,
    visited_blocks: BTreeSet<BlockId>,
    work_list: Vec<BlockId>,
    offset_mapping: BTreeMap<CodeOffset, CodeOffset>,
    local_types: Vec<SignatureToken>,
    code: Vec<StacklessBytecode>,
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
                let cfg = VMControlFlowGraph::new(&function_definition.code.code);
                StacklessBytecodeGenerator::new(self.module, function_definition, &cfg)
                    .generate_function()
            })
            .collect()
    }
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn new(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Self {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let locals_signature_view = function_definition_view.locals_signature();
        let mut local_types = vec![];
        for (_, arg_type_view) in locals_signature_view.tokens().enumerate() {
            local_types.push(arg_type_view.as_inner().clone());
        }
        StacklessBytecodeGenerator {
            module,
            function_definition_view,
            cfg: &cfg,
            temp_count: locals_signature_view.len(),
            temp_stack: vec![],
            visited_blocks: BTreeSet::new(),
            work_list: vec![],
            offset_mapping: BTreeMap::new(),
            local_types,
            locals_signature_view,
            code: vec![],
        }
    }

    pub fn generate_function(mut self) -> StacklessFunction {
        self.work_list = vec![0];
        self.visited_blocks.insert(0);
        while !self.work_list.is_empty() {
            let block_id = self.work_list.pop().unwrap();
            let block = &self.cfg.block_of_id(block_id).unwrap();
            self.compute(block);
            for next_block_id in block.successors.iter().rev() {
                if !self.visited_blocks.contains(next_block_id) {
                    self.work_list.push(*next_block_id);
                    self.visited_blocks.insert(*next_block_id);
                }
            }
        }

        // map original code offset to new offset
        let code = self
            .code
            .clone()
            .into_iter()
            .map(|code| match code {
                StacklessBytecode::Branch(o) => {
                    StacklessBytecode::Branch(*self.offset_mapping.get(&o).unwrap())
                }
                StacklessBytecode::BrTrue(o, t) => {
                    StacklessBytecode::BrTrue(*self.offset_mapping.get(&o).unwrap(), t)
                }
                StacklessBytecode::BrFalse(o, t) => {
                    StacklessBytecode::BrFalse(*self.offset_mapping.get(&o).unwrap(), t)
                }
                _ => code,
            })
            .collect();

        StacklessFunction {
            code,
            local_types: self.local_types,
        }
    }

    fn compute(&mut self, block: &BasicBlock) {
        let mut offset = block.entry;
        while offset <= block.exit {
            self.generate_bytecode(
                &self.function_definition_view.code().code[offset as usize],
                offset as usize,
            );
            offset += 1;
        }
    }

    fn insert_stackless_bytecode(&mut self, bytecode: StacklessBytecode, original_offset: usize) {
        let new_offset = self.code.len();
        self.code.push(bytecode);
        self.offset_mapping
            .insert(original_offset as CodeOffset, new_offset as CodeOffset);
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

    #[allow(clippy::cognitive_complexity)]
    pub fn generate_bytecode(&mut self, bytecode: &Bytecode, offset: usize) {
        match bytecode {
            Bytecode::Pop => {
                self.temp_stack.pop();
            }

            Bytecode::ReleaseRef => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(StacklessBytecode::ReleaseRef(temp_index), offset);
            }

            Bytecode::BrTrue(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::BrTrue(*code_offset, temp_index),
                    offset,
                );
            }

            Bytecode::BrFalse(code_offset) => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::BrFalse(*code_offset, temp_index),
                    offset,
                );
            }

            Bytecode::Abort => {
                let error_code_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(StacklessBytecode::Abort(error_code_index), offset);
            }

            Bytecode::StLoc(idx) => {
                let operand_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::StLoc(*idx, operand_index),
                    offset,
                );
            }

            Bytecode::Ret => {
                let mut return_temps = vec![];
                for _ in self.function_definition_view.signature().return_tokens() {
                    let return_temp_index = self.temp_stack.pop().unwrap();
                    return_temps.push(return_temp_index);
                }
                return_temps.reverse();
                self.insert_stackless_bytecode(StacklessBytecode::Ret(return_temps), offset);
            }

            Bytecode::Branch(code_offset) => {
                self.insert_stackless_bytecode(StacklessBytecode::Branch(*code_offset), offset);
            }

            Bytecode::FreezeRef => {
                let mutable_ref_index = self.temp_stack.pop().unwrap();
                let mutable_ref_sig = self.local_types[mutable_ref_index].clone();
                if let SignatureToken::MutableReference(signature) = mutable_ref_sig {
                    let immutable_ref_index = self.temp_count;
                    self.temp_stack.push(immutable_ref_index);
                    self.local_types.push(SignatureToken::Reference(signature));
                    self.insert_stackless_bytecode(
                        StacklessBytecode::FreezeRef(immutable_ref_index, mutable_ref_index),
                        offset,
                    );
                    self.temp_count += 1;
                }
            }

            Bytecode::BorrowField(field_definition_index) => {
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let struct_ref_sig = self.local_types[struct_ref_index].clone();
                let field_signature = self.get_field_signature(*field_definition_index);

                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);

                self.insert_stackless_bytecode(
                    StacklessBytecode::BorrowField(
                        field_ref_index,
                        struct_ref_index,
                        *field_definition_index,
                    ),
                    offset,
                );
                self.temp_count += 1;
                if struct_ref_sig.is_mutable_reference() {
                    self.local_types
                        .push(SignatureToken::MutableReference(Box::new(field_signature)));
                } else {
                    self.local_types
                        .push(SignatureToken::Reference(Box::new(field_signature)));
                }
            }

            Bytecode::LdConst(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.insert_stackless_bytecode(
                    StacklessBytecode::LdConst(temp_index, *number),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::LdAddr(address_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Address);
                self.insert_stackless_bytecode(
                    StacklessBytecode::LdAddr(temp_index, *address_pool_index),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::LdStr(string_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::String);
                self.insert_stackless_bytecode(
                    StacklessBytecode::LdStr(temp_index, *string_pool_index),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::LdByteArray(byte_array_pool_index) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::ByteArray);
                self.insert_stackless_bytecode(
                    StacklessBytecode::LdByteArray(temp_index, *byte_array_pool_index),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::LdTrue => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Bool);
                self.insert_stackless_bytecode(StacklessBytecode::LdTrue(temp_index), offset);
                self.temp_count += 1;
            }

            Bytecode::LdFalse => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Bool);
                self.insert_stackless_bytecode(StacklessBytecode::LdFalse(temp_index), offset);
                self.temp_count += 1;
            }

            Bytecode::CopyLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.insert_stackless_bytecode(
                    StacklessBytecode::CopyLoc(temp_index, *idx),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::MoveLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(signature); // same type as the value copied
                self.insert_stackless_bytecode(
                    StacklessBytecode::MoveLoc(temp_index, *idx),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::BorrowLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types
                    .push(SignatureToken::MutableReference(Box::new(signature)));
                self.insert_stackless_bytecode(
                    StacklessBytecode::BorrowLoc(temp_index, *idx),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::Call(idx, _) => {
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
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    self.local_types.push(return_type_view.as_inner().clone());
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                return_temp_indices.reverse();
                self.insert_stackless_bytecode(
                    StacklessBytecode::Call(return_temp_indices, *idx, arg_temp_indices),
                    offset,
                )
            }

            Bytecode::Pack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);

                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;
                for _ in struct_definition_view.fields() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                self.local_types.push(SignatureToken::Struct(
                    struct_definition.struct_handle,
                    vec![],
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                self.insert_stackless_bytecode(
                    StacklessBytecode::Pack(struct_temp_index, *idx, field_temp_indices),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::Unpack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_definition_view in struct_definition_view.fields() {
                    let field_signature_view = field_definition_view.type_signature();
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    self.local_types
                        .push(field_signature_view.token().as_inner().clone());
                    self.temp_count += 1;
                }
                self.insert_stackless_bytecode(
                    StacklessBytecode::Unpack(field_temp_indices, *idx, struct_temp_index),
                    offset,
                );
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
                self.insert_stackless_bytecode(
                    StacklessBytecode::ReadRef(temp_index, operand_index),
                    offset,
                );
            }

            Bytecode::WriteRef => {
                let ref_operand_index = self.temp_stack.pop().unwrap();
                let val_operand_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::WriteRef(ref_operand_index, val_operand_index),
                    offset,
                );
            }

            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::U64);
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                match bytecode {
                    Bytecode::Add => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Add(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Sub => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Sub(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Mul => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Mul(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Mod => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Mod(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Div => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Div(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::BitOr => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::BitOr(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::BitAnd => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::BitAnd(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Xor => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Xor(temp_index, operand1_index, operand2_index),
                            offset,
                        );
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
                self.insert_stackless_bytecode(
                    StacklessBytecode::Or(temp_index, operand1_index, operand2_index),
                    offset,
                );
            }

            Bytecode::And => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.insert_stackless_bytecode(
                    StacklessBytecode::And(temp_index, operand1_index, operand2_index),
                    offset,
                );
            }

            Bytecode::Not => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.insert_stackless_bytecode(
                    StacklessBytecode::Not(temp_index, operand_index),
                    offset,
                );
            }
            Bytecode::Eq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.insert_stackless_bytecode(
                    StacklessBytecode::Eq(temp_index, operand1_index, operand2_index),
                    offset,
                );
            }
            Bytecode::Neq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.insert_stackless_bytecode(
                    StacklessBytecode::Neq(temp_index, operand1_index, operand2_index),
                    offset,
                );
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
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Lt(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Gt => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Gt(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Le => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Le(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    Bytecode::Ge => {
                        self.insert_stackless_bytecode(
                            StacklessBytecode::Ge(temp_index, operand1_index, operand2_index),
                            offset,
                        );
                    }
                    _ => {}
                }
            }
            Bytecode::Exists(struct_index, _) => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types.push(SignatureToken::Bool);
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                self.insert_stackless_bytecode(
                    StacklessBytecode::Exists(temp_index, operand_index, *struct_index),
                    offset,
                );
            }
            Bytecode::BorrowGlobal(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);

                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.local_types
                    .push(SignatureToken::MutableReference(Box::new(
                        SignatureToken::Struct(struct_definition.struct_handle, vec![]),
                    )));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                self.insert_stackless_bytecode(
                    StacklessBytecode::BorrowGlobal(temp_index, operand_index, *idx),
                    offset,
                );
            }
            Bytecode::MoveFrom(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Struct(
                    struct_definition.struct_handle,
                    vec![],
                ));
                self.temp_count += 1;
                self.insert_stackless_bytecode(
                    StacklessBytecode::MoveFrom(temp_index, operand_index, *idx),
                    offset,
                );
            }
            Bytecode::MoveToSender(idx, _) => {
                let value_operand_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::MoveToSender(value_operand_index, *idx),
                    offset,
                );
            }
            Bytecode::GetTxnGasUnitPrice => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetTxnGasUnitPrice(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }
            Bytecode::GetTxnMaxGasUnits => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetTxnMaxGasUnits(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }
            Bytecode::GetGasRemaining => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetGasRemaining(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }
            Bytecode::GetTxnSequenceNumber => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::U64);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetTxnSequenceNumber(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::GetTxnSenderAddress => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::Address);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetTxnSenderAddress(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }

            Bytecode::GetTxnPublicKey => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                self.local_types.push(SignatureToken::ByteArray);
                self.insert_stackless_bytecode(
                    StacklessBytecode::GetTxnPublicKey(temp_index),
                    offset,
                );
                self.temp_count += 1;
            }
            Bytecode::CreateAccount => {
                let temp_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::CreateAccount(temp_index),
                    offset,
                );
            }

            Bytecode::EmitEvent => {
                // TODO: EmitEvent is currently unimplemented
                let operand3_index = self.temp_stack.pop().unwrap();
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                self.insert_stackless_bytecode(
                    StacklessBytecode::EmitEvent(operand1_index, operand2_index, operand3_index),
                    offset,
                );
            }
        }
    }
}
