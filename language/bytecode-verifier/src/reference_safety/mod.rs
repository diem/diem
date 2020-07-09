// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying reference safety of a procedure body.
//! The checks include (but are not limited to)
//! - verifying that there are no dangaling references,
//! - accesses to mutable references are safe
//! - accesses to global storage references are safe

mod abstract_state;

use crate::{
    absint::{AbstractInterpreter, BlockInvariant, BlockPostcondition, TransferFunctions},
    binary_views::{BinaryIndexedView, FunctionView},
};
use abstract_state::{AbstractState, AbstractValue};
use mirai_annotations::*;
use std::collections::{BTreeSet, HashMap};
use vm::{
    errors::{PartialVMError, PartialVMResult},
    file_format::{
        Bytecode, CodeOffset, FunctionDefinitionIndex, FunctionHandle, IdentifierIndex,
        SignatureToken, StructDefinition, StructFieldInformation,
    },
};

struct ReferenceSafetyAnalysis<'a> {
    resolver: &'a BinaryIndexedView<'a>,
    function_view: &'a FunctionView<'a>,
    name_def_map: &'a HashMap<IdentifierIndex, FunctionDefinitionIndex>,
    stack: Vec<AbstractValue>,
}

impl<'a> ReferenceSafetyAnalysis<'a> {
    fn new(
        resolver: &'a BinaryIndexedView<'a>,
        function_view: &'a FunctionView<'a>,
        name_def_map: &'a HashMap<IdentifierIndex, FunctionDefinitionIndex>,
    ) -> Self {
        Self {
            resolver,
            function_view,
            name_def_map,
            stack: vec![],
        }
    }
}

pub(crate) fn verify<'a>(
    resolver: &'a BinaryIndexedView<'a>,
    function_view: &FunctionView,
    name_def_map: &'a HashMap<IdentifierIndex, FunctionDefinitionIndex>,
) -> PartialVMResult<()> {
    let initial_state = AbstractState::new(function_view);

    let mut verifier = ReferenceSafetyAnalysis::new(resolver, function_view, name_def_map);
    let inv_map = verifier.analyze_function(initial_state, function_view);
    // Report all the join failures
    for (_block_id, BlockInvariant { post, .. }) in inv_map {
        match post {
            BlockPostcondition::Error(err) => return Err(err),
            // Block might be unprocessed if all predecessors had an error
            BlockPostcondition::Unprocessed | BlockPostcondition::Success => (),
        }
    }
    Ok(())
}

fn call(
    verifier: &mut ReferenceSafetyAnalysis,
    state: &mut AbstractState,
    offset: CodeOffset,
    function_handle: &FunctionHandle,
) -> PartialVMResult<()> {
    let parameters = verifier.resolver.signature_at(function_handle.parameters);
    let arguments = parameters
        .0
        .iter()
        .map(|_| verifier.stack.pop().unwrap())
        .rev()
        .collect();

    let acquired_resources = match verifier.name_def_map.get(&function_handle.name) {
        Some(idx) => {
            let func_def = verifier.resolver.function_def_at(*idx)?;
            let fh = verifier.resolver.function_handle_at(func_def.function);
            if function_handle == fh {
                func_def.acquires_global_resources.iter().cloned().collect()
            } else {
                BTreeSet::new()
            }
        }
        None => BTreeSet::new(),
    };
    let return_ = verifier.resolver.signature_at(function_handle.return_);
    let values = state.call(offset, arguments, &acquired_resources, return_)?;
    for value in values {
        verifier.stack.push(value)
    }
    Ok(())
}

fn num_fields(struct_def: &StructDefinition) -> usize {
    match &struct_def.field_information {
        StructFieldInformation::Native => 0,
        StructFieldInformation::Declared(fields) => fields.len(),
    }
}

fn pack(verifier: &mut ReferenceSafetyAnalysis, struct_def: &StructDefinition) {
    for _ in 0..num_fields(struct_def) {
        checked_verify!(verifier.stack.pop().unwrap().is_value())
    }
    // TODO maybe call state.value_for
    verifier.stack.push(AbstractValue::NonReference)
}

fn unpack(verifier: &mut ReferenceSafetyAnalysis, struct_def: &StructDefinition) {
    checked_verify!(verifier.stack.pop().unwrap().is_value());
    // TODO maybe call state.value_for
    for _ in 0..num_fields(struct_def) {
        verifier.stack.push(AbstractValue::NonReference)
    }
}

