// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type and memory safety of a
//! procedure body.
use crate::{
    absint::{AbstractInterpreter, BlockPrecondition, TransferFunctions},
    abstract_state::{AbstractState, AbstractValue, TypedAbstractValue},
    control_flow_graph::VMControlFlowGraph,
    nonce::Nonce,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use mirai_annotations::checked_verify;
use std::collections::BTreeSet;
use vm::{
    access::ModuleAccess,
    errors::{err_at_offset, VMResult},
    file_format::{
        Bytecode, CompiledModule, FieldDefinitionIndex, FunctionDefinition, Kind, LocalIndex,
        LocalsSignatureIndex, SignatureToken, StructDefinitionIndex,
    },
    views::{
        FunctionDefinitionView, FunctionSignatureView, LocalsSignatureView, ModuleView,
        SignatureTokenView, StructDefinitionView, ViewInternals,
    },
};

pub struct TypeAndMemorySafetyAnalysis<'a> {
    module_view: ModuleView<'a, CompiledModule>,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    locals_signature_view: LocalsSignatureView<'a, CompiledModule>,
    stack: Vec<TypedAbstractValue>,
    errors: Vec<VMStatus>,
}

impl<'a> TypeAndMemorySafetyAnalysis<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Vec<VMStatus> {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let locals_signature_view = function_definition_view.locals_signature();
        let function_signature_view = function_definition_view.signature();
        if function_signature_view.arg_count() > locals_signature_view.len() {
            return vec![VMStatus::new(StatusCode::RANGE_OUT_OF_BOUNDS)
                .with_message("Fewer locals than parameters".to_string())];
        }
        let errors: Vec<VMStatus> = function_signature_view
            .arg_tokens()
            .enumerate()
            .flat_map(|(arg_idx, arg_type_view)| {
                let arg_token = arg_type_view.as_inner();
                let local_token = locals_signature_view
                    .token_at(arg_idx as LocalIndex)
                    .as_inner();
                if arg_token == local_token {
                    vec![]
                } else {
                    vec![
                        VMStatus::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                            "Type mismatch at index {} between parameter and local",
                            arg_idx
                        )),
                    ]
                }
            })
            .collect();
        if !errors.is_empty() {
            return errors;
        }
        let initial_state =
            AbstractState::new(FunctionDefinitionView::new(module, function_definition));
        let mut verifier = Self {
            module_view: ModuleView::new(module),
            function_definition_view: FunctionDefinitionView::new(module, function_definition),
            locals_signature_view,
            stack: vec![],
            errors: vec![],
        };

        let inv_map = verifier.analyze_function(initial_state, &function_definition_view, cfg);
        // Report all the join failures
        for (block_id, inv) in inv_map.iter() {
            match inv.pre() {
                BlockPrecondition::JoinFailure => verifier
                    .errors
                    .push(err_at_offset(StatusCode::JOIN_FAILURE, *block_id as usize)),
                BlockPrecondition::State(_) => (),
            }
        }
        verifier.errors
    }

    fn module(&self) -> &'a CompiledModule {
        self.module_view.as_inner()
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
        state: &AbstractState,
        signature: &SignatureToken,
        nonce: Nonce,
    ) -> bool {
        checked_verify!(signature.is_reference());
        !signature.is_mutable_reference() || state.is_nonce_freezable(nonce)
    }

    // helper for both `ImmBorrowField` and `MutBorrowField`
    fn borrow_field(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        field_definition_index: FieldDefinitionIndex,
    ) -> VMResult<()> {
        let operand = self.stack.pop().unwrap();
        let struct_handle_index =
            match SignatureToken::get_struct_handle_from_reference(&operand.signature) {
                Some(struct_handle_index) => struct_handle_index,
                None => {
                    return Err(err_at_offset(
                        StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }
            };
        if !self
            .module()
            .is_field_in_struct(field_definition_index, struct_handle_index)
        {
            return Err(err_at_offset(
                StatusCode::BORROWFIELD_BAD_FIELD_ERROR,
                offset,
            ));
        }
        if mut_ && !operand.signature.is_mutable_reference() {
            return Err(err_at_offset(
                StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR,
                offset,
            ));
        }

        if let Some(nonce) = state.borrow_field_from_nonce(&operand, mut_, field_definition_index) {
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
            self.stack.push(TypedAbstractValue {
                signature,
                value: AbstractValue::Reference(nonce),
            });
            let operand_nonce = operand.value.extract_nonce().unwrap();
            state.remove_nonce(operand_nonce);
            Ok(())
        } else {
            Err(err_at_offset(
                StatusCode::BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR,
                offset,
            ))
        }
    }

    // helper for both `ImmBorrowLoc` and `MutBorrowLoc`
    fn borrow_loc(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: LocalIndex,
    ) -> VMResult<()> {
        let loc_signature = self.locals_signature_view.token_at(idx).as_inner().clone();

        if loc_signature.is_reference() {
            return Err(err_at_offset(StatusCode::BORROWLOC_REFERENCE_ERROR, offset));
        }
        if !state.is_available(idx) {
            return Err(err_at_offset(
                StatusCode::BORROWLOC_UNAVAILABLE_ERROR,
                offset,
            ));
        }

        if let Some(nonce) = state.borrow_local_value(mut_, idx) {
            let signature = if mut_ {
                SignatureToken::MutableReference(Box::new(loc_signature))
            } else {
                SignatureToken::Reference(Box::new(loc_signature))
            };
            self.stack.push(TypedAbstractValue {
                signature,
                value: AbstractValue::Reference(nonce),
            });
            Ok(())
        } else {
            Err(err_at_offset(
                StatusCode::BORROWLOC_EXISTS_BORROW_ERROR,
                offset,
            ))
        }
    }

    fn borrow_global(
        &mut self,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: StructDefinitionIndex,
        type_actuals_idx: LocalsSignatureIndex,
    ) -> VMResult<()> {
        let struct_definition = self.module().struct_def_at(idx);
        if !StructDefinitionView::new(self.module(), struct_definition).is_nominal_resource() {
            return Err(err_at_offset(
                StatusCode::BORROWGLOBAL_NO_RESOURCE_ERROR,
                offset,
            ));
        }

        let type_actuals = &self.module().locals_signature_at(type_actuals_idx).0;
        let struct_type =
            SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
        SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());
        let operand = self.stack.pop().unwrap();
        if operand.signature != SignatureToken::Address {
            return Err(err_at_offset(
                StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR,
                offset,
            ));
        }

        if let Some(nonce) = state.borrow_global_value(mut_, idx) {
            let signature = if mut_ {
                SignatureToken::MutableReference(Box::new(struct_type))
            } else {
                SignatureToken::Reference(Box::new(struct_type))
            };
            self.stack.push(TypedAbstractValue {
                signature,
                value: AbstractValue::Reference(nonce),
            });
            Ok(())
        } else {
            Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset))
        }
    }

    fn execute_inner(
        &mut self,
        state: &mut AbstractState,
        bytecode: &Bytecode,
        offset: usize,
    ) -> VMResult<()> {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                let kind = SignatureTokenView::new(self.module(), &operand.signature)
                    .kind(self.type_formals());
                if kind != Kind::Unrestricted {
                    return Err(err_at_offset(StatusCode::POP_RESOURCE_ERROR, offset));
                }

                if let AbstractValue::Reference(nonce) = operand.value {
                    state.remove_nonce(nonce);
                }

                Ok(())
            }

            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != SignatureToken::Bool {
                    Err(err_at_offset(StatusCode::BR_TYPE_MISMATCH_ERROR, offset))
                } else {
                    Ok(())
                }
            }

            Bytecode::StLoc(idx) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != *self.locals_signature_view.token_at(*idx).as_inner() {
                    return Err(err_at_offset(StatusCode::STLOC_TYPE_MISMATCH_ERROR, offset));
                }
                if state.is_available(*idx) {
                    if state.is_local_safe_to_destroy(*idx) {
                        state.destroy_local(*idx);
                    } else {
                        return Err(err_at_offset(
                            StatusCode::STLOC_UNSAFE_TO_DESTROY_ERROR,
                            offset,
                        ));
                    }
                }
                state.insert_local(*idx, operand);
                Ok(())
            }

            Bytecode::Abort => {
                let error_code = self.stack.pop().unwrap();
                if error_code.signature != SignatureToken::U64 {
                    return Err(err_at_offset(StatusCode::ABORT_TYPE_MISMATCH_ERROR, offset));
                }
                *state = AbstractState::default();
                Ok(())
            }

            Bytecode::Ret => {
                for idx in 0..self.locals_signature_view.len() {
                    let local_idx = idx as LocalIndex;
                    let is_reference = state.is_available(local_idx)
                        && state.local(local_idx).value.is_reference();
                    if is_reference {
                        state.destroy_local(local_idx);
                    }
                }
                if !state.is_frame_safe_to_destroy() {
                    return Err(err_at_offset(
                        StatusCode::RET_UNSAFE_TO_DESTROY_ERROR,
                        offset,
                    ));
                }
                for return_type_view in self
                    .function_definition_view
                    .signature()
                    .return_tokens()
                    .rev()
                {
                    let operand = self.stack.pop().unwrap();
                    if operand.signature != *return_type_view.as_inner() {
                        return Err(err_at_offset(StatusCode::RET_TYPE_MISMATCH_ERROR, offset));
                    }
                    if return_type_view.is_mutable_reference() {
                        if let AbstractValue::Reference(nonce) = operand.value {
                            if state.is_nonce_borrowed(nonce) {
                                return Err(err_at_offset(
                                    StatusCode::RET_BORROWED_MUTABLE_REFERENCE_ERROR,
                                    offset,
                                ));
                            }
                        }
                    }
                }
                *state = AbstractState::default();
                Ok(())
            }

            Bytecode::Branch(_) => Ok(()),

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_nonce = operand.value.extract_nonce().unwrap();
                    if state.is_nonce_freezable(operand_nonce) {
                        self.stack.push(TypedAbstractValue {
                            signature: SignatureToken::Reference(signature),
                            value: operand.value,
                        });
                        Ok(())
                    } else {
                        Err(err_at_offset(
                            StatusCode::FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR,
                            offset,
                        ))
                    }
                } else {
                    Err(err_at_offset(
                        StatusCode::FREEZEREF_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::MutBorrowField(field_definition_index) => {
                self.borrow_field(state, offset, true, *field_definition_index)
            }

            Bytecode::ImmBorrowField(field_definition_index) => {
                self.borrow_field(state, offset, false, *field_definition_index)
            }

            Bytecode::LdU8(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U8,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdU64(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdU128(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U128,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdAddr(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdByteArray(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    Err(err_at_offset(StatusCode::COPYLOC_UNAVAILABLE_ERROR, offset))
                } else if signature_view.is_reference() {
                    let nonce = state.borrow_from_local_reference(*idx);
                    self.stack.push(TypedAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    });
                    Ok(())
                } else {
                    match signature_view.kind(self.type_formals()) {
                        Kind::Resource | Kind::All => {
                            Err(err_at_offset(StatusCode::COPYLOC_RESOURCE_ERROR, offset))
                        }
                        Kind::Unrestricted => {
                            if !state.is_local_borrowed(*idx) {
                                self.stack.push(TypedAbstractValue {
                                    signature: signature_view.as_inner().clone(),
                                    value: AbstractValue::Value(Kind::Unrestricted),
                                });
                                Ok(())
                            } else {
                                Err(err_at_offset(
                                    StatusCode::COPYLOC_EXISTS_BORROW_ERROR,
                                    offset,
                                ))
                            }
                        }
                    }
                }
            }

            Bytecode::MoveLoc(idx) => {
                let signature = self.locals_signature_view.token_at(*idx).as_inner().clone();
                if !state.is_available(*idx) {
                    Err(err_at_offset(StatusCode::MOVELOC_UNAVAILABLE_ERROR, offset))
                } else if signature.is_reference() || !state.is_local_borrowed(*idx) {
                    let value = state.remove_local(*idx);
                    self.stack.push(value);
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::MOVELOC_EXISTS_BORROW_ERROR,
                        offset,
                    ))
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
                    if state.is_global_borrowed(*acquired_resource) {
                        return Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
                    }
                }

                let function_signature_view =
                    FunctionSignatureView::new(self.module(), function_signature);
                let mut all_references_to_borrow_from = BTreeSet::new();
                let mut mutable_references_to_borrow_from = BTreeSet::new();
                for arg_type in function_signature.arg_types.iter().rev() {
                    let arg = self.stack.pop().unwrap();
                    if arg.signature != arg_type.substitute(type_actuals) {
                        return Err(err_at_offset(StatusCode::CALL_TYPE_MISMATCH_ERROR, offset));
                    }
                    if let AbstractValue::Reference(nonce) = arg.value {
                        if arg_type.is_mutable_reference() {
                            if state.is_nonce_borrowed(nonce) {
                                return Err(err_at_offset(
                                    StatusCode::CALL_BORROWED_MUTABLE_REFERENCE_ERROR,
                                    offset,
                                ));
                            }
                            mutable_references_to_borrow_from.insert(nonce);
                        }
                        all_references_to_borrow_from.insert(nonce);
                    }
                }
                for return_type_view in function_signature_view.return_tokens() {
                    if return_type_view.is_reference() {
                        let nonce = if return_type_view.is_mutable_reference() {
                            state.borrow_from_nonces(&mutable_references_to_borrow_from)
                        } else {
                            state.borrow_from_nonces(&all_references_to_borrow_from)
                        };
                        self.stack.push(TypedAbstractValue {
                            signature: return_type_view.as_inner().substitute(type_actuals),
                            value: AbstractValue::Reference(nonce),
                        });
                    } else {
                        let return_type = return_type_view.as_inner().substitute(type_actuals);
                        let kind = SignatureTokenView::new(self.module(), &return_type)
                            .kind(self.type_formals());
                        self.stack.push(TypedAbstractValue {
                            signature: return_type,
                            value: AbstractValue::Value(kind),
                        });
                    }
                }
                for nonce in all_references_to_borrow_from {
                    state.remove_nonce(nonce);
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
                            .push(err_at_offset(StatusCode::PACK_TYPE_MISMATCH_ERROR, offset));
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
                                self.errors.push(err_at_offset(
                                    StatusCode::PACK_TYPE_MISMATCH_ERROR,
                                    offset,
                                ));
                            }
                        }
                    }
                }

                self.stack.push(TypedAbstractValue {
                    signature: struct_type,
                    value: AbstractValue::Value(kind),
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
                    return Err(err_at_offset(
                        StatusCode::UNPACK_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }

                // For each field, push an abstract value to the stack.
                let struct_definition_view =
                    StructDefinitionView::new(self.module(), struct_definition);
                match struct_definition_view.fields() {
                    None => {
                        // TODO unpack on native error
                        self.errors.push(err_at_offset(
                            StatusCode::UNPACK_TYPE_MISMATCH_ERROR,
                            offset,
                        ));
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
                            self.stack.push(TypedAbstractValue {
                                signature: field_type,
                                value: AbstractValue::Value(kind),
                            })
                        }
                    }
                }
                Ok(())
            }

            Bytecode::ReadRef => {
                let TypedAbstractValue {
                    signature: operand_signature,
                    value: operand_value,
                } = self.stack.pop().unwrap();
                if !operand_signature.is_reference() {
                    return Err(err_at_offset(
                        StatusCode::READREF_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }
                let operand_nonce = operand_value.extract_nonce().unwrap();
                if !Self::is_readable_reference(state, &operand_signature, operand_nonce) {
                    Err(err_at_offset(
                        StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                        offset,
                    ))
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
                        Err(err_at_offset(StatusCode::READREF_RESOURCE_ERROR, offset))
                    } else {
                        self.stack.push(TypedAbstractValue {
                            signature: inner_signature,
                            value: AbstractValue::Value(Kind::Unrestricted),
                        });
                        state.remove_nonce(operand_nonce);
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
                            Err(err_at_offset(StatusCode::WRITEREF_RESOURCE_ERROR, offset))
                        }
                        Kind::Unrestricted => {
                            if val_operand.signature != *signature {
                                Err(err_at_offset(
                                    StatusCode::WRITEREF_TYPE_MISMATCH_ERROR,
                                    offset,
                                ))
                            } else {
                                let ref_operand_nonce = ref_operand.value.extract_nonce().unwrap();
                                if !state.is_nonce_borrowed(ref_operand_nonce) {
                                    state.remove_nonce(ref_operand_nonce);
                                    Ok(())
                                } else {
                                    Err(err_at_offset(
                                        StatusCode::WRITEREF_EXISTS_BORROW_ERROR,
                                        offset,
                                    ))
                                }
                            }
                        }
                    }
                } else {
                    Err(err_at_offset(
                        StatusCode::WRITEREF_NO_MUTABLE_REFERENCE_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::CastU8 => {
                let operand = self.stack.pop().unwrap();
                if operand.signature.is_integer() {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::U8,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }
            Bytecode::CastU64 => {
                let operand = self.stack.pop().unwrap();
                if operand.signature.is_integer() {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::U64,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }
            Bytecode::CastU128 => {
                let operand = self.stack.pop().unwrap();
                if operand.signature.is_integer() {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::U128,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
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
                if operand1.signature.is_integer() && operand1.signature == operand2.signature {
                    self.stack.push(TypedAbstractValue {
                        signature: operand1.signature,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Shl | Bytecode::Shr => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature.is_integer() && operand2.signature.is_integer() {
                    self.stack.push(TypedAbstractValue {
                        signature: operand2.signature,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Or | Bytecode::And => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature == SignatureToken::Bool
                    && operand2.signature == SignatureToken::Bool
                {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Not => {
                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Bool {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
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
                        if Self::is_readable_reference(state, &operand1.signature, nonce) {
                            state.remove_nonce(nonce);
                        } else {
                            return Err(err_at_offset(
                                StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                                offset,
                            ));
                        }
                    }
                    if let AbstractValue::Reference(nonce) = operand2.value {
                        if Self::is_readable_reference(state, &operand2.signature, nonce) {
                            state.remove_nonce(nonce);
                        } else {
                            return Err(err_at_offset(
                                StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                                offset,
                            ));
                        }
                    }
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::EQUALITY_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature.is_integer() && operand1.signature == operand2.signature {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Exists(idx, type_actuals_idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                if !StructDefinitionView::new(self.module(), struct_definition)
                    .is_nominal_resource()
                {
                    return Err(err_at_offset(
                        StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Unrestricted),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
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
                    return Err(err_at_offset(
                        StatusCode::MOVEFROM_NO_RESOURCE_ERROR,
                        offset,
                    ));
                } else if state.is_global_borrowed(*idx) {
                    return Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let operand = self.stack.pop().unwrap();
                if operand.signature == SignatureToken::Address {
                    self.stack.push(TypedAbstractValue {
                        signature: struct_type,
                        value: AbstractValue::Value(Kind::Resource),
                    });
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::MOVEFROM_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::MoveToSender(idx, type_actuals_idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                if !StructDefinitionView::new(self.module(), struct_definition)
                    .is_nominal_resource()
                {
                    return Err(err_at_offset(
                        StatusCode::MOVETOSENDER_NO_RESOURCE_ERROR,
                        offset,
                    ));
                }

                let type_actuals = &self.module().locals_signature_at(*type_actuals_idx).0;
                let struct_type =
                    SignatureToken::Struct(struct_definition.struct_handle, type_actuals.clone());
                SignatureTokenView::new(self.module(), &struct_type).kind(self.type_formals());

                let value_operand = self.stack.pop().unwrap();
                if value_operand.signature == struct_type {
                    Ok(())
                } else {
                    Err(err_at_offset(
                        StatusCode::MOVETOSENDER_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnSequenceNumber => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }

            Bytecode::GetTxnPublicKey => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::ByteArray,
                    value: AbstractValue::Value(Kind::Unrestricted),
                });
                Ok(())
            }
        }
    }
}

impl<'a> TransferFunctions for TypeAndMemorySafetyAnalysis<'a> {
    type State = AbstractState;
    type AnalysisError = VMStatus;

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
