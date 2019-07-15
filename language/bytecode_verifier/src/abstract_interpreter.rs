// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract interpretater for verifying type and memory safety on a
//! function body.
use crate::{
    absint::{self, AbstractDomain, TransferFunctions},
    abstract_state::{AbstractState, AbstractValue, JoinResult},
    control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph},
    nonce::Nonce,
};
use mirai_annotations::checked_verify;
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    errors::VMStaticViolation,
    file_format::{
        Bytecode, CompiledModule, FieldDefinitionIndex, FunctionDefinition, LocalIndex,
        SignatureToken, StructHandleIndex,
    },
    views::{
        FieldDefinitionView, FunctionDefinitionView, FunctionSignatureView, LocalsSignatureView,
        SignatureTokenView, StructDefinitionView, ViewInternals,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct StackAbstractValue {
    signature: SignatureToken,
    value: AbstractValue,
}

pub struct AbstractInterpreter<'a> {
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    locals_signature_view: LocalsSignatureView<'a, CompiledModule>,
    cfg: &'a VMControlFlowGraph,
    block_id_to_state: BTreeMap<BlockId, AbstractState>,
    erroneous_blocks: BTreeSet<BlockId>,
    work_list: Vec<BlockId>,
    stack: Vec<StackAbstractValue>,
    next_nonce: usize,
    errors: Vec<VMStaticViolation>,
}

