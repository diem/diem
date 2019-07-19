// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type and memory safety of a
//! procedure body.
use crate::{
    absint::{AbstractInterpreter, TransferFunctions},
    abstract_state::{AbstractState, AbstractValue},
    control_flow_graph::VMControlFlowGraph,
    nonce::Nonce,
};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    errors::VMStaticViolation,
    file_format::{Bytecode, CompiledModule, FunctionDefinition, LocalIndex, SignatureToken},
    views::{
        FunctionDefinitionView, FunctionSignatureView, LocalsSignatureView, SignatureTokenView,
        StructDefinitionView, ViewInternals,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct StackAbstractValue {
    signature: SignatureToken,
    value: AbstractValue,
}

pub struct TypeAndMemorySafetyAnalysis<'a> {
    module: &'a CompiledModule,
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
                locals.insert(
                    arg_idx as LocalIndex,
                    AbstractValue::full_value(arg_type_view.is_resource()),
                );
            }
        }
        let initial_state = AbstractState::new(locals);
        // nonces in [0, locals_signature_view.len()) are reserved for constructing canonical state
        let next_nonce = locals_signature_view.len();
        let mut verifier = Self {
            module,
            function_definition_view: FunctionDefinitionView::new(module, function_definition),
            locals_signature_view,
            stack: vec![],
            next_nonce,
            errors: vec![],
        };

        verifier.analyze_function(initial_state, &function_definition_view, cfg);
        verifier.errors
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

    fn execute_inner(
        &mut self,
        mut state: &mut AbstractState,
        bytecode: &Bytecode,
        offset: usize,
    ) -> Result<(), VMStaticViolation> {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                if SignatureTokenView::new(self.module, &operand.signature).is_resource() {
                    Err(VMStaticViolation::PopResourceError(offset))
                } else if operand.value.is_reference() {
                    Err(VMStaticViolation::PopReferenceError(offset))
                } else {
                    Ok(())
                }
            }

            Bytecode::ReleaseRef => {
                let operand = self.stack.pop().unwrap();
                if let AbstractValue::Reference(nonce) = operand.value {
                    state.destroy_nonce(nonce);
                    Ok(())
                } else {
                    Err(VMStaticViolation::ReleaseRefTypeMismatchError(offset))
                }
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
                    if state.is_safe_to_destroy(*idx) {
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
                *state = AbstractState::new(BTreeMap::new());
                Ok(())
            }

            Bytecode::Ret => {
                for arg_idx in 0..self.locals_signature_view.len() {
                    let idx = arg_idx as LocalIndex;
                    if state.is_available(idx) && !state.is_safe_to_destroy(idx) {
                        return Err(VMStaticViolation::RetUnsafeToDestroyError(offset));
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
                        return Err(VMStaticViolation::RetTypeMismatchError(offset));
                    }
                }
                *state = AbstractState::new(BTreeMap::new());
                Ok(())
            }

            Bytecode::Branch(_) => Ok(()),

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_nonce = operand.value.extract_nonce().unwrap().clone();
                    let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                    if self.freeze_ok(&state, borrowed_nonces) {
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

            Bytecode::BorrowField(field_definition_index) => {
                let operand = self.stack.pop().unwrap();
                if let Some(struct_handle_index) =
                    SignatureToken::get_struct_handle_from_reference(&operand.signature)
                {
                    if self
                        .module
                        .is_field_in_struct(*field_definition_index, struct_handle_index)
                    {
                        let field_signature = self
                            .module
                            .get_field_signature(*field_definition_index)
                            .0
                            .clone();
                        let operand_nonce = operand.value.extract_nonce().unwrap().clone();
                        let nonce = self.get_nonce(&mut state);
                        if operand.signature.is_mutable_reference() {
                            let borrowed_nonces = state.borrowed_nonces_for_field(
                                *field_definition_index,
                                operand_nonce.clone(),
                            );
                            if Self::write_borrow_ok(borrowed_nonces) {
                                self.stack.push(StackAbstractValue {
                                    signature: SignatureToken::MutableReference(Box::new(
                                        field_signature.clone(),
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
                                return Err(
                                    VMStaticViolation::BorrowFieldExistsMutableBorrowError(offset),
                                );
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
                        return Err(VMStaticViolation::BorrowFieldBadFieldError(offset));
                    }
                } else {
                    return Err(VMStaticViolation::BorrowFieldTypeMismatchError(offset));
                }
                Ok(())
            }

            Bytecode::LdConst(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::LdAddr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::LdStr(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::String,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::LdByteArray(_) => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    Err(VMStaticViolation::CopyLocUnavailableError(offset))
                } else if signature_view.is_reference() {
                    let nonce = self.get_nonce(&mut state);
                    state.borrow_from_local_reference(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(())
                } else if signature_view.is_resource() {
                    Err(VMStaticViolation::CopyLocResourceError(offset))
                } else if state.is_full(state.local(*idx)) {
                    self.stack.push(StackAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::full_value(false),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::CopyLocExistsBorrowError(offset))
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

            Bytecode::BorrowLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if signature.is_reference() {
                    Err(VMStaticViolation::BorrowLocReferenceError(offset))
                } else if !state.is_available(*idx) {
                    Err(VMStaticViolation::BorrowLocUnavailableError(offset))
                } else if state.is_full(state.local(*idx)) {
                    let nonce = self.get_nonce(&mut state);
                    state.borrow_from_local_value(*idx, nonce.clone());
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(signature)),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::BorrowLocExistsBorrowError(offset))
                }
            }

            // TODO: Handle type actuals for generics
            Bytecode::Call(idx, _) => {
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
                        return Err(VMStaticViolation::CallTypeMismatchError(offset));
                    }
                    if arg_type.is_mutable_reference() && !state.is_full(&arg.value) {
                        return Err(VMStaticViolation::CallBorrowedMutableReferenceError(offset));
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
                Ok(())
            }

            // TODO: Handle type actuals for generics
            Bytecode::Pack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_definition_view =
                    StructDefinitionView::new(self.module, struct_definition);
                for field_definition_view in struct_definition_view.fields().rev() {
                    let field_signature_view = field_definition_view.type_signature();
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != *field_signature_view.token().as_inner() {
                        return Err(VMStaticViolation::PackTypeMismatchError(offset));
                    }
                }
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Struct(struct_definition.struct_handle, vec![]),
                    value: AbstractValue::full_value(struct_definition_view.is_resource()),
                });
                Ok(())
            }

            // TODO: Handle type actuals for generics
            Bytecode::Unpack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let struct_arg = self.stack.pop().unwrap();
                if struct_arg.signature
                    != SignatureToken::Struct(struct_definition.struct_handle, vec![])
                {
                    return Err(VMStaticViolation::UnpackTypeMismatchError(offset));
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
                Ok(())
            }

            Bytecode::ReadRef => {
                let operand = self.stack.pop().unwrap();
                match operand.signature {
                    SignatureToken::Reference(signature) => {
                        let operand_nonce = operand.value.extract_nonce().unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            Err(VMStaticViolation::ReadRefResourceError(offset))
                        } else {
                            self.stack.push(StackAbstractValue {
                                signature: *signature,
                                value: AbstractValue::full_value(false),
                            });
                            state.destroy_nonce(operand_nonce);
                            Ok(())
                        }
                    }
                    SignatureToken::MutableReference(signature) => {
                        let operand_nonce = operand.value.extract_nonce().unwrap().clone();
                        if SignatureTokenView::new(self.module, &signature).is_resource() {
                            Err(VMStaticViolation::ReadRefResourceError(offset))
                        } else {
                            let borrowed_nonces = state.borrowed_nonces(operand_nonce.clone());
                            if self.freeze_ok(&state, borrowed_nonces) {
                                self.stack.push(StackAbstractValue {
                                    signature: *signature,
                                    value: AbstractValue::full_value(false),
                                });
                                state.destroy_nonce(operand_nonce);
                                Ok(())
                            } else {
                                Err(VMStaticViolation::ReadRefExistsMutableBorrowError(offset))
                            }
                        }
                    }
                    _ => Err(VMStaticViolation::ReadRefTypeMismatchError(offset)),
                }
            }

            Bytecode::WriteRef => {
                let ref_operand = self.stack.pop().unwrap();
                let val_operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = ref_operand.signature {
                    if SignatureTokenView::new(self.module, &signature).is_resource() {
                        Err(VMStaticViolation::WriteRefResourceError(offset))
                    } else if val_operand.signature != *signature {
                        Err(VMStaticViolation::WriteRefTypeMismatchError(offset))
                    } else if state.is_full(&ref_operand.value) {
                        let ref_operand_nonce = ref_operand.value.extract_nonce().unwrap().clone();
                        state.destroy_nonce(ref_operand_nonce);
                        Ok(())
                    } else {
                        Err(VMStaticViolation::WriteRefExistsBorrowError(offset))
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
                        value: AbstractValue::full_value(false),
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
                        value: AbstractValue::full_value(false),
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
                        value: AbstractValue::full_value(false),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::BooleanOpTypeMismatchError(offset))
                }
            }

            Bytecode::Eq | Bytecode::Neq => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                let is_resource =
                    SignatureTokenView::new(self.module, &operand1.signature).is_resource();
                if !is_resource && operand1.signature == operand2.signature {
                    if let AbstractValue::Reference(nonce) = operand1.value {
                        state.destroy_nonce(nonce);
                    }
                    if let AbstractValue::Reference(nonce) = operand2.value {
                        state.destroy_nonce(nonce);
                    }
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
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
                        value: AbstractValue::full_value(false),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::IntegerOpTypeMismatchError(offset))
                }
            }

            // TODO: Handle type actuals for generics
            Bytecode::Exists(_, _) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::full_value(false),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::ExistsResourceTypeMismatchError(offset))
                }
            }

            // TODO: Handle type actuals for generics
            Bytecode::BorrowGlobal(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(VMStaticViolation::BorrowGlobalNoResourceError(offset));
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    let nonce = self.get_nonce(&mut state);
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::MutableReference(Box::new(
                            SignatureToken::Struct(struct_definition.struct_handle, vec![]),
                        )),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::BorrowGlobalTypeMismatchError(offset))
                }
            }

            // TODO: Handle type actuals for generics
            Bytecode::MoveFrom(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(VMStaticViolation::MoveFromNoResourceError(offset));
                }

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(StackAbstractValue {
                        signature: SignatureToken::Struct(struct_definition.struct_handle, vec![]),
                        value: AbstractValue::full_value(true),
                    });
                    Ok(())
                } else {
                    Err(VMStaticViolation::MoveFromTypeMismatchError(offset))
                }
            }

            // TODO: Handle type actuals for generics
            Bytecode::MoveToSender(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                if !StructDefinitionView::new(self.module, struct_definition).is_resource() {
                    return Err(VMStaticViolation::MoveToSenderNoResourceError(offset));
                }

                let value_operand = self.stack.pop().unwrap();
                if value_operand.signature
                    == SignatureToken::Struct(struct_definition.struct_handle, vec![])
                {
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
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::full_value(false),
                });
                Ok(())
            }

            Bytecode::GetTxnPublicKey => {
                self.stack.push(StackAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::full_value(false),
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

            Bytecode::EmitEvent => {
                // TODO: EmitEvent is currently unimplemented
                //       following is a workaround to skip the check
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                Ok(())
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
