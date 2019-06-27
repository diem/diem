// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract interpretater for verifying type and memory safety on a
//! function body.
use crate::{
    abstract_state::{AbstractState, AbstractValue, JoinResult},
    code_unit_verifier::VerificationPass,
    control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph},
    nonce::Nonce,
};
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
}

impl<'a> VerificationPass<'a> for AbstractInterpreter<'a> {
    fn new(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Self {
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
        let next_nonce = function_signature_view.arg_count();
        Self {
            module,
            function_definition_view,
            locals_signature_view,
            cfg,
            block_id_to_state,
            erroneous_blocks,
            work_list: vec![0],
            stack: vec![],
            next_nonce,
        }
    }

    fn verify(mut self) -> Vec<VMStaticViolation> {
        let mut errors = vec![];
        while !self.work_list.is_empty() {
            let block_id = self.work_list.pop().unwrap();
            errors.append(&mut self.propagate(block_id));
        }
        errors
    }
}

impl<'a> AbstractInterpreter<'a> {
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
        let mut state = self.block_id_to_state[&block_id].clone();
        let block = &self.cfg.block_of_id(block_id).unwrap();
        let mut offset = block.entry;
        while offset <= block.exit {
            let result = self.next(
                state,
                offset as usize,
                &self.function_definition_view.code().code[offset as usize],
            );
            match result {
                Ok(next_state) => state = next_state,
                Err(errors) => return Err(errors),
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

    fn next(
        &mut self,
        state: AbstractState,
        offset: usize,
        bytecode: &Bytecode,
    ) -> Result<AbstractState, Vec<VMStaticViolation>> {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                if SignatureTokenView::new(self.module, &operand.signature).is_resource() {
                    Err(vec![VMStaticViolation::PopResourceError(offset)])
                } else if operand.value.is_reference() {
                    Err(vec![VMStaticViolation::PopReferenceError(offset)])
                } else {
                    Ok(state)
                }
            }

            Bytecode::ReleaseRef => {
                let operand = self.stack.pop().unwrap();
                if let AbstractValue::Reference(nonce) = operand.value {
                    let mut next_state = state;
                    next_state.destroy_nonce(nonce);
                    Ok(next_state)
                } else {
                    Err(vec![VMStaticViolation::ReleaseRefTypeMismatchError(offset)])
                }
            }

            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Bool {
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::BrTypeMismatchError(offset)])
                }
            }

            Bytecode::Assert => {
                let condition = self.stack.pop().unwrap();
                let error_code = self.stack.pop().unwrap();
                if condition.signature == SignatureToken::Bool
                    && error_code.signature == SignatureToken::U64
                {
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::AssertTypeMismatchError(offset)])
                }
            }

            Bytecode::StLoc(idx) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != *self.locals_signature_view.token_at(*idx).as_inner() {
                    return Err(vec![VMStaticViolation::StLocTypeMismatchError(offset)]);
                }
                let mut next_state = state;
                if next_state.is_available(*idx) {
                    if self.is_safe_to_destroy(&next_state, *idx) {
                        next_state.destroy_local(*idx);
                    } else {
                        return Err(vec![VMStaticViolation::StLocUnsafeToDestroyError(offset)]);
                    }
                }
                next_state.insert_local(*idx, operand.value);
                Ok(next_state)
            }

            Bytecode::Ret => {
                for arg_idx in 0..self.locals_signature_view.len() {
                    let idx = arg_idx as LocalIndex;
                    if state.is_available(idx) && !self.is_safe_to_destroy(&state, idx) {
                        return Err(vec![VMStaticViolation::RetUnsafeToDestroyError(offset)]);
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
                        return Err(vec![VMStaticViolation::RetTypeMismatchError(offset)]);
                    }
                }
                Ok(AbstractState::new(BTreeMap::new()))
            }

            Bytecode::Branch(_) => Ok(state),

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                    let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                    if self.freeze_ok(&state, borrowed_nonces) {
                        self.stack.push(StackAbstractValue {
                            signature: SignatureToken::Reference(signature),
                            value: operand.value,
                        });
                        return Ok(state);
                    } else {
                        Err(vec![VMStaticViolation::FreezeRefExistsMutableBorrowError(
                            offset,
                        )])
                    }
                } else {
                    Err(vec![VMStaticViolation::FreezeRefTypeMismatchError(offset)])
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
                        let mut next_state = state;
                        let nonce = self.get_nonce(&mut next_state);
                        if operand.signature.is_mutable_reference() {
                            let borrowed_nonces = next_state.borrowed_nonces_for_field(
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
                                next_state.borrow_field_from_nonce(
                                    *field_definition_index,
                                    operand_nonce.clone(),
                                    nonce,
                                );
                                next_state.destroy_nonce(operand_nonce);
                                Ok(next_state)
                            } else {
                                Err(vec![
                                    VMStaticViolation::BorrowFieldExistsMutableBorrowError(offset),
                                ])
                            }
                        } else {
                            self.stack.push(StackAbstractValue {
                                signature: SignatureToken::Reference(Box::new(field_signature)),
                                value: AbstractValue::Reference(nonce.clone()),
                            });
                            next_state.borrow_field_from_nonce(
                                *field_definition_index,
                                operand_nonce.clone(),
                                nonce,
                            );
                            next_state.destroy_nonce(operand_nonce);
                            Ok(next_state)
                        }
                    } else {
                        Err(vec![VMStaticViolation::BorrowFieldBadFieldError(offset)])
                    }
                } else {
                    Err(vec![VMStaticViolation::BorrowFieldTypeMismatchError(
                        offset,
                    )])
                }
            }

            Bytecode::LdConst(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::LdAddr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::LdStr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::String,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::LdByteArray(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    Err(vec![VMStaticViolation::CopyLocUnavailableError(offset)])
                } else if signature_view.is_reference() {
                    let mut next_state = state;
                    let nonce = self.get_nonce(&mut next_state);
                    next_state.borrow_from_local_reference(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(next_state)
                } else if signature_view.is_resource() {
                    Err(vec![VMStaticViolation::CopyLocResourceError(offset)])
                } else if state.is_full(state.local(*idx)) {
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::full_value(false),
                    });
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::CopyLocExistsBorrowError(offset)])
                }
            }

            Bytecode::MoveLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if !state.is_available(*idx) {
                    Err(vec![VMStaticViolation::MoveLocUnavailableError(offset)])
                } else if signature.is_reference() || state.is_full(state.local(*idx)) {
                    let mut next_state = state;
                    let value = next_state.remove_local(*idx);
                    self.stack.push(StackAbstractValue { signature, value });
                    Ok(next_state)
                } else {
                    Err(vec![VMStaticViolation::MoveLocExistsBorrowError(offset)])
                }
            }

            Bytecode::BorrowLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if signature.is_reference() {
                    Err(vec![VMStaticViolation::BorrowLocReferenceError(offset)])
                } else if !state.is_available(*idx) {
                    Err(vec![VMStaticViolation::BorrowLocUnavailableError(offset)])
                } else if state.is_full(state.local(*idx)) {
                    let mut next_state = state;
                    let nonce = self.get_nonce(&mut next_state);
                    next_state.borrow_from_local_value(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(signature)),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(next_state)
                } else {
                    Err(vec![VMStaticViolation::BorrowLocExistsBorrowError(offset)])
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
                        return Err(vec![VMStaticViolation::CallTypeMismatchError(offset)]);
                    }
                    if arg_type.is_mutable_reference() && !state.is_full(&arg.value) {
                        return Err(vec![VMStaticViolation::CallBorrowedMutableReferenceError(
                            offset,
                        )]);
                    }
                    if let AbstractValue::Reference(nonce) = arg.value {
                        all_references_to_borrow_from.insert(nonce.clone());
                        if arg_type.is_mutable_reference() {
                            mutable_references_to_borrow_from.insert(nonce.clone());
                        }
                    }
                }
                let mut next_state = state;
                for return_type_view in function_signature_view.return_tokens() {
                    if return_type_view.is_reference() {
                        let nonce = self.get_nonce(&mut next_state);
                        if return_type_view.is_mutable_reference() {
                            next_state.borrow_from_nonces(
                                &mutable_references_to_borrow_from,
                                nonce.clone(),
                            );
                        } else {
                            next_state
                                .borrow_from_nonces(&all_references_to_borrow_from, nonce.clone());
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
                    next_state.destroy_nonce(x);
                }
                Ok(next_state)
            }

            Bytecode::Pack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                for field_definition_view in struct_definition_view.fields().rev() {
                    let field_signature_view = field_definition_view.type_signature();
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != *field_signature_view.token().as_inner() {
                        return Err(vec![VMStaticViolation::PackTypeMismatchError(offset)]);
                    }
                }
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Struct(struct_definition.struct_handle),
                    value: AbstractValue::full_value(struct_definition_view.is_resource()),
                });
                Ok(state)
            }

            Bytecode::Unpack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_arg = self.stack.pop().unwrap();
                if struct_arg.signature != SignatureToken::Struct(struct_definition.struct_handle) {
                    return Err(vec![VMStaticViolation::UnpackTypeMismatchError(offset)]);
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
                Ok(state)
            }

            Bytecode::ReadRef => {
                let operand = self.stack.pop().unwrap();
                match operand.signature {
                    SignatureToken::Reference(signature) => {
                        let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            Err(vec![VMStaticViolation::ReadRefResourceError(offset)])
                        } else {
                            self.stack.push(StackAbstractValue {
                                signature: *signature,
                                value: AbstractValue::full_value(false),
                            });
                            let mut next_state = state;
                            next_state.destroy_nonce(operand_nonce);
                            Ok(next_state)
                        }
                    }
                    SignatureToken::MutableReference(signature) => {
                        let operand_nonce = Self::extract_nonce(&operand.value).unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            Err(vec![VMStaticViolation::ReadRefResourceError(offset)])
                        } else {
                            let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                            if self.freeze_ok(&state, borrowed_nonces) {
                                self.stack.push(StackAbstractValue {
                                    signature: *signature,
                                    value: AbstractValue::full_value(false),
                                });
                                let mut next_state = state;
                                next_state.destroy_nonce(operand_nonce);
                                Ok(next_state)
                            } else {
                                Err(vec![VMStaticViolation::ReadRefExistsMutableBorrowError(
                                    offset,
                                )])
                            }
                        }
                    }
                    _ => Err(vec![VMStaticViolation::ReadRefTypeMismatchError(offset)]),
                }
            }

            Bytecode::WriteRef => {
                let ref_operand = self.stack.pop().unwrap();
                let val_operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = ref_operand.signature {
                    if SignatureTokenView::new(self.module, &signature).is_resource() {
                        Err(vec![VMStaticViolation::WriteRefResourceError(offset)])
                    } else if val_operand.signature != *signature {
                        Err(vec![VMStaticViolation::WriteRefTypeMismatchError(offset)])
                    } else if state.is_full(&ref_operand.value) {
                        let ref_operand_nonce =
                            Self::extract_nonce(&ref_operand.value).unwrap().clone();
                        let mut next_state = state;
                        next_state.destroy_nonce(ref_operand_nonce);
                        Ok(next_state)
                    } else {
                        Err(vec![VMStaticViolation::WriteRefExistsBorrowError(offset)])
                    }
                } else {
                    Err(vec![VMStaticViolation::WriteRefNoMutableReferenceError(
                        offset,
                    )])
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
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::IntegerOpTypeMismatchError(offset)])
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
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::BooleanOpTypeMismatchError(offset)])
                }
            }

            Bytecode::Not => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Bool {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::BooleanOpTypeMismatchError(offset)])
                }
            }

            Bytecode::Eq | Bytecode::Neq => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature.allows_equality() && operand1.signature == operand2.signature
                {
                    let mut next_state = state;
                    if let AbstractValue::Reference(nonce) = operand1.value {
                        next_state.destroy_nonce(nonce);
                    }
                    if let AbstractValue::Reference(nonce) = operand2.value {
                        next_state.destroy_nonce(nonce);
                    }
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                    Ok(next_state)
                } else {
                    Err(vec![VMStaticViolation::EqualityOpTypeMismatchError(offset)])
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
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::IntegerOpTypeMismatchError(offset)])
                }
            }

            Bytecode::Exists(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::ExistsResourceTypeMismatchError(
                        offset,
                    )])
                }
            }

            Bytecode::BorrowGlobal(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(vec![VMStaticViolation::BorrowGlobalNoResourceError(offset)]);
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    let mut next_state = state;
                    let nonce = self.get_nonce(&mut next_state);
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(
                            SignatureToken::Struct(struct_definition.struct_handle),
                        )),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(next_state)
                } else {
                    Err(vec![VMStaticViolation::BorrowGlobalTypeMismatchError(
                        offset,
                    )])
                }
            }

            Bytecode::MoveFrom(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(vec![VMStaticViolation::MoveFromNoResourceError(offset)]);
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Struct(struct_definition.struct_handle),
                        value: AbstractValue::full_value(true),
                    });
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::MoveFromTypeMismatchError(offset)])
                }
            }

            Bytecode::MoveToSender(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(vec![VMStaticViolation::MoveToSenderNoResourceError(offset)]);
                }

                let value_operand = self.stack.pop().unwrap();
                if value_operand.signature
                    == SignatureToken::Struct(struct_definition.struct_handle)
                {
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::MoveToSenderTypeMismatchError(
                        offset,
                    )])
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
                Ok(state)
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::GetTxnPublicKey => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
                });
                Ok(state)
            }

            Bytecode::CreateAccount => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    Ok(state)
                } else {
                    Err(vec![VMStaticViolation::CreateAccountTypeMismatchError(
                        offset,
                    )])
                }
            }

            Bytecode::EmitEvent => {
                // TODO: EmitEvent is currently unimplemented
                //       following is a workaround to skip the check
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                Ok(state)
            }
        }
    }
}
