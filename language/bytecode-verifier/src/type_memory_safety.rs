// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type and memory safety of a
//! procedure body.
use crate::{
    absint::{
        AbstractInterpreter, BlockInvariant, BlockPostcondition, BlockPrecondition,
        TransferFunctions,
    },
    abstract_state::{AbstractState, AbstractValue, TypedAbstractValue},
    control_flow_graph::VMControlFlowGraph,
    signature::kind,
};
use borrow_graph::references::RefID;
use libra_types::vm_error::{StatusCode, VMStatus};
use mirai_annotations::*;
use std::collections::BTreeSet;
use vm::{
    access::ModuleAccess,
    errors::err_at_offset,
    file_format::{
        Bytecode, CompiledModule, FieldHandleIndex, FunctionDefinition, FunctionHandle, Kind,
        LocalIndex, Signature, SignatureToken, StructDefinition, StructDefinitionIndex,
        StructFieldInformation, StructHandleIndex,
    },
    views::{
        FunctionDefinitionView, FunctionHandleView, ModuleView, SignatureView,
        StructDefinitionView, ViewInternals,
    },
};

pub struct TypeAndMemorySafetyAnalysis<'a> {
    module_view: ModuleView<'a, CompiledModule>,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    locals_signature_view: SignatureView<'a, CompiledModule>,
    stack: Vec<TypedAbstractValue>,
}

