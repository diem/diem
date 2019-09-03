// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type and memory safety of a
//! procedure body.
use crate::{
    absint::{AbstractInterpreter, BlockPrecondition, TransferFunctions},
    abstract_state::{AbstractState, AbstractValue},
    control_flow_graph::VMControlFlowGraph,
    nonce::Nonce,
};
use mirai_annotations::checked_verify;
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    errors::VMStaticViolation,
    file_format::{
        Bytecode, CompiledModule, FieldDefinitionIndex, FunctionDefinition, Kind, LocalIndex,
        LocalsSignatureIndex, SignatureToken, StructDefinitionIndex,
    },
    views::{
        FunctionDefinitionView, FunctionSignatureView, LocalsSignatureView, ModuleView,
        SignatureTokenView, StructDefinitionView, ViewInternals,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct StackAbstractValue {
    signature: SignatureToken,
    value: AbstractValue,
}

pub struct TypeAndMemorySafetyAnalysis<'a> {
    module_view: ModuleView<'a, CompiledModule>,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    locals_signature_view: LocalsSignatureView<'a, CompiledModule>,
    stack: Vec<StackAbstractValue>,
    next_nonce: usize,
    errors: Vec<VMStaticViolation>,
}

impl<'a> TypeAndMemorySafetyAnalysis<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Vec<VMStaticViolation> {
        let module_view = ModuleView::new(module);
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let locals_signature_view = function_definition_view.locals_signature();
        let function_signature_view = function_definition_view.signature();
        let mut locals = BTreeMap::new();
        for (arg_idx, arg_type_view) in function_signature_view.arg_tokens().enumerate() {
            if arg_type_view.is_reference() {
                locals.insert(
                    arg_idx as LocalIndex,
                    AbstractValue::Reference(Nonce::new(arg_idx)),
                );
            } else {
                let arg_kind = arg_type_view
                    .kind(&function_definition_view.signature().as_inner().type_formals);
                locals.insert(arg_idx as LocalIndex, AbstractValue::full_value(arg_kind));
            }
        }
        let initial_state = AbstractState::new(locals, BTreeMap::new());
        // nonces in [0, locals_signature_view.len()) are reserved for constructing canonical state
        let next_nonce = locals_signature_view.len();
        let mut verifier = Self {
            module_view,
            function_definition_view: FunctionDefinitionView::new(module, function_definition),
            locals_signature_view,
            stack: vec![],
            next_nonce,
            errors: vec![],
        };

        let inv_map = verifier.analyze_function(initial_state, &function_definition_view, cfg);
        // Report all the join failures
        for (block_id, inv) in inv_map.iter() {
            match inv.pre() {
                BlockPrecondition::JoinFailure => verifier
                    .errors
                    .push(VMStaticViolation::JoinFailure(*block_id as usize)),
                BlockPrecondition::State(_) => (),
            }
        }
        verifier.errors
    }

    fn module(&self) -> &'a CompiledModule {
        self.module_view.as_inner()
    }

    fn get_nonce(&mut self, state: &mut AbstractState) -> Nonce {
        let nonce = Nonce::new(self.next_nonce);
        state.add_nonce(nonce);
        self.next_nonce += 1;
        nonce
    }

    fn freeze_ok(&self, state: &AbstractState, existing_borrows: &BTreeSet<Nonce>) -> bool {
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

    /// Gives the current constraints on the type formals in the current function.
    fn type_formals(&self) -> &[Kind] {
        &self
            .function_definition_view
            .signature()
            .as_inner()
            .type_formals
    }

    fn is_readable_reference(
        &self,
        state: &AbstractState,
        signature: &SignatureToken,
        nonce: Nonce,
    ) -> bool {
        checked_verify!(signature.is_reference());
        !signature.is_mutable_reference() || {
            let borrowed_nonces = state.borrowed_nonces(nonce);
            self.freeze_ok(state, &borrowed_nonces)
        }
    }

    // helper for both `ImmBorrowFIeld` and `MutBorrowField`
    fn borrow_field(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        field_definition_index: FieldDefinitionIndex,
    ) -> Result<(), VMStaticViolation> {
        let operand = self.stack.pop().unwrap();
        let struct_handle_index =
            match SignatureToken::get_struct_handle_from_reference(&operand.signature) {
                Some(struct_handle_index) => struct_handle_index,
                None => {
                    return Err(VMStaticViolation::BorrowFieldTypeMismatchError(offset));
                }
            };
        if !self
            .module()
            .is_field_in_struct(field_definition_index, struct_handle_index)
        {
            return Err(VMStaticViolation::BorrowFieldBadFieldError(offset));
        }

        let operand_nonce = operand.value.extract_nonce().unwrap();
        let nonce = self.get_nonce(state);
        if mut_ {
            if !operand.signature.is_mutable_reference() {
                return Err(VMStaticViolation::BorrowFieldTypeMismatchError(offset));
            }

            let borrowed_nonces =
                state.borrowed_nonces_for_field(field_definition_index, operand_nonce);
            if !Self::write_borrow_ok(borrowed_nonces) {
                return Err(VMStaticViolation::BorrowFieldExistsMutableBorrowError(
                    offset,
                ));
            }
        } else {
            // No checks needed for immutable case
            if operand.signature.is_mutable_reference() {
                let borrowed_nonces =
                    state.borrowed_nonces_for_field(field_definition_index, operand_nonce);
                if !self.freeze_ok(&state, &borrowed_nonces) {
                    return Err(VMStaticViolation::BorrowFieldExistsMutableBorrowError(
                        offset,
                    ));
                }
            }
        }

        let field_signature = self
            .module()
            .get_field_signature(field_definition_index)
            .0
            .clone();
        let field_token = Box::new(
            field_signature
                .substitute(operand.signature.get_type_actuals_from_reference().unwrap()),
        );
        let signature = if mut_ {
            SignatureToken::MutableReference(field_token)
        } else {
            SignatureToken::Reference(field_token)
        };
        self.stack.push(StackAbstractValue {
            signature,
            value: AbstractValue::Reference(nonce),
        });
        state.borrow_field_from_nonce(field_definition_index, operand_nonce, nonce);
        state.destroy_nonce(operand_nonce);
        Ok(())
    }

    // helper for both `ImmBorrowLoc` and `MutBorrowLoc`
    fn borrow_loc(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: LocalIndex,
    ) -> Result<(), VMStaticViolation> {
        let loc_signature = self.locals_signature_view.token_at(idx).as_inner().clone();

        if loc_signature.is_reference() {
            return Err(VMStaticViolation::BorrowLocReferenceError(offset));
        }
        if !state.is_available(idx) {
            return Err(VMStaticViolation::BorrowLocUnavailableError(offset));
        }

        if mut_ {
            if !state.is_full(state.local(idx)) {
                return Err(VMStaticViolation::BorrowLocExistsBorrowError(offset));
            }
        } else {
            let borrowed_nonces = match state.local(idx) {
                AbstractValue::Reference(_) => {
                    // TODO This should really be a VMInvariantViolation
                    // But it is not supported in the verifier currently
                    return Err(VMStaticViolation::BorrowLocReferenceError(offset));
                }
                AbstractValue::Value(_, nonces) => nonces,
            };
            if !self.freeze_ok(state, &borrowed_nonces) {
                return Err(VMStaticViolation::BorrowLocExistsBorrowError(offset));
            }
        }

        let nonce = self.get_nonce(state);
        state.borrow_from_local_value(idx, nonce);
        let signature = if mut_ {
            SignatureToken::MutableReference(Box::new(loc_signature))
        } else {
            SignatureToken::Reference(Box::new(loc_signature))
        };
        self.stack.push(StackAbstractValue {
            signature,
            value: AbstractValue::Reference(nonce),
        });
        Ok(())
    }

    fn borrow_global(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: StructDefinitionIndex,
        type_actuals_idx: LocalsSignatureIndex,
    ) -> Result<(), VMStaticViolation> {
        let struct_definition = self.module().struct_def_at(idx);
        if !StructDefinitionView::new(self.module(), struct_definition).is_nominal_resource() {
            return Err(VMStaticViolation::BorrowGlobalNoResourceError(offset));
        }

        if mut_ {
            if !state.global(idx).is_empty() {
                return Err(VMStaticViolation::GlobalReferenceError(offset));
            }
        } else {
            let freeze_ok = state
                .global_opt(idx)
                .map(|globals| self.freeze_ok(state, globals))
                .unwrap_or(true);
            if !freeze_ok {
                return Err(VMStaticViolation::GlobalReferenceError(offset));
            }
        }

        let type_actuals = &self.module().locals_signature_at(type_actuals_idx).0;
        let struct_type =
            SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
        SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

        let operand = self.stack.pop().unwrap();
        if operand.signature != SignatureToken::Address {
            return Err(VMStaticViolation::BorrowGlobalTypeMismatchError(offset));
        }

        let nonce = self.get_nonce(state);
        state.borrow_from_global_value(idx, nonce);

        let signature = if mut_ {
            SignatureToken::MutableReference(Box::new(struct_type))
        } else {
            SignatureToken::Reference(Box::new(struct_type))
        };
        self.stack.push(StackAbstractValue {
            signature,
            value: AbstractValue::Reference(nonce),
        });
        Ok(())
    }

    fn execute_inner(
        &mut self,
        state: &mut AbstractState,
        bytecode: &Bytecode,
        offset: usize,
    ) -> Result<(), VMStaticViolation> {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                let kind = SignatureTokenView::new(self.module(), &operand.signature)
                    .kind(self.type_formals());
                if kind != Kind::Unrestricted {
                    return Err(VMStaticViolation::PopResourceError(offset));
                }

                if let AbstractValue::Reference(nonce) = operand.value {
                    state.destroy_nonce(nonce);
                }

                Ok(())
            }

            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != SignatureToken::Bool {
                    Err(VMStaticViolation::BrTypeMismatchError(offset))
                } else {
                    Ok(())
                }
            }

            Bytecode::StLoc(idx) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != *self.locals_signature_view.token_at(*idx).as_inner() {
                    return Err(VMStaticViolation::StLocTypeMismatchError(offset));
                }
                if state.is_available(*idx) {
                    if state.is_local_safe_to_destroy(*idx) {
                        state.destroy_local(*idx);
                    } else {
                        return Err(VMStaticViolation::StLocUnsafeToDestroyError(offset));
                    }
                }
                state.insert_local(*idx, operand.value);
                Ok(())
            }

            Bytecode::Abort => {
                let error_code = self.stack.pop().unwrap();
                if error_code.signature != SignatureToken::U64 {
                    return Err(VMStaticViolation::AbortTypeMismatchError(offset));
                }
                *state = AbstractState::new(BTreeMap::new(), BTreeMap::new());
                Ok(())
            }

            Bytecode::Ret => {
                for return_type_view in self
                    .function_definition_view
                    .signature()
                    .return_tokens()
                    .rev()
                {
                    let operand = self.stack.pop().unwrap();
                    if operand.signature != *return_type_view.as_inner() {
                        return Err(VMStaticViolation::RetTypeMismatchError(offset));
                    }
                }
                for idx in 0..self.locals_signature_view.len() {
                    let is_reference = state.is_available(idx as LocalIndex)
                        && state.local(idx as LocalIndex).is_reference();
                    if is_reference {
                        state.destroy_local(idx as LocalIndex)
                    }
                }
                if !state.is_safe_to_destroy() {
                    return Err(VMStaticViolation::RetUnsafeToDestroyError(offset));
                }

                *state = AbstractState::new(BTreeMap::new(), BTreeMap::new());
                Ok(())
            }

            Bytecode::Branch(_) => Ok(()),

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_nonce = operand.value.extract_nonce().unwrap();
                    let borrowed_nonces = state.borrowed_nonces(operand_nonce);
                    if self.freeze_ok(&state, &borrowed_nonces) {
                        self.stack.push(StackAbstractValue {
                            signature: SignatureToken::Reference(signature),
                            value: operand.value,
                        });
                        Ok(())
                    } else {
                        Err(VMStaticViolation::FreezeRefExistsMutableBorrowError(offset))
                    }
                } else {
                    Err(VMStaticViolation::FreezeRefTypeMismatchError(offset))
                }
            }

            Bytecode::MutBorrowField(field_definition_index) => {
                self.borrow_field(state, offset, true, *field_definition_index)
            }

            Bytecode::ImmBorrowField(field_definition_index) => {
                self.borrow_field(state, offset, false, *field_definition_index)
            }

            Bytecode::LdConst(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdAddr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdStr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::String,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdByteArray(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    Err(VMStaticViolation::CopyLocUnavailableError(offset))
                } else if signature_view.is_reference() {
                    let nonce = self.get_nonce(state);
                    state.borrow_from_local_reference(*idx, nonce);
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(())
                } else {
                    match signature_view.kind(self.type_formals()) {
                        Kind::Resource | Kind::All => {
                            Err(VMStaticViolation::CopyLocResourceError(offset))
                        }
                        Kind::Unrestricted => {
                            if state.is_full(state.local(*idx)) {
                                self.stack.push(StackAbstractValue {
                                    signature: signature_view.as_inner().clone(),
                                    value: AbstractValue::full_value(Kind::Unrestricted),
                                });
                                Ok(())
                            } else {
                                Err(VMStaticViolation::CopyLocExistsBorrowError(offset))
                            }
                        }
                    }
                }
            }

            Bytecode::MoveLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if !state.is_available(*idx) {
                    Err(VMStaticViolation::MoveLocUnavailableError(offset))
                } else if signature.is_reference() || state.is_full(state.local(*idx)) {
                    let value = state.remove_local(*idx);
                    self.stack.push(StackAbstractValue { signature, value });
                    Ok(())
                } else {
                    Err(VMStaticViolation::MoveLocExistsBorrowError(offset))
                }
            }

            Bytecode::MutBorrowLoc(idx) => self.borrow_loc(state, offset, true, *idx),

            Bytecode::ImmBorrowLoc(idx) => self.borrow_loc(state, offset, false, *idx),

            Bytecode::Call(idx, type_actuals_idx) => {
                let function_handle = self.module().function_handle_at(*idx);
                let function_signature = self
                    .module()
                    .function_signature_at(function_handle.signature);

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;

                let function_acquired_resources = self
                    .module_view
                    .function_acquired_resources(&function_handle);
                for acquired_resource in &function_acquired_resources {
                    if !state.global(*acquired_resource).is_empty() {
                        return Err(VMStaticViolation::GlobalReferenceError(offset));
                    }
                }

                let function_signature_view =
                    FunctionSignatureView::new(self.module(), function_signature);
                let mut all_references_to_borrow_from = BTreeSet::new();
                let mut mutable_references_to_borrow_from = BTreeSet::new();
                for arg_type in function_signature.arg_types.iter().rev() {
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != arg_type.substitute(type_actuals) {
                        return Err(VMStaticViolation::CallTypeMismatchError(offset));
                    }
                    if arg_type.is_mutable_reference() && !state.is_full(&arg.value) {
                        return Err(VMStaticViolation::CallBorrowedMutableReferenceError(offset));
                    }
                    if let AbstractValue::Reference(nonce) = arg.value {
                        all_references_to_borrow_from.insert(nonce);
                        if arg_type.is_mutable_reference() {
                            mutable_references_to_borrow_from.insert(nonce);
                        }
                    }
                }
                for return_type_view in function_signature_view.return_tokens() {
                    if return_type_view.is_reference() {
                        let nonce = self.get_nonce(state);
                        if return_type_view.is_mutable_reference() {
                            state.borrow_from_nonces(&mutable_references_to_borrow_from, nonce);
                        } else {
                            state.borrow_from_nonces(&all_references_to_borrow_from, nonce);
                        }
                        self.stack.push(StackAbstractValue {
                            signature: return_type_view.as_inner().substitute(type_actuals),
                            value: AbstractValue::Reference(nonce),
                        });
                    } else {
                        let return_type = return_type_view.as_inner().substitute(type_actuals);
                        let kind = SignatureTokenView::new(self.module(), &return_type)
                            .kind(self.type_formals());
                        self.stack.push(StackAbstractValue {
                            signature: return_type,
                            value: AbstractValue::full_value(kind),
                        });
                    }
                }
                for nonce in all_references_to_borrow_from {
                    state.destroy_nonce(nonce);
                }
                Ok(())
            }

            Bytecode::Pack(idx, type_actuals_idx) => {
                // Build and verify the struct type.
                let struct_definition = self.module().struct_def_at(*idx);
                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                let kind =
                    SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let struct_definition_view =
                    StructDefinitionView::new(self.module(), struct_definition);
                match struct_definition_view.fields() {
                    None => {
                        // TODO pack on native error
                        self.errors
                            .push(VMStaticViolation::PackTypeMismatchError(offset));
                    }
                    Some(fields) => {
                        for field_definition_view in fields.rev() {
                            let field_signature_view = field_definition_view.type_signature();
                            // Substitute type variables with actual types.
                            let field_type = field_signature_view
                                .token()
                                .as_inner()
                                .substitute(type_actuals);
                            // TODO: is it necessary to verify kind constraints here?
                            let arg = self.stack.pop().unwrap();
                            if arg.signature != field_type {
                                self.errors
                                    .push(VMStaticViolation::PackTypeMismatchError(offset));
                            }
                        }
                    }
                }

                self.stack.push(StackAbstractValue {
                    signature: struct_type,
                    value: AbstractValue::full_value(kind),
                });
                Ok(())
            }

            Bytecode::Unpack(idx, type_actuals_idx) => {
                // Build and verify the struct type.
                let struct_definition = self.module().struct_def_at(*idx);
                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());

                // Pop an abstract value from the stack and check if its type is equal to the one
                // declared. TODO: is it safe to not call verify the kinds if the types are equal?
                let arg = self.stack.pop().unwrap();
                if arg.signature != struct_type {
                    return Err(VMStaticViolation::UnpackTypeMismatchError(offset));
                }

                // For each field, push an abstract value to the stack.
                let struct_definition_view =
                    StructDefinitionView::new(self.module(), struct_definition);
                match struct_definition_view.fields() {
                    None => {
                        // TODO unpack on native error
                        self.errors
                            .push(VMStaticViolation::UnpackTypeMismatchError(offset));
                    }
                    Some(fields) => {
                        for field_definition_view in fields {
                            let field_signature_view = field_definition_view.type_signature();
                            // Substitute type variables with actual types.
                            let field_type = field_signature_view
                                .token()
                                .as_inner()
                                .substitute(type_actuals);
                            // Get the kind of the type.
                            let kind = SignatureTokenView::new(self.module(), &field_type)
                                .kind(self.type_formals());
                            self.stack.push(StackAbstractValue {
                                signature: field_type,
                                value: AbstractValue::full_value(kind),
                            })
                        }
                    }
                }
                Ok(())
            }

            Bytecode::ReadRef => {
                let StackAbstractValue {
                    signature: operand_signature,
                    value: operand_value,
                } = self.stack.pop().unwrap();
                if !operand_signature.is_reference() {
                    return Err(VMStaticViolation::ReadRefTypeMismatchError(offset));
                }
                let operand_nonce = operand_value.extract_nonce().unwrap();
                if !self.is_readable_reference(state, &operand_signature, operand_nonce) {
                    Err(VMStaticViolation::ReadRefExistsMutableBorrowError(offset))
                } else {
                    let inner_signature = *match operand_signature {
                        SignatureToken::Reference(signature) => signature,
                        SignatureToken::MutableReference(signature) => signature,
                        _ => panic!("Unreachable"),
                    };
                    if SignatureTokenView::new(self.module(), &inner_signature)
                        .kind(self.type_formals())
                        != Kind::Unrestricted
                    {
                        Err(VMStaticViolation::ReadRefResourceError(offset))
                    } else {
                        self.stack.push(StackAbstractValue {
                            signature: inner_signature,
                            value: AbstractValue::full_value(Kind::Unrestricted),
                        });
                        state.destroy_nonce(operand_nonce);
                        Ok(())
                    }
                }
            }

            Bytecode::WriteRef => {
                let ref_operand = self.stack.pop().unwrap();
                let val_operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = ref_operand.signature {
                    let kind = SignatureTokenView::new(self.module(), &signature)
                        .kind(self.type_formals());
                    match kind {
                        Kind::Resource | Kind::All => {
                            Err(VMStaticViolation::WriteRefResourceError(offset))
                        }
                        Kind::Unrestricted => {
                            if val_operand.signature != *signature {
                                Err(VMStaticViolation::WriteRefTypeMismatchError(offset))
                            } else if state.is_full(&ref_operand.value) {
                                let ref_operand_nonce = ref_operand.value.extract_nonce().unwrap();
                                state.destroy_nonce(ref_operand_nonce);
                                Ok(())
                            } else {
                                Err(VMStaticViolation::WriteRefExistsBorrowError(offset))
                            }
                        }
                    }
                } else {
                    Err(VMStaticViolation::WriteRefNoMutableReferenceError(offset))
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
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::IntegerOpTypeMismatchError(offset))
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
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::BooleanOpTypeMismatchError(offset))
                }
            }

            Bytecode::Not => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Bool {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::BooleanOpTypeMismatchError(offset))
                }
            }

            Bytecode::Eq | Bytecode::Neq => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                let kind1 = SignatureTokenView::new(self.module(), &operand1.signature)
                    .kind(self.type_formals());
                let is_copyable = kind1 == Kind::Unrestricted;
                if is_copyable && operand1.signature == operand2.signature {
                    if let AbstractValue::Reference(nonce) = operand1.value {
                        if self.is_readable_reference(state, &operand1.signature, nonce) {
                            state.destroy_nonce(nonce);
                        } else {
                            return Err(VMStaticViolation::ReadRefExistsMutableBorrowError(offset));
                        }
                    }
                    if let AbstractValue::Reference(nonce) = operand2.value {
                        if self.is_readable_reference(state, &operand2.signature, nonce) {
                            state.destroy_nonce(nonce);
                        } else {
                            return Err(VMStaticViolation::ReadRefExistsMutableBorrowError(offset));
                        }
                    }
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::EqualityOpTypeMismatchError(offset))
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
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::IntegerOpTypeMismatchError(offset))
                }
            }

            Bytecode::Exists(idx, type_actuals_idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                if !StructDefinitionView::new(self.module(), struct_definition)
                    .is_nominal_resource()
                {
                    return Err(VMStaticViolation::ExistsNoResourceError(offset));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::ExistsResourceTypeMismatchError(offset))
                }
            }

            Bytecode::MutBorrowGlobal(idx, type_actuals_idx) => {
                self.borrow_global(state, offset, true, *idx, *type_actuals_idx)
            }

            Bytecode::ImmBorrowGlobal(idx, type_actuals_idx) => {
                self.borrow_global(state, offset, false, *idx, *type_actuals_idx)
            }

            Bytecode::MoveFrom(idx, type_actuals_idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                if !StructDefinitionView::new(self.module(), struct_definition)
                    .is_nominal_resource()
                {
                    return Err(VMStaticViolation::MoveFromNoResourceError(offset));
                } else if !state.global(*idx).is_empty() {
                    return Err(VMStaticViolation::GlobalReferenceError(offset));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: struct_type,
                        value: AbstractValue::full_value(Kind::Resource),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::MoveFromTypeMismatchError(offset))
                }
            }

            Bytecode::MoveToSender(idx, type_actuals_idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                if !StructDefinitionView::new(self.module(), struct_definition)
                    .is_nominal_resource()
                {
                    return Err(VMStaticViolation::MoveToSenderNoResourceError(offset));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let value_operand = self.stack.pop().unwrap();
                if value_operand.signature == struct_type {
                    Ok(())
                } else {
                    Err(VMStaticViolation::MoveToSenderTypeMismatchError(offset))
                }
            }

            Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnSequenceNumber => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::GetTxnPublicKey => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::CreateAccount => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    Ok(())
                } else {
                    Err(VMStaticViolation::CreateAccountTypeMismatchError(offset))
                }
            }
        }
    }
}

impl<'a> TransferFunctions for TypeAndMemorySafetyAnalysis<'a> {
    type State = AbstractState;
    type AnalysisError = VMStaticViolation;

    fn execute(
        &mut self,
        state: &mut Self::State,
        bytecode: &Bytecode,
        index: usize,
        last_index: usize,
    ) -> Result<(), Self::AnalysisError> {
        match self.execute_inner(state, bytecode, index) {
            Err(err) => {
                self.errors.push(err.clone());
                Err(err)
            }
            Ok(()) => {
                if index == last_index {
                    *state = state.construct_canonical_state()
                }
                Ok(())
            }
        }
    }
}

impl<'a> AbstractInterpreter for TypeAndMemorySafetyAnalysis<'a> {}