impl<'a> AbstractInterpreter<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Vec<VMStaticViolation> {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let locals_signature_view = function_definition_view.locals_signature();
        let function_signature_view = function_definition_view.signature();
        let mut block_id_to_state = BTreeMap::new();
        let erroneous_blocks = BTreeSet::new();
        let mut locals = BTreeMap::new();
        for (arg_idx, arg_type_view) in function_signature_view.arg_tokens().enumerate() {
            if arg_type_view.is_reference() {
                locals.insert(
                    arg_idx as LocalIndex,
                    AbstractValue::Reference(Nonce::new(arg_idx)),
                );
            } else {
                locals.insert(
                    arg_idx as LocalIndex,
                    AbstractValue::full_value(arg_type_view.is_resource()),
                );
            }
        }
        block_id_to_state.insert(0, AbstractState::new(locals));
        // nonces in [0, locals_signature_view.len()) are reserved for constructing canonical state
        let next_nonce = locals_signature_view.len();
        let mut verifier = Self {
            module,
            function_definition_view,
            locals_signature_view,
            cfg,
            block_id_to_state,
            erroneous_blocks,
            work_list: vec![0],
            stack: vec![],
            next_nonce,
            errors: vec![],
        };

        let mut errors = vec![];
        while !verifier.work_list.is_empty() {
            let block_id = verifier.work_list.pop().unwrap();
            if verifier.erroneous_blocks.contains(&block_id) {
                continue;
            }

            errors.append(&mut verifier.propagate(block_id));
        }
        errors
    }

    fn propagate(&mut self, block_id: BlockId) -> Vec<VMStaticViolation> {
        match self.compute(block_id) {
            Ok(flow_state) => {
                let state = flow_state.construct_canonical_state();
                let mut errors = vec![];
                let block = &self
                    .cfg
                    .block_of_id(block_id)
                    .expect("block_id is not the start offset of a block");
                for next_block_id in &block.successors {
                    if self.erroneous_blocks.contains(next_block_id) {
                        continue;
                    }
                    if !self.block_id_to_state.contains_key(next_block_id) {
                        self.work_list.push(*next_block_id);
                        self.block_id_to_state
                            .entry(*next_block_id)
                            .or_insert_with(|| state.clone());
                    } else {
                        let curr_state = &self.block_id_to_state[next_block_id];
                        let join_result = curr_state.join(&state);
                        match join_result {
                            JoinResult::Unchanged => {}
                            JoinResult::Changed(next_state) => {
                                self.block_id_to_state
                                    .entry(*next_block_id)
                                    .and_modify(|entry| *entry = next_state);
                                self.work_list.push(*next_block_id);
                            }
                            JoinResult::Error => {
                                errors.append(&mut vec![VMStaticViolation::JoinFailure(
                                    *next_block_id as usize,
                                )]);
                            }
                        }
                    }
                }
                errors
            }
            Err(es) => {
                self.erroneous_blocks.insert(block_id);
                es
            }
        }
    }

    fn compute(&mut self, block_id: BlockId) -> Result<AbstractState, Vec<VMStaticViolation>> {
        checked_verify!(self.errors.is_empty());

        let mut state = self.block_id_to_state[&block_id].clone();
        let block = &self.cfg.block_of_id(block_id).unwrap();
        let mut offset = block.entry;
        while offset <= block.exit {
            self.execute(
                &mut state,
                &self.function_definition_view.code().code[offset as usize],
                offset as usize,
            );
            if !self.errors.is_empty() {
                let mut es = vec![];
                es.append(&mut self.errors);
                return Err(es);
            }
            offset += 1;
        }
        Ok(state)
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

    fn is_field_in_struct(
        &self,
        field_definition_index: FieldDefinitionIndex,
        struct_handle_index: StructHandleIndex,
    ) -> bool {
        let field_definition = self.module.field_def_at(field_definition_index);
        struct_handle_index == field_definition.struct_
    }

    fn get_nonce(&mut self, state: &mut AbstractState) -> Nonce {
        let nonce = Nonce::new(self.next_nonce);
        state.add_nonce(nonce.clone());
        self.next_nonce += 1;
        nonce
    }

    fn freeze_ok(&self, state: &AbstractState, existing_borrows: BTreeSet<Nonce>) -> bool {
        for (arg_idx, arg_type_view) in self.locals_signature_view.tokens().enumerate() {
            if arg_type_view.as_inner().is_mutable_reference()
                && state.is_available(arg_idx as LocalIndex)
            {
                if let AbstractValue::Reference(nonce) = state.local(arg_idx as LocalIndex) {
                    if existing_borrows.contains(nonce) {
                        return false;
                    }
                }
            }
        }
        for stack_value in &self.stack {
            if stack_value.signature.is_mutable_reference() {
                if let AbstractValue::Reference(nonce) = &stack_value.value {
                    if existing_borrows.contains(nonce) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn write_borrow_ok(existing_borrows: BTreeSet<Nonce>) -> bool {
        existing_borrows.is_empty()
    }

    fn extract_nonce(value: &AbstractValue) -> Option<&Nonce> {
        match value {
            AbstractValue::Reference(nonce) => Some(nonce),
            AbstractValue::Value(_, _) => None,
        }
    }

    fn is_safe_to_destroy(&self, state: &AbstractState, idx: LocalIndex) -> bool {
        match state.local(idx) {
            AbstractValue::Reference(_) => false,
            AbstractValue::Value(is_resource, borrowed_nonces) => {
                !is_resource && borrowed_nonces.is_empty()
            }
        }
    }
}

impl AbstractDomain for AbstractState {
    fn join(&mut self, _other: &Self) -> absint::JoinResult {
        // TODO: add this once we are using the generic abstract interpreter. only needed to keep
        // the type checker happy for now; this will never be called.
        panic!("Unimplemented")
    }
}

impl<'a> TransferFunctions for AbstractInterpreter<'a> {
    type State = AbstractState;

    fn execute(&mut self, mut state: &mut Self::State, bytecode: &Bytecode, offset: usize) {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                if SignatureTokenView::new(self.module, &operand.signature).is_resource() {
                    self.errors
                        .push(VMStaticViolation::PopResourceError(offset))
                } else if operand.value.is_reference() {
                    self.errors
                        .push(VMStaticViolation::PopReferenceError(offset))
                }
            }

            Bytecode::ReleaseRef => {
                let operand = self.stack.pop().unwrap();
                if let AbstractValue::Reference(nonce) = operand.value {
                    state.destroy_nonce(nonce);
                } else {
                    self.errors
                        .push(VMStaticViolation::ReleaseRefTypeMismatchError(offset))
                }
            }

            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != SignatureToken::Bool {
                    self.errors
                        .push(VMStaticViolation::BrTypeMismatchError(offset))
                }
            }

            Bytecode::StLoc(idx) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != *self.locals_signature_view.token_at(*idx).as_inner() {
                    self.errors
                        .push(VMStaticViolation::StLocTypeMismatchError(offset));
                }
                if state.is_available(*idx) {
                    if self.is_safe_to_destroy(&state, *idx) {
                        state.destroy_local(*idx);
                    } else {
                        self.errors
                            .push(VMStaticViolation::StLocUnsafeToDestroyError(offset));
                    }
                }
                state.insert_local(*idx, operand.value);
            }

            Bytecode::Abort => {
                let error_code = self.stack.pop().unwrap();
                if error_code.signature != SignatureToken::U64 {
                    self.errors
                        .push(VMStaticViolation::AbortTypeMismatchError(offset))
                }
                *state = AbstractState::new(BTreeMap::new())
            }

            Bytecode::Ret => {
                for arg_idx in 0..self.locals_signature_view.len() {
                    let idx = arg_idx as LocalIndex;
                    if state.is_available(idx) && !self.is_safe_to_destroy(&state, idx) {
                        self.errors
                            .push(VMStaticViolation::RetUnsafeToDestroyError(offset));
                    }
                }
                for return_type_view in self
                    .function_definition_view
                    .signature()
                    .return_tokens()
                    .rev()
                {
                    let operand = self.stack.pop().unwrap();
                    if operand.signature != *return_type_view.as_inner() {
                        self.errors
                            .push(VMStaticViolation::RetTypeMismatchError(offset));
                    }
                }
                *state = AbstractState::new(BTreeMap::new())
            }

            Bytecode::Branch(_) => {}

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                    let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                    if self.freeze_ok(&state, borrowed_nonces) {
                        self.stack.push(StackAbstractValue {
                            signature: SignatureToken::Reference(signature),
                            value: operand.value,
                        })
                    } else {
                        self.errors
                            .push(VMStaticViolation::FreezeRefExistsMutableBorrowError(offset))
                    }
                } else {
                    self.errors
                        .push(VMStaticViolation::FreezeRefTypeMismatchError(offset))
                }
            }

            Bytecode::BorrowField(field_definition_index) => {
                let operand = self.stack.pop().unwrap();
                if let Some(struct_handle_index) =
                    SignatureToken::get_struct_handle_from_reference(&operand.signature)
                {
                    if self.is_field_in_struct(*field_definition_index, struct_handle_index) {
                        let field_signature = self.get_field_signature(*field_definition_index);
                        let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                        let nonce = self.get_nonce(&mut state);
                        if operand.signature.is_mutable_reference() {
                            let borrowed_nonces = state.borrowed_nonces_for_field(
                                *field_definition_index,
                                operand_nonce.clone(),
                            );
                            if Self::write_borrow_ok(borrowed_nonces) {
                                self.stack.push(StackAbstractValue {
                                    signature: SignatureToken::MutableReference(Box::new(
                                        field_signature,
                                    )),
                                    value: AbstractValue::Reference(nonce.clone()),
                                });
                                state.borrow_field_from_nonce(
                                    *field_definition_index,
                                    operand_nonce.clone(),
                                    nonce,
                                );
                                state.destroy_nonce(operand_nonce);
                            } else {
                                self.errors.push(
                                    VMStaticViolation::BorrowFieldExistsMutableBorrowError(offset),
                                )
                            }
                        } else {
                            self.stack.push(StackAbstractValue {
                                signature: SignatureToken::Reference(Box::new(field_signature)),
                                value: AbstractValue::Reference(nonce.clone()),
                            });
                            state.borrow_field_from_nonce(
                                *field_definition_index,
                                operand_nonce.clone(),
                                nonce,
                            );
                            state.destroy_nonce(operand_nonce);
                        }
                    } else {
                        self.errors
                            .push(VMStaticViolation::BorrowFieldBadFieldError(offset))
                    }
                } else {
                    self.errors
                        .push(VMStaticViolation::BorrowFieldTypeMismatchError(offset))
                }
            }

            Bytecode::LdConst(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::LdAddr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::LdStr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::String,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::LdByteArray(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    self.errors
                        .push(VMStaticViolation::CopyLocUnavailableError(offset))
                } else if signature_view.is_reference() {
                    let nonce = self.get_nonce(&mut state);
                    state.borrow_from_local_reference(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    })
                } else if signature_view.is_resource() {
                    self.errors
                        .push(VMStaticViolation::CopyLocResourceError(offset))
                } else if state.is_full(state.local(*idx)) {
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::CopyLocExistsBorrowError(offset))
                }
            }

            Bytecode::MoveLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if !state.is_available(*idx) {
                    self.errors
                        .push(VMStaticViolation::MoveLocUnavailableError(offset))
                } else if signature.is_reference() || state.is_full(state.local(*idx)) {
                    let value = state.remove_local(*idx);
                    self.stack.push(StackAbstractValue { signature, value })
                } else {
                    self.errors
                        .push(VMStaticViolation::MoveLocExistsBorrowError(offset))
                }
            }

            Bytecode::BorrowLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if signature.is_reference() {
                    self.errors
                        .push(VMStaticViolation::BorrowLocReferenceError(offset))
                } else if !state.is_available(*idx) {
                    self.errors
                        .push(VMStaticViolation::BorrowLocUnavailableError(offset))
                } else if state.is_full(state.local(*idx)) {
                    let nonce = self.get_nonce(&mut state);
                    state.borrow_from_local_value(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(signature)),
                        value: AbstractValue::Reference(nonce),
                    })
                } else {
                    self.errors
                        .push(VMStaticViolation::BorrowLocExistsBorrowError(offset))
                }
            }

            Bytecode::Call(idx) => {
                let function_handle = self.module.function_handle_at(*idx);
                let function_signature =
                    self.module.function_signature_at(function_handle.signature);
                let function_signature_view =
                    FunctionSignatureView::new(self.module, function_signature);
                let mut all_references_to_borrow_from = BTreeSet::new();
                let mut mutable_references_to_borrow_from = BTreeSet::new();
                for arg_type in function_signature.arg_types.iter().rev() {
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != *arg_type {
                        self.errors
                            .push(VMStaticViolation::CallTypeMismatchError(offset));
                    }
                    if arg_type.is_mutable_reference() && !state.is_full(&arg.value) {
                        self.errors
                            .push(VMStaticViolation::CallBorrowedMutableReferenceError(offset))
                    }
                    if let AbstractValue::Reference(nonce) = arg.value {
                        all_references_to_borrow_from.insert(nonce.clone());
                        if arg_type.is_mutable_reference() {
                            mutable_references_to_borrow_from.insert(nonce.clone());
                        }
                    }
                }
                for return_type_view in function_signature_view.return_tokens() {
                    if return_type_view.is_reference() {
                        let nonce = self.get_nonce(&mut state);
                        if return_type_view.is_mutable_reference() {
                            state.borrow_from_nonces(
                                &mutable_references_to_borrow_from,
                                nonce.clone(),
                            );
                        } else {
                            state.borrow_from_nonces(&all_references_to_borrow_from, nonce.clone());
                        }
                        self.stack.push(StackAbstractValue {
                            signature: return_type_view.as_inner().clone(),
                            value: AbstractValue::Reference(nonce),
                        });
                    } else {
                        self.stack.push(StackAbstractValue {
                            signature: return_type_view.as_inner().clone(),
                            value: AbstractValue::full_value(return_type_view.is_resource()),
                        });
                    }
                }
                for x in all_references_to_borrow_from {
                    state.destroy_nonce(x);
                }
            }

            Bytecode::Pack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                for field_definition_view in struct_definition_view.fields().rev() {
                    let field_signature_view = field_definition_view.type_signature();
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != *field_signature_view.token().as_inner() {
                        self.errors
                            .push(VMStaticViolation::PackTypeMismatchError(offset));
                    }
                }
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Struct(struct_definition.struct_handle),
                    value: AbstractValue::full_value(struct_definition_view.is_resource()),
                });
            }

            Bytecode::Unpack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_arg = self.stack.pop().unwrap();
                if struct_arg.signature != SignatureToken::Struct(struct_definition.struct_handle) {
                    self.errors
                        .push(VMStaticViolation::UnpackTypeMismatchError(offset));
                }
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                for field_definition_view in struct_definition_view.fields() {
                    let field_signature_view = field_definition_view.type_signature();
                    self.stack.push(StackAbstractValue {
                        signature: field_signature_view.token().as_inner().clone(),
                        value: AbstractValue::full_value(field_signature_view.is_resource()),
                    })
                }
            }

            Bytecode::ReadRef => {
                let operand = self.stack.pop().unwrap();
                match operand.signature {
                    SignatureToken::Reference(signature) => {
                        let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            self.errors
                                .push(VMStaticViolation::ReadRefResourceError(offset))
                        } else {
                            self.stack.push(StackAbstractValue {
                                signature: *signature,
                                value: AbstractValue::full_value(false),
                            });
                            state.destroy_nonce(operand_nonce)
                        }
                    }
                    SignatureToken::MutableReference(signature) => {
                        let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            self.errors
                                .push(VMStaticViolation::ReadRefResourceError(offset))
                        } else {
                            let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                            if self.freeze_ok(&state, borrowed_nonces) {
                                self.stack.push(StackAbstractValue {
                                    signature: *signature,
                                    value: AbstractValue::full_value(false),
                                });
                                state.destroy_nonce(operand_nonce)
                            } else {
                                self.errors.push(
                                    VMStaticViolation::ReadRefExistsMutableBorrowError(offset),
                                )
                            }
                        }
                    }
                    _ => self
                        .errors
                        .push(VMStaticViolation::ReadRefTypeMismatchError(offset)),
                }
            }

            Bytecode::WriteRef => {
                let ref_operand = self.stack.pop().unwrap();
                let val_operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = ref_operand.signature {
                    if SignatureTokenView::new(self.module, &signature).is_resource() {
                        self.errors
                            .push(VMStaticViolation::WriteRefResourceError(offset))
                    } else if val_operand.signature != *signature {
                        self.errors
                            .push(VMStaticViolation::WriteRefTypeMismatchError(offset))
                    } else if state.is_full(&ref_operand.value) {
                        let ref_operand_nonce =
                            Self::extract_nonce(&ref_operand.value).unwrap().clone();
                        state.destroy_nonce(ref_operand_nonce)
                    } else {
                        self.errors
                            .push(VMStaticViolation::WriteRefExistsBorrowError(offset))
                    }
                } else {
                    self.errors
                        .push(VMStaticViolation::WriteRefNoMutableReferenceError(offset))
                }
            }

            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature == SignatureToken::U64
                    && operand2.signature == SignatureToken::U64
                {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::U64,
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::IntegerOpTypeMismatchError(offset))
                }
            }

            Bytecode::Or | Bytecode::And => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature == SignatureToken::Bool
                    && operand2.signature == SignatureToken::Bool
                {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::BooleanOpTypeMismatchError(offset))
                }
            }

            Bytecode::Not => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Bool {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::BooleanOpTypeMismatchError(offset))
                }
            }

            Bytecode::Eq | Bytecode::Neq => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature.allows_equality() && operand1.signature == operand2.signature
                {
                    if let AbstractValue::Reference(nonce) = operand1.value {
                        state.destroy_nonce(nonce);
                    }
                    if let AbstractValue::Reference(nonce) = operand2.value {
                        state.destroy_nonce(nonce);
                    }
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    })
                } else {
                    self.errors
                        .push(VMStaticViolation::EqualityOpTypeMismatchError(offset))
                }
            }

            Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature == SignatureToken::U64
                    && operand2.signature == SignatureToken::U64
                {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::IntegerOpTypeMismatchError(offset))
                }
            }

            Bytecode::Exists(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::ExistsResourceTypeMismatchError(offset))
                }
            }

            Bytecode::BorrowGlobal(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    self.errors
                        .push(VMStaticViolation::BorrowGlobalNoResourceError(offset));
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    let nonce = self.get_nonce(&mut state);
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(
                            SignatureToken::Struct(struct_definition.struct_handle),
                        )),
                        value: AbstractValue::Reference(nonce),
                    })
                } else {
                    self.errors
                        .push(VMStaticViolation::BorrowGlobalTypeMismatchError(offset))
                }
            }

            Bytecode::MoveFrom(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    self.errors
                        .push(VMStaticViolation::MoveFromNoResourceError(offset));
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Struct(struct_definition.struct_handle),
                        value: AbstractValue::full_value(true),
                    });
                } else {
                    self.errors
                        .push(VMStaticViolation::MoveFromTypeMismatchError(offset))
                }
            }

            Bytecode::MoveToSender(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    self.errors
                        .push(VMStaticViolation::MoveToSenderNoResourceError(offset));
                }

                let value_operand = self.stack.pop().unwrap();
                if value_operand.signature
                    == SignatureToken::Struct(struct_definition.struct_handle)
                {

                } else {
                    self.errors
                        .push(VMStaticViolation::MoveToSenderTypeMismatchError(offset))
                }
            }

            Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnSequenceNumber => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::GetTxnPublicKey => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
                });
            }

            Bytecode::CreateAccount => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {

                } else {
                    self.errors
                        .push(VMStaticViolation::CreateAccountTypeMismatchError(offset))
                }
            }

            Bytecode::EmitEvent => {
                // TODO: EmitEvent is currently unimplemented
                //       following is a workaround to skip the check
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
            }
        }
    }
}