impl<'a> TypeAndMemorySafetyAnalysis<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Vec<VMStatus> {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let locals_signature_view = function_definition_view.locals_signature();
        // TODO: this check should probably be elsewhere
        if function_definition_view.arg_count() > locals_signature_view.len() {
            return vec![VMStatus::new(StatusCode::RANGE_OUT_OF_BOUNDS)
                .with_message("Fewer locals than parameters".to_string())];
        }
        let errors: Vec<VMStatus> = function_definition_view
            .arg_tokens()
            .enumerate()
            .flat_map(|(arg_idx, parameter_view)| {
                let arg_token = parameter_view.as_inner();
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
        let initial_state = AbstractState::new(module, function_definition);
        let mut verifier = Self {
            module_view: ModuleView::new(module),
            function_definition_view: FunctionDefinitionView::new(module, function_definition),
            locals_signature_view,
            stack: vec![],
        };

        let mut errors = vec![];
        let inv_map = verifier.analyze_function(initial_state, &function_definition_view, cfg);
        // Report all the join failures
        for (block_id, BlockInvariant { pre, post }) in inv_map {
            match pre {
                BlockPrecondition::JoinFailure => {
                    errors.push(err_at_offset(StatusCode::JOIN_FAILURE, block_id as usize))
                }
                BlockPrecondition::State(_) => (),
            }
            match post {
                BlockPostcondition::Error(mut err) => {
                    assert!(!err.is_empty());
                    errors.append(&mut err);
                }
                BlockPostcondition::Success => (),
            }
        }
        errors
    }

    fn module(&self) -> &'a CompiledModule {
        self.module_view.as_inner()
    }

    /// Gives the current constraints on the type formals in the current function.
    fn type_parameters(&self) -> &[Kind] {
        self.function_definition_view.type_parameters()
    }

    fn is_readable_reference(state: &AbstractState, signature: &SignatureToken, id: RefID) -> bool {
        checked_verify!(signature.is_reference());
        !signature.is_mutable_reference() || state.is_freezable(id)
    }

    // helper for both `ImmBorrowField` and `MutBorrowField`
    fn borrow_field(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        field_handle_index: FieldHandleIndex,
        type_args: &Signature,
    ) {
        // load operand and check mutability constraints
        let operand = self.stack.pop().unwrap();
        if mut_ && !operand.signature.is_mutable_reference() {
            errors.push(err_at_offset(
                StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR,
                offset,
            ));
            return;
        }

        // check the reference on the stack is the expected type.
        // Load the type that owns the field according to the instruction.
        // For generic fields access, this step materializes that type
        let field_handle = self.module().field_handle_at(field_handle_index);
        let struct_def = self.module().struct_def_at(field_handle.owner);
        let expected_type = materialize_type(struct_def.struct_handle, type_args);
        if !operand.reference_to(&expected_type) {
            errors.push(err_at_offset(
                StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR,
                offset,
            ));
            return;
        }

        match state.borrow_field(&operand, mut_, field_handle_index) {
            Some(id) => {
                // load the field type
                let field_def = match &struct_def.field_information {
                    StructFieldInformation::Native => {
                        errors.push(err_at_offset(
                            StatusCode::BORROWFIELD_BAD_FIELD_ERROR,
                            offset,
                        ));
                        return;
                    }
                    StructFieldInformation::Declared(fields) => {
                        // TODO: review the whole error story here, way too much is left to chances...
                        // definition of a more proper OM for the verifier could work around the problem
                        // (maybe, maybe not..)
                        &fields[field_handle.field as usize]
                    }
                };
                let field_type = Box::new(instantiate(&field_def.signature.0, type_args));
                let signature = if mut_ {
                    SignatureToken::MutableReference(field_type)
                } else {
                    SignatureToken::Reference(field_type)
                };
                self.stack.push(TypedAbstractValue {
                    signature,
                    value: AbstractValue::Reference(id),
                });
                let operand_id = operand.value.extract_id().unwrap();
                state.release(operand_id);
            }
            None => errors.push(err_at_offset(
                StatusCode::BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR,
                offset,
            )),
        }
    }

    // helper for both `ImmBorrowLoc` and `MutBorrowLoc`
    fn borrow_loc(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: LocalIndex,
    ) {
        let loc_signature = self.locals_signature_view.token_at(idx).as_inner().clone();

        if loc_signature.is_reference() {
            errors.push(err_at_offset(StatusCode::BORROWLOC_REFERENCE_ERROR, offset));
            return;
        }
        if !state.is_available(idx) {
            errors.push(err_at_offset(
                StatusCode::BORROWLOC_UNAVAILABLE_ERROR,
                offset,
            ));
            return;
        }

        if let Some(id) = state.borrow_local_value(mut_, idx) {
            let signature = if mut_ {
                SignatureToken::MutableReference(Box::new(loc_signature))
            } else {
                SignatureToken::Reference(Box::new(loc_signature))
            };
            self.stack.push(TypedAbstractValue {
                signature,
                value: AbstractValue::Reference(id),
            });
        } else {
            errors.push(err_at_offset(
                StatusCode::BORROWLOC_EXISTS_BORROW_ERROR,
                offset,
            ))
        }
    }

    fn borrow_global(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        offset: usize,
        mut_: bool,
        idx: StructDefinitionIndex,
        type_args: &Signature,
    ) {
        // check and consume top of stack
        let operand = self.stack.pop().unwrap();
        if operand.signature != SignatureToken::Address {
            errors.push(err_at_offset(
                StatusCode::BORROWGLOBAL_TYPE_MISMATCH_ERROR,
                offset,
            ));
            return;
        }

        let struct_def = self.module().struct_def_at(idx);
        let struct_handle = self.module().struct_handle_at(struct_def.struct_handle);
        if !struct_handle.is_nominal_resource {
            errors.push(err_at_offset(
                StatusCode::BORROWGLOBAL_NO_RESOURCE_ERROR,
                offset,
            ));
            return;
        }

        if let Some(id) = state.borrow_global_value(mut_, idx) {
            let struct_type = materialize_type(struct_def.struct_handle, type_args);
            let signature = if mut_ {
                SignatureToken::MutableReference(Box::new(struct_type))
            } else {
                SignatureToken::Reference(Box::new(struct_type))
            };
            self.stack.push(TypedAbstractValue {
                signature,
                value: AbstractValue::Reference(id),
            });
        } else {
            errors.push(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset))
        }
    }

    fn call(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        offset: usize,
        function_handle: &FunctionHandle,
        type_actuals: &Signature,
    ) {
        let function_acquired_resources = self
            .module_view
            .function_acquired_resources(&function_handle);
        for acquired_resource in &function_acquired_resources {
            if state.is_global_borrowed(*acquired_resource) {
                errors.push(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
                return;
            }
        }

        let mut all_references_to_borrow_from = BTreeSet::new();
        let mut mutable_references_to_borrow_from = BTreeSet::new();
        let parameters = self.module().signature_at(function_handle.parameters);
        for parameter in parameters.0.iter().rev() {
            let arg = self.stack.pop().unwrap();
            if arg.signature != instantiate(parameter, type_actuals) {
                errors.push(err_at_offset(StatusCode::CALL_TYPE_MISMATCH_ERROR, offset));
                return;
            }
            if let AbstractValue::Reference(id) = arg.value {
                if parameter.is_mutable_reference() {
                    if state.is_borrowed(id) {
                        errors.push(err_at_offset(
                            StatusCode::CALL_BORROWED_MUTABLE_REFERENCE_ERROR,
                            offset,
                        ));
                        return;
                    }
                    mutable_references_to_borrow_from.insert(id);
                }
                all_references_to_borrow_from.insert(id);
            }
        }
        let function_handle_view = FunctionHandleView::new(self.module(), function_handle);
        for return_type_view in function_handle_view.return_tokens() {
            if return_type_view.is_reference() {
                let id = if return_type_view.is_mutable_reference() {
                    state.borrow_from(&mutable_references_to_borrow_from, true)
                } else {
                    state.borrow_from(&all_references_to_borrow_from, false)
                };
                self.stack.push(TypedAbstractValue {
                    signature: instantiate(return_type_view.as_inner(), type_actuals),
                    value: AbstractValue::Reference(id),
                });
            } else {
                let return_type = instantiate(return_type_view.as_inner(), type_actuals);
                let k = kind(self.module(), &return_type, self.type_parameters());
                self.stack.push(TypedAbstractValue {
                    signature: return_type,
                    value: AbstractValue::Value(k),
                });
            }
        }
        for id in all_references_to_borrow_from {
            state.release(id);
        }
    }

    fn type_fields_signature(
        &mut self,
        errors: &mut Vec<VMStatus>,
        offset: usize,
        struct_def: &StructDefinition,
        type_args: &Signature,
    ) -> Option<Signature> {
        let mut field_sig = vec![];
        match &struct_def.field_information {
            StructFieldInformation::Native => {
                // TODO: this is more of "unreachable"
                errors.push(err_at_offset(StatusCode::PACK_TYPE_MISMATCH_ERROR, offset));
                return None;
            }
            StructFieldInformation::Declared(fields) => {
                for field_def in fields.iter() {
                    field_sig.push(instantiate(&field_def.signature.0, type_args));
                }
            }
        };
        Some(Signature(field_sig))
    }

    fn pack(
        &mut self,
        errors: &mut Vec<VMStatus>,
        offset: usize,
        struct_def: &StructDefinition,
        type_args: &Signature,
    ) {
        let struct_type = materialize_type(struct_def.struct_handle, type_args);
        if let Some(field_sig) = self.type_fields_signature(errors, offset, struct_def, type_args) {
            for sig in field_sig.0.iter().rev() {
                let arg = self.stack.pop().unwrap();
                if &arg.signature != sig {
                    errors.push(err_at_offset(StatusCode::PACK_TYPE_MISMATCH_ERROR, offset));
                }
            }
        }
        let kind = kind(self.module(), &struct_type, self.type_parameters());
        self.stack.push(TypedAbstractValue {
            signature: struct_type,
            value: AbstractValue::Value(kind),
        })
    }

    fn unpack(
        &mut self,
        errors: &mut Vec<VMStatus>,
        offset: usize,
        struct_def: &StructDefinition,
        type_args: &Signature,
    ) {
        let struct_type = materialize_type(struct_def.struct_handle, type_args);

        // Pop an abstract value from the stack and check if its type is equal to the one
        // declared. TODO: is it safe to not call verify the kinds if the types are equal?
        let arg = self.stack.pop().unwrap();
        if arg.signature != struct_type {
            errors.push(err_at_offset(
                StatusCode::UNPACK_TYPE_MISMATCH_ERROR,
                offset,
            ));
            return;
        }

        if let Some(field_sig) = self.type_fields_signature(errors, offset, struct_def, type_args) {
            for sig in field_sig.0.into_iter() {
                let kind = kind(self.module(), &sig, self.type_parameters());
                self.stack.push(TypedAbstractValue {
                    signature: sig,
                    value: AbstractValue::Value(kind),
                })
            }
        }
    }

    fn exists(&mut self, errors: &mut Vec<VMStatus>, offset: usize, struct_def: &StructDefinition) {
        if !StructDefinitionView::new(self.module(), struct_def).is_nominal_resource() {
            errors.push(err_at_offset(
                StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR,
                offset,
            ));
            return;
        }

        let operand = self.stack.pop().unwrap();
        if operand.signature == SignatureToken::Address {
            self.stack.push(TypedAbstractValue {
                signature: SignatureToken::Bool,
                value: AbstractValue::Value(Kind::Copyable),
            });
        } else {
            errors.push(err_at_offset(
                StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR,
                offset,
            ))
        }
    }

    pub fn move_from(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        offset: usize,
        def_idx: StructDefinitionIndex,
        type_args: &Signature,
    ) {
        let struct_def = self.module().struct_def_at(def_idx);
        if !StructDefinitionView::new(self.module(), struct_def).is_nominal_resource() {
            errors.push(err_at_offset(
                StatusCode::MOVEFROM_NO_RESOURCE_ERROR,
                offset,
            ));
            return;
        } else if state.is_global_borrowed(def_idx) {
            errors.push(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
            return;
        }

        let struct_type = materialize_type(struct_def.struct_handle, type_args);
        let operand = self.stack.pop().unwrap();
        if operand.signature == SignatureToken::Address {
            self.stack.push(TypedAbstractValue {
                signature: struct_type,
                value: AbstractValue::Value(Kind::Resource),
            });
        } else {
            errors.push(err_at_offset(
                StatusCode::MOVEFROM_TYPE_MISMATCH_ERROR,
                offset,
            ))
        }
    }

    pub fn move_to(
        &mut self,
        errors: &mut Vec<VMStatus>,
        offset: usize,
        struct_def: &StructDefinition,
        type_args: &Signature,
    ) {
        if !StructDefinitionView::new(self.module(), struct_def).is_nominal_resource() {
            errors.push(err_at_offset(
                StatusCode::MOVETOSENDER_NO_RESOURCE_ERROR,
                offset,
            ));
            return;
        }

        let struct_type = materialize_type(struct_def.struct_handle, type_args);
        let value_operand = self.stack.pop().unwrap();
        if value_operand.signature != struct_type {
            errors.push(err_at_offset(
                StatusCode::MOVETOSENDER_TYPE_MISMATCH_ERROR,
                offset,
            ))
        }
    }

    fn execute_inner(
        &mut self,
        errors: &mut Vec<VMStatus>,
        state: &mut AbstractState,
        bytecode: &Bytecode,
        offset: usize,
    ) {
        match bytecode {
            Bytecode::Pop => {
                let operand = self.stack.pop().unwrap();
                let kind = kind(self.module(), &operand.signature, self.type_parameters());
                if kind != Kind::Copyable {
                    errors.push(err_at_offset(StatusCode::POP_RESOURCE_ERROR, offset));
                    return;
                }

                if let AbstractValue::Reference(id) = operand.value {
                    state.release(id);
                }
            }

            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != SignatureToken::Bool {
                    errors.push(err_at_offset(StatusCode::BR_TYPE_MISMATCH_ERROR, offset))
                }
            }

            Bytecode::StLoc(idx) => {
                let operand = self.stack.pop().unwrap();
                if operand.signature != *self.locals_signature_view.token_at(*idx).as_inner() {
                    errors.push(err_at_offset(StatusCode::STLOC_TYPE_MISMATCH_ERROR, offset));
                    return;
                }
                if state.is_available(*idx) {
                    if state.is_local_safe_to_destroy(*idx) {
                        state.destroy_local(*idx);
                    } else {
                        errors.push(err_at_offset(
                            StatusCode::STLOC_UNSAFE_TO_DESTROY_ERROR,
                            offset,
                        ));
                        return;
                    }
                }
                state.insert_local(*idx, operand);
            }

            Bytecode::Abort => {
                let error_code = self.stack.pop().unwrap();
                if error_code.signature != SignatureToken::U64 {
                    errors.push(err_at_offset(StatusCode::ABORT_TYPE_MISMATCH_ERROR, offset));
                    return;
                }
                *state = AbstractState::default();
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
                    errors.push(err_at_offset(
                        StatusCode::RET_UNSAFE_TO_DESTROY_ERROR,
                        offset,
                    ));
                    return;
                }
                for return_type_view in self.function_definition_view.return_tokens().rev() {
                    let operand = self.stack.pop().unwrap();
                    if operand.signature != *return_type_view.as_inner() {
                        errors.push(err_at_offset(StatusCode::RET_TYPE_MISMATCH_ERROR, offset));
                        return;
                    }
                    if return_type_view.is_mutable_reference() {
                        if let AbstractValue::Reference(id) = operand.value {
                            if state.is_borrowed(id) {
                                errors.push(err_at_offset(
                                    StatusCode::RET_BORROWED_MUTABLE_REFERENCE_ERROR,
                                    offset,
                                ));
                                return;
                            }
                        }
                    }
                }
                *state = AbstractState::default();
            }

            Bytecode::Branch(_) | Bytecode::Nop => (),

            Bytecode::FreezeRef => {
                let operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = operand.signature {
                    let operand_id = operand.value.extract_id().unwrap();
                    if let Some(frozen_id) = state.freeze_ref(operand_id) {
                        self.stack.push(TypedAbstractValue {
                            signature: SignatureToken::Reference(signature),
                            value: AbstractValue::Reference(frozen_id),
                        });
                    } else {
                        errors.push(err_at_offset(
                            StatusCode::FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR,
                            offset,
                        ))
                    }
                } else {
                    errors.push(err_at_offset(
                        StatusCode::FREEZEREF_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::MutBorrowField(field_handle_index) => self.borrow_field(
                errors,
                state,
                offset,
                true,
                *field_handle_index,
                &Signature(vec![]),
            ),

            Bytecode::MutBorrowFieldGeneric(field_inst_index) => {
                let field_inst = self.module().field_instantiation_at(*field_inst_index);
                let type_inst = self.module().signature_at(field_inst.type_parameters);
                self.borrow_field(errors, state, offset, true, field_inst.handle, type_inst)
            }

            Bytecode::ImmBorrowField(field_handle_index) => self.borrow_field(
                errors,
                state,
                offset,
                false,
                *field_handle_index,
                &Signature(vec![]),
            ),

            Bytecode::ImmBorrowFieldGeneric(field_inst_index) => {
                let field_inst = self.module().field_instantiation_at(*field_inst_index);
                let type_inst = self.module().signature_at(field_inst.type_parameters);
                self.borrow_field(errors, state, offset, false, field_inst.handle, type_inst)
            }

            Bytecode::LdU8(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U8,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }

            Bytecode::LdU64(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U64,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }

            Bytecode::LdU128(_) => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::U128,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }

            Bytecode::LdConst(idx) => {
                let signature = self.module().constant_at(*idx).type_.clone();
                self.stack.push(TypedAbstractValue {
                    signature,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }

            Bytecode::LdTrue | Bytecode::LdFalse => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::Bool,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }

            Bytecode::CopyLoc(idx) => {
                let signature_view = self.locals_signature_view.token_at(*idx);
                if !state.is_available(*idx) {
                    errors.push(err_at_offset(StatusCode::COPYLOC_UNAVAILABLE_ERROR, offset))
                } else if signature_view.is_reference() {
                    let id =
                        state.borrow_local_reference(*idx, signature_view.is_mutable_reference());
                    self.stack.push(TypedAbstractValue {
                        signature: signature_view.as_inner().clone(),
                        value: AbstractValue::Reference(id),
                    });
                } else {
                    match kind(
                        self.module(),
                        signature_view.signature_token(),
                        self.type_parameters(),
                    ) {
                        Kind::Resource | Kind::All => {
                            errors.push(err_at_offset(StatusCode::COPYLOC_RESOURCE_ERROR, offset))
                        }
                        Kind::Copyable => {
                            if !state.is_local_mutably_borrowed(*idx) {
                                self.stack.push(TypedAbstractValue {
                                    signature: signature_view.as_inner().clone(),
                                    value: AbstractValue::Value(Kind::Copyable),
                                })
                            } else {
                                errors.push(err_at_offset(
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
                    errors.push(err_at_offset(StatusCode::MOVELOC_UNAVAILABLE_ERROR, offset))
                } else if signature.is_reference() || !state.is_local_borrowed(*idx) {
                    let value = state.remove_local(*idx);
                    self.stack.push(value);
                } else {
                    errors.push(err_at_offset(
                        StatusCode::MOVELOC_EXISTS_BORROW_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::MutBorrowLoc(idx) => self.borrow_loc(errors, state, offset, true, *idx),

            Bytecode::ImmBorrowLoc(idx) => self.borrow_loc(errors, state, offset, false, *idx),

            Bytecode::Call(idx) => {
                let function_handle = self.module().function_handle_at(*idx);
                self.call(errors, state, offset, function_handle, &Signature(vec![]))
            }

            Bytecode::CallGeneric(idx) => {
                let func_inst = self.module().function_instantiation_at(*idx);
                let func_handle = self.module().function_handle_at(func_inst.handle);
                let type_args = &self.module().signature_at(func_inst.type_parameters);
                self.call(errors, state, offset, func_handle, type_args)
            }

            Bytecode::Pack(idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                self.pack(errors, offset, struct_definition, &Signature(vec![]))
            }

            Bytecode::PackGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let struct_def = self.module().struct_def_at(struct_inst.def);
                let type_args = &self.module().signature_at(struct_inst.type_parameters);
                self.pack(errors, offset, struct_def, type_args)
            }

            Bytecode::Unpack(idx) => {
                let struct_definition = self.module().struct_def_at(*idx);
                self.unpack(errors, offset, struct_definition, &Signature(vec![]))
            }

            Bytecode::UnpackGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let struct_def = self.module().struct_def_at(struct_inst.def);
                let type_args = &self.module().signature_at(struct_inst.type_parameters);
                self.unpack(errors, offset, struct_def, type_args)
            }

            Bytecode::ReadRef => {
                let TypedAbstractValue {
                    signature: operand_signature,
                    value: operand_value,
                } = self.stack.pop().unwrap();
                if !operand_signature.is_reference() {
                    errors.push(err_at_offset(
                        StatusCode::READREF_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                    return;
                }
                let operand_id = operand_value.extract_id().unwrap();
                if !Self::is_readable_reference(state, &operand_signature, operand_id) {
                    errors.push(err_at_offset(
                        StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                        offset,
                    ))
                } else {
                    let inner_signature = *match operand_signature {
                        SignatureToken::Reference(signature) => signature,
                        SignatureToken::MutableReference(signature) => signature,
                        _ => panic!("Unreachable"),
                    };
                    if kind(self.module(), &inner_signature, self.type_parameters())
                        != Kind::Copyable
                    {
                        errors.push(err_at_offset(StatusCode::READREF_RESOURCE_ERROR, offset))
                    } else {
                        self.stack.push(TypedAbstractValue {
                            signature: inner_signature,
                            value: AbstractValue::Value(Kind::Copyable),
                        });
                        state.release(operand_id);
                    }
                }
            }

            Bytecode::WriteRef => {
                let ref_operand = self.stack.pop().unwrap();
                let val_operand = self.stack.pop().unwrap();
                if let SignatureToken::MutableReference(signature) = ref_operand.signature {
                    let kind = kind(self.module(), &signature, self.type_parameters());
                    match kind {
                        Kind::Resource | Kind::All => {
                            errors.push(err_at_offset(StatusCode::WRITEREF_RESOURCE_ERROR, offset))
                        }
                        Kind::Copyable => {
                            if val_operand.signature != *signature {
                                errors.push(err_at_offset(
                                    StatusCode::WRITEREF_TYPE_MISMATCH_ERROR,
                                    offset,
                                ))
                            } else {
                                let ref_operand_id = ref_operand.value.extract_id().unwrap();
                                if !state.is_borrowed(ref_operand_id) {
                                    state.release(ref_operand_id);
                                } else {
                                    errors.push(err_at_offset(
                                        StatusCode::WRITEREF_EXISTS_BORROW_ERROR,
                                        offset,
                                    ))
                                }
                            }
                        }
                    }
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Shl | Bytecode::Shr => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand2.signature.is_integer() && operand1.signature == SignatureToken::U8 {
                    self.stack.push(TypedAbstractValue {
                        signature: operand2.signature,
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
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
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
                        StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ))
                }
            }

            Bytecode::Eq | Bytecode::Neq => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                let kind1 = kind(self.module(), &operand1.signature, self.type_parameters());
                let is_copyable = kind1 == Kind::Copyable;
                if is_copyable && operand1.signature == operand2.signature {
                    if let (AbstractValue::Reference(id1), AbstractValue::Reference(id2)) =
                        (operand1.value, operand2.value)
                    {
                        if Self::is_readable_reference(state, &operand1.signature, id1)
                            && Self::is_readable_reference(state, &operand2.signature, id2)
                        {
                            state.release(id1);
                            state.release(id2);
                        } else {
                            errors.push(err_at_offset(
                                StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                                offset,
                            ));
                            return;
                        }
                    }
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
                        StatusCode::EQUALITY_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }
            }

            Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
                let operand1 = self.stack.pop().unwrap();
                let operand2 = self.stack.pop().unwrap();
                if operand1.signature.is_integer() && operand1.signature == operand2.signature {
                    self.stack.push(TypedAbstractValue {
                        signature: SignatureToken::Bool,
                        value: AbstractValue::Value(Kind::Copyable),
                    });
                } else {
                    errors.push(err_at_offset(
                        StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR,
                        offset,
                    ));
                }
            }

            Bytecode::MutBorrowGlobal(idx) => {
                self.borrow_global(errors, state, offset, true, *idx, &Signature(vec![]))
            }

            Bytecode::MutBorrowGlobalGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let type_inst = self.module().signature_at(struct_inst.type_parameters);
                self.borrow_global(errors, state, offset, true, struct_inst.def, type_inst)
            }

            Bytecode::ImmBorrowGlobal(idx) => {
                self.borrow_global(errors, state, offset, false, *idx, &Signature(vec![]))
            }

            Bytecode::ImmBorrowGlobalGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let type_inst = self.module().signature_at(struct_inst.type_parameters);
                self.borrow_global(errors, state, offset, false, struct_inst.def, type_inst)
            }

            Bytecode::Exists(idx) => {
                let struct_def = self.module().struct_def_at(*idx);
                self.exists(errors, offset, struct_def)
            }

            Bytecode::ExistsGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let struct_def = self.module().struct_def_at(struct_inst.def);
                self.exists(errors, offset, struct_def)
            }

            Bytecode::MoveFrom(idx) => {
                self.move_from(errors, state, offset, *idx, &Signature(vec![]))
            }

            Bytecode::MoveFromGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let type_args = &self.module().signature_at(struct_inst.type_parameters);
                self.move_from(errors, state, offset, struct_inst.def, type_args)
            }

            Bytecode::MoveToSender(idx) => {
                let struct_def = self.module().struct_def_at(*idx);
                self.move_to(errors, offset, struct_def, &Signature(vec![]))
            }

            Bytecode::MoveToSenderGeneric(idx) => {
                let struct_inst = self.module().struct_instantiation_at(*idx);
                let struct_def = self.module().struct_def_at(struct_inst.def);
                let type_args = &self.module().signature_at(struct_inst.type_parameters);
                self.move_to(errors, offset, struct_def, type_args)
            }

            Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnSequenceNumber
            | Bytecode::GetTxnPublicKey => {
                errors.push(
                    VMStatus::new(StatusCode::UNKNOWN_VERIFICATION_ERROR).with_message(format!(
                        "Bytecode {:?} is deprecated and will be removed soon",
                        bytecode
                    )),
                );
            }

            Bytecode::GetTxnSenderAddress => {
                self.stack.push(TypedAbstractValue {
                    signature: SignatureToken::Address,
                    value: AbstractValue::Value(Kind::Copyable),
                });
            }
        }
    }
}

impl<'a> TransferFunctions for TypeAndMemorySafetyAnalysis<'a> {
    type State = AbstractState;
    type AnalysisError = Vec<VMStatus>;

    fn execute(
        &mut self,
        state: &mut Self::State,
        bytecode: &Bytecode,
        index: usize,
        last_index: usize,
    ) -> Result<(), Self::AnalysisError> {
        let mut errors = vec![];
        self.execute_inner(&mut errors, state, bytecode, index);
        if errors.is_empty() {
            if index == last_index {
                *state = state.construct_canonical_state()
            }
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> AbstractInterpreter for TypeAndMemorySafetyAnalysis<'a> {}

fn materialize_type(struct_handle: StructHandleIndex, type_args: &Signature) -> SignatureToken {
    if type_args.is_empty() {
        SignatureToken::Struct(struct_handle)
    } else {
        SignatureToken::StructInstantiation(struct_handle, type_args.0.clone())
    }
}

fn instantiate(token: &SignatureToken, subst: &Signature) -> SignatureToken {
    use SignatureToken::*;

    match token {
        Bool => Bool,
        U8 => U8,
        U64 => U64,
        U128 => U128,
        Address => Address,
        Vector(ty) => Vector(Box::new(instantiate(ty, subst))),
        Struct(idx) => Struct(*idx),
        StructInstantiation(idx, struct_type_args) => StructInstantiation(
            *idx,
            struct_type_args
                .iter()
                .map(|ty| instantiate(ty, subst))
                .collect(),
        ),
        Reference(ty) => Reference(Box::new(instantiate(ty, subst))),
        MutableReference(ty) => MutableReference(Box::new(instantiate(ty, subst))),
        TypeParameter(idx) => {
            // Assume that the caller has previously parsed and verified the structure of the
            // file and that this guarantees that type parameter indices are always in bounds.
            assume!((*idx as usize) < subst.len());
            subst.0[*idx as usize].clone()
        }
    }
}