fn execute_inner(
    verifier: &mut ReferenceSafetyAnalysis,
    state: &mut AbstractState,
    bytecode: &Bytecode,
    offset: CodeOffset,
) -> PartialVMResult<()> {
    match bytecode {
        Bytecode::Pop => state.release_value(verifier.stack.pop().unwrap()),

        Bytecode::CopyLoc(local) => {
            let value = state.copy_loc(offset, *local)?;
            verifier.stack.push(value)
        }
        Bytecode::MoveLoc(local) => {
            let value = state.move_loc(offset, *local)?;
            verifier.stack.push(value)
        }
        Bytecode::StLoc(local) => state.st_loc(offset, *local, verifier.stack.pop().unwrap())?,

        Bytecode::FreezeRef => {
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let frozen = state.freeze_ref(offset, id)?;
            verifier.stack.push(frozen)
        }
        Bytecode::Eq | Bytecode::Neq => {
            let v1 = verifier.stack.pop().unwrap();
            let v2 = verifier.stack.pop().unwrap();
            let value = state.comparison(offset, v1, v2)?;
            verifier.stack.push(value)
        }
        Bytecode::ReadRef => {
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let value = state.read_ref(offset, id)?;
            verifier.stack.push(value)
        }
        Bytecode::WriteRef => {
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let val_operand = verifier.stack.pop().unwrap();
            checked_verify!(val_operand.is_value());
            state.write_ref(offset, id)?
        }

        Bytecode::MutBorrowLoc(local) => {
            let value = state.borrow_loc(offset, true, *local)?;
            verifier.stack.push(value)
        }
        Bytecode::ImmBorrowLoc(local) => {
            let value = state.borrow_loc(offset, false, *local)?;
            verifier.stack.push(value)
        }
        Bytecode::MutBorrowField(field_handle_index) => {
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let value = state.borrow_field(offset, true, id, *field_handle_index)?;
            verifier.stack.push(value)
        }
        Bytecode::MutBorrowFieldGeneric(field_inst_index) => {
            let field_inst = verifier
                .resolver
                .field_instantiation_at(*field_inst_index)?;
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let value = state.borrow_field(offset, true, id, field_inst.handle)?;
            verifier.stack.push(value)
        }
        Bytecode::ImmBorrowField(field_handle_index) => {
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let value = state.borrow_field(offset, false, id, *field_handle_index)?;
            verifier.stack.push(value)
        }
        Bytecode::ImmBorrowFieldGeneric(field_inst_index) => {
            let field_inst = verifier
                .resolver
                .field_instantiation_at(*field_inst_index)?;
            let id = verifier.stack.pop().unwrap().ref_id().unwrap();
            let value = state.borrow_field(offset, false, id, field_inst.handle)?;
            verifier.stack.push(value)
        }

        Bytecode::MutBorrowGlobal(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let value = state.borrow_global(offset, true, *idx)?;
            verifier.stack.push(value)
        }
        Bytecode::MutBorrowGlobalGeneric(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let value = state.borrow_global(offset, true, struct_inst.def)?;
            verifier.stack.push(value)
        }
        Bytecode::ImmBorrowGlobal(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let value = state.borrow_global(offset, false, *idx)?;
            verifier.stack.push(value)
        }
        Bytecode::ImmBorrowGlobalGeneric(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let value = state.borrow_global(offset, false, struct_inst.def)?;
            verifier.stack.push(value)
        }
        Bytecode::MoveFrom(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let value = state.move_from(offset, *idx)?;
            verifier.stack.push(value)
        }
        Bytecode::MoveFromGeneric(idx) => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let value = state.move_from(offset, struct_inst.def)?;
            verifier.stack.push(value)
        }

        Bytecode::Call(idx) => {
            let function_handle = verifier.resolver.function_handle_at(*idx);
            call(verifier, state, offset, function_handle)?
        }
        Bytecode::CallGeneric(idx) => {
            let func_inst = verifier.resolver.function_instantiation_at(*idx);
            let function_handle = verifier.resolver.function_handle_at(func_inst.handle);
            call(verifier, state, offset, function_handle)?
        }

        Bytecode::Ret => {
            let mut return_values = vec![];
            for _ in 0..verifier.function_view.return_().len() {
                return_values.push(verifier.stack.pop().unwrap());
            }
            return_values.reverse();

            state.ret(offset, return_values)?
        }

        Bytecode::Branch(_)
        | Bytecode::Nop
        | Bytecode::CastU8
        | Bytecode::CastU64
        | Bytecode::CastU128
        | Bytecode::Not
        | Bytecode::Exists(_)
        | Bytecode::ExistsGeneric(_) => (),

        Bytecode::BrTrue(_) | Bytecode::BrFalse(_) | Bytecode::Abort => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
        }
        Bytecode::MoveTo(_) | Bytecode::MoveToGeneric(_) => {
            // resource value
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            // signer reference
            state.release_value(verifier.stack.pop().unwrap());
        }

        Bytecode::LdTrue | Bytecode::LdFalse => {
            verifier.stack.push(state.value_for(&SignatureToken::Bool))
        }
        Bytecode::LdU8(_) => verifier.stack.push(state.value_for(&SignatureToken::U8)),
        Bytecode::LdU64(_) => verifier.stack.push(state.value_for(&SignatureToken::U64)),
        Bytecode::LdU128(_) => verifier.stack.push(state.value_for(&SignatureToken::U128)),
        Bytecode::LdConst(idx) => {
            let signature = &verifier.resolver.constant_at(*idx).type_;
            verifier.stack.push(state.value_for(signature))
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
        | Bytecode::Shr
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge => {
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            checked_verify!(verifier.stack.pop().unwrap().is_value());
            // TODO maybe call state.value_for
            verifier.stack.push(AbstractValue::NonReference)
        }

        Bytecode::Pack(idx) => {
            let struct_def = verifier.resolver.struct_def_at(*idx)?;
            pack(verifier, struct_def)
        }
        Bytecode::PackGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            pack(verifier, struct_def)
        }
        Bytecode::Unpack(idx) => {
            let struct_def = verifier.resolver.struct_def_at(*idx)?;
            unpack(verifier, struct_def)
        }
        Bytecode::UnpackGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            unpack(verifier, struct_def)
        }
    };
    Ok(())
}

impl<'a> TransferFunctions for ReferenceSafetyAnalysis<'a> {
    type State = AbstractState;
    type AnalysisError = PartialVMError;

    fn execute(
        &mut self,
        state: &mut Self::State,
        bytecode: &Bytecode,
        index: CodeOffset,
        last_index: CodeOffset,
    ) -> Result<(), Self::AnalysisError> {
        execute_inner(self, state, bytecode, index)?;
        if index == last_index {
            checked_verify!(self.stack.is_empty());
            *state = state.construct_canonical_state()
        }
        Ok(())
    }
}

impl<'a> AbstractInterpreter for ReferenceSafetyAnalysis<'a> {}
