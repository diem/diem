// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type safety of a procedure body.
//! It does not utilize control flow, but does check each block independently

use crate::{
    binary_views::{BinaryIndexedView, FunctionView},
    control_flow_graph::ControlFlowGraph,
};
use libra_types::vm_status::StatusCode;
use mirai_annotations::*;
use vm::{
    errors::{PartialVMError, PartialVMResult},
    file_format::{
        Bytecode, CodeOffset, FieldHandleIndex, FunctionDefinitionIndex, FunctionHandle, Kind,
        LocalIndex, Signature, SignatureToken, SignatureToken as ST, StructDefinition,
        StructDefinitionIndex, StructFieldInformation, StructHandleIndex,
    },
};

struct Locals<'a> {
    param_count: usize,
    parameters: &'a Signature,
    locals: &'a Signature,
}

impl<'a> Locals<'a> {
    fn new(parameters: &'a Signature, locals: &'a Signature) -> Self {
        Self {
            param_count: parameters.len(),
            parameters,
            locals,
        }
    }

    fn local_at(&self, i: LocalIndex) -> &SignatureToken {
        let idx = i as usize;
        if idx < self.param_count {
            &self.parameters.0[idx]
        } else {
            &self.locals.0[idx - self.param_count]
        }
    }
}

struct TypeSafetyChecker<'a> {
    resolver: &'a BinaryIndexedView<'a>,
    function_view: &'a FunctionView<'a>,
    locals: Locals<'a>,
    stack: Vec<SignatureToken>,
}

impl<'a> TypeSafetyChecker<'a> {
    fn new(resolver: &'a BinaryIndexedView<'a>, function_view: &'a FunctionView<'a>) -> Self {
        let locals = Locals::new(function_view.parameters(), function_view.locals());
        Self {
            resolver,
            function_view,
            locals,
            stack: vec![],
        }
    }

    fn local_at(&self, i: LocalIndex) -> &SignatureToken {
        self.locals.local_at(i)
    }

    fn error(&self, status: StatusCode, offset: CodeOffset) -> PartialVMError {
        PartialVMError::new(status).at_code_offset(
            self.function_view
                .index()
                .unwrap_or(FunctionDefinitionIndex(0)),
            offset,
        )
    }
}

pub(crate) fn verify<'a>(
    resolver: &'a BinaryIndexedView<'a>,
    function_view: &'a FunctionView<'a>,
) -> PartialVMResult<()> {
    let verifier = &mut TypeSafetyChecker::new(resolver, function_view);

    for block_id in function_view.cfg().blocks() {
        for offset in function_view.cfg().instr_indexes(block_id) {
            let instr = &verifier.function_view.code().code[offset as usize];
            verify_instr(verifier, instr, offset)?
        }
    }

    Ok(())
}

// helper for both `ImmBorrowField` and `MutBorrowField`
fn borrow_field(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    mut_: bool,
    field_handle_index: FieldHandleIndex,
    type_args: &Signature,
) -> PartialVMResult<()> {
    // load operand and check mutability constraints
    let operand = verifier.stack.pop().unwrap();
    if mut_ && !operand.is_mutable_reference() {
        return Err(verifier.error(StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR, offset));
    }

    // check the reference on the stack is the expected type.
    // Load the type that owns the field according to the instruction.
    // For generic fields access, this step materializes that type
    let field_handle = verifier.resolver.field_handle_at(field_handle_index)?;
    let struct_def = verifier.resolver.struct_def_at(field_handle.owner)?;
    let expected_type = materialize_type(struct_def.struct_handle, type_args);
    match operand {
        ST::Reference(inner) | ST::MutableReference(inner) if expected_type == *inner => (),
        _ => return Err(verifier.error(StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR, offset)),
    }

    let field_def = match &struct_def.field_information {
        StructFieldInformation::Native => {
            return Err(verifier.error(StatusCode::BORROWFIELD_BAD_FIELD_ERROR, offset));
        }
        StructFieldInformation::Declared(fields) => {
            // TODO: review the whole error story here, way too much is left to chances...
            // definition of a more proper OM for the verifier could work around the problem
            // (maybe, maybe not..)
            &fields[field_handle.field as usize]
        }
    };
    let field_type = Box::new(instantiate(&field_def.signature.0, type_args));
    verifier.stack.push(if mut_ {
        ST::MutableReference(field_type)
    } else {
        ST::Reference(field_type)
    });
    Ok(())
}

// helper for both `ImmBorrowLoc` and `MutBorrowLoc`
fn borrow_loc(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    mut_: bool,
    idx: LocalIndex,
) -> PartialVMResult<()> {
    let loc_signature = verifier.local_at(idx).clone();

    if loc_signature.is_reference() {
        return Err(verifier.error(StatusCode::BORROWLOC_REFERENCE_ERROR, offset));
    }

    verifier.stack.push(if mut_ {
        ST::MutableReference(Box::new(loc_signature))
    } else {
        ST::Reference(Box::new(loc_signature))
    });
    Ok(())
}

fn borrow_global(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    mut_: bool,
    idx: StructDefinitionIndex,
    type_args: &Signature,
) -> PartialVMResult<()> {
    // check and consume top of stack
    let operand = verifier.stack.pop().unwrap();
    if operand != ST::Address {
        return Err(verifier.error(StatusCode::BORROWGLOBAL_TYPE_MISMATCH_ERROR, offset));
    }

    let struct_def = verifier.resolver.struct_def_at(idx)?;
    let struct_handle = verifier.resolver.struct_handle_at(struct_def.struct_handle);
    if !struct_handle.is_nominal_resource {
        return Err(verifier.error(StatusCode::BORROWGLOBAL_NO_RESOURCE_ERROR, offset));
    }

    let struct_type = materialize_type(struct_def.struct_handle, type_args);
    verifier.stack.push(if mut_ {
        ST::MutableReference(Box::new(struct_type))
    } else {
        ST::Reference(Box::new(struct_type))
    });
    Ok(())
}

fn call(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    function_handle: &FunctionHandle,
    type_actuals: &Signature,
) -> PartialVMResult<()> {
    let parameters = verifier.resolver.signature_at(function_handle.parameters);
    for parameter in parameters.0.iter().rev() {
        let arg = verifier.stack.pop().unwrap();
        if arg != instantiate(parameter, type_actuals) {
            return Err(verifier.error(StatusCode::CALL_TYPE_MISMATCH_ERROR, offset));
        }
    }
    for return_type in &verifier.resolver.signature_at(function_handle.return_).0 {
        verifier.stack.push(instantiate(return_type, type_actuals))
    }
    Ok(())
}

fn type_fields_signature(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    struct_def: &StructDefinition,
    type_args: &Signature,
) -> PartialVMResult<Signature> {
    match &struct_def.field_information {
        StructFieldInformation::Native => {
            // TODO: this is more of "unreachable"
            Err(verifier.error(StatusCode::PACK_TYPE_MISMATCH_ERROR, offset))
        }
        StructFieldInformation::Declared(fields) => {
            let mut field_sig = vec![];
            for field_def in fields.iter() {
                field_sig.push(instantiate(&field_def.signature.0, type_args));
            }
            Ok(Signature(field_sig))
        }
    }
}

fn pack(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    struct_def: &StructDefinition,
    type_args: &Signature,
) -> PartialVMResult<()> {
    let struct_type = materialize_type(struct_def.struct_handle, type_args);
    let field_sig = type_fields_signature(verifier, offset, struct_def, type_args)?;
    for sig in field_sig.0.iter().rev() {
        let arg = verifier.stack.pop().unwrap();
        if &arg != sig {
            return Err(verifier.error(StatusCode::PACK_TYPE_MISMATCH_ERROR, offset));
        }
    }

    verifier.stack.push(struct_type);
    Ok(())
}

fn unpack(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    struct_def: &StructDefinition,
    type_args: &Signature,
) -> PartialVMResult<()> {
    let struct_type = materialize_type(struct_def.struct_handle, type_args);

    // Pop an abstract value from the stack and check if its type is equal to the one
    // declared. TODO: is it safe to not call verify the kinds if the types are equal?
    let arg = verifier.stack.pop().unwrap();
    if arg != struct_type {
        return Err(verifier.error(StatusCode::UNPACK_TYPE_MISMATCH_ERROR, offset));
    }

    let field_sig = type_fields_signature(verifier, offset, struct_def, type_args)?;
    for sig in field_sig.0 {
        verifier.stack.push(sig)
    }
    Ok(())
}

fn exists(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    struct_def: &StructDefinition,
) -> PartialVMResult<()> {
    if !verifier
        .resolver
        .struct_handle_at(struct_def.struct_handle)
        .is_nominal_resource
    {
        return Err(verifier.error(StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR, offset));
    }

    let operand = verifier.stack.pop().unwrap();
    if operand != ST::Address {
        // TODO better error here
        return Err(verifier.error(StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR, offset));
    }

    verifier.stack.push(ST::Bool);
    Ok(())
}

fn move_from(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    def_idx: StructDefinitionIndex,
    type_args: &Signature,
) -> PartialVMResult<()> {
    let struct_def = verifier.resolver.struct_def_at(def_idx)?;
    if !verifier
        .resolver
        .struct_handle_at(struct_def.struct_handle)
        .is_nominal_resource
    {
        return Err(verifier.error(StatusCode::MOVEFROM_NO_RESOURCE_ERROR, offset));
    }

    let struct_type = materialize_type(struct_def.struct_handle, type_args);
    let operand = verifier.stack.pop().unwrap();
    if operand != ST::Address {
        return Err(verifier.error(StatusCode::MOVEFROM_TYPE_MISMATCH_ERROR, offset));
    }

    verifier.stack.push(struct_type);
    Ok(())
}

fn move_to(
    verifier: &mut TypeSafetyChecker,
    offset: CodeOffset,
    struct_def: &StructDefinition,
    type_args: &Signature,
) -> PartialVMResult<()> {
    if !verifier
        .resolver
        .struct_handle_at(struct_def.struct_handle)
        .is_nominal_resource
    {
        return Err(verifier.error(StatusCode::MOVETO_NO_RESOURCE_ERROR, offset));
    }

    let struct_type = materialize_type(struct_def.struct_handle, type_args);
    let resource_operand = verifier.stack.pop().unwrap();
    let signer_reference_operand = verifier.stack.pop().unwrap();
    if resource_operand != struct_type {
        return Err(verifier.error(StatusCode::MOVETO_TYPE_MISMATCH_ERROR, offset));
    }
    match signer_reference_operand {
        ST::Reference(inner) => match *inner {
            ST::Signer => Ok(()),
            _ => Err(verifier.error(StatusCode::MOVETO_TYPE_MISMATCH_ERROR, offset)),
        },
        _ => Err(verifier.error(StatusCode::MOVETO_TYPE_MISMATCH_ERROR, offset)),
    }
}

fn verify_instr(
    verifier: &mut TypeSafetyChecker,
    bytecode: &Bytecode,
    offset: CodeOffset,
) -> PartialVMResult<()> {
    match bytecode {
        Bytecode::Pop => {
            let operand = verifier.stack.pop().unwrap();
            let kind = verifier
                .resolver
                .kind(&operand, verifier.function_view.type_parameters());
            if kind != Kind::Copyable {
                return Err(verifier.error(StatusCode::POP_RESOURCE_ERROR, offset));
            }
        }

        Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
            let operand = verifier.stack.pop().unwrap();
            if operand != ST::Bool {
                return Err(verifier.error(StatusCode::BR_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::StLoc(idx) => {
            let operand = verifier.stack.pop().unwrap();
            if &operand != verifier.local_at(*idx) {
                return Err(verifier.error(StatusCode::STLOC_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Abort => {
            let operand = verifier.stack.pop().unwrap();
            if operand != ST::U64 {
                return Err(verifier.error(StatusCode::ABORT_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Ret => {
            let return_ = &verifier.function_view.return_().0;
            for return_type in return_.iter().rev() {
                let operand = verifier.stack.pop().unwrap();
                if &operand != return_type {
                    return Err(verifier.error(StatusCode::RET_TYPE_MISMATCH_ERROR, offset));
                }
            }
        }

        Bytecode::Branch(_) | Bytecode::Nop => (),

        Bytecode::FreezeRef => {
            let operand = verifier.stack.pop().unwrap();
            match operand {
                ST::MutableReference(inner) => verifier.stack.push(ST::Reference(inner)),
                _ => return Err(verifier.error(StatusCode::FREEZEREF_TYPE_MISMATCH_ERROR, offset)),
            }
        }

        Bytecode::MutBorrowField(field_handle_index) => borrow_field(
            verifier,
            offset,
            true,
            *field_handle_index,
            &Signature(vec![]),
        )?,

        Bytecode::MutBorrowFieldGeneric(field_inst_index) => {
            let field_inst = verifier
                .resolver
                .field_instantiation_at(*field_inst_index)?;
            let type_inst = verifier.resolver.signature_at(field_inst.type_parameters);
            borrow_field(verifier, offset, true, field_inst.handle, type_inst)?
        }

        Bytecode::ImmBorrowField(field_handle_index) => borrow_field(
            verifier,
            offset,
            false,
            *field_handle_index,
            &Signature(vec![]),
        )?,

        Bytecode::ImmBorrowFieldGeneric(field_inst_index) => {
            let field_inst = verifier
                .resolver
                .field_instantiation_at(*field_inst_index)?;
            let type_inst = verifier.resolver.signature_at(field_inst.type_parameters);
            borrow_field(verifier, offset, false, field_inst.handle, type_inst)?
        }

        Bytecode::LdU8(_) => {
            verifier.stack.push(ST::U8);
        }

        Bytecode::LdU64(_) => {
            verifier.stack.push(ST::U64);
        }

        Bytecode::LdU128(_) => {
            verifier.stack.push(ST::U128);
        }

        Bytecode::LdConst(idx) => {
            let signature = verifier.resolver.constant_at(*idx).type_.clone();
            verifier.stack.push(signature);
        }

        Bytecode::LdTrue | Bytecode::LdFalse => {
            verifier.stack.push(ST::Bool);
        }

        Bytecode::CopyLoc(idx) => {
            let local_signature = verifier.local_at(*idx).clone();
            match verifier
                .resolver
                .kind(&local_signature, verifier.function_view.type_parameters())
            {
                Kind::Resource | Kind::All => {
                    return Err(verifier.error(StatusCode::COPYLOC_RESOURCE_ERROR, offset))
                }
                Kind::Copyable => verifier.stack.push(local_signature),
            }
        }

        Bytecode::MoveLoc(idx) => {
            let local_signature = verifier.local_at(*idx).clone();
            verifier.stack.push(local_signature)
        }

        Bytecode::MutBorrowLoc(idx) => borrow_loc(verifier, offset, true, *idx)?,

        Bytecode::ImmBorrowLoc(idx) => borrow_loc(verifier, offset, false, *idx)?,

        Bytecode::Call(idx) => {
            let function_handle = verifier.resolver.function_handle_at(*idx);
            call(verifier, offset, function_handle, &Signature(vec![]))?
        }

        Bytecode::CallGeneric(idx) => {
            let func_inst = verifier.resolver.function_instantiation_at(*idx);
            let func_handle = verifier.resolver.function_handle_at(func_inst.handle);
            let type_args = &verifier.resolver.signature_at(func_inst.type_parameters);
            call(verifier, offset, func_handle, type_args)?
        }

        Bytecode::Pack(idx) => {
            let struct_definition = verifier.resolver.struct_def_at(*idx)?;
            pack(verifier, offset, struct_definition, &Signature(vec![]))?
        }

        Bytecode::PackGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            let type_args = verifier.resolver.signature_at(struct_inst.type_parameters);
            pack(verifier, offset, struct_def, type_args)?
        }

        Bytecode::Unpack(idx) => {
            let struct_definition = verifier.resolver.struct_def_at(*idx)?;
            unpack(verifier, offset, struct_definition, &Signature(vec![]))?
        }

        Bytecode::UnpackGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            let type_args = verifier.resolver.signature_at(struct_inst.type_parameters);
            unpack(verifier, offset, struct_def, type_args)?
        }

        Bytecode::ReadRef => {
            let operand = verifier.stack.pop().unwrap();
            match operand {
                ST::Reference(inner) | ST::MutableReference(inner) => {
                    let kind = verifier
                        .resolver
                        .kind(&inner, verifier.function_view.type_parameters());
                    if kind != Kind::Copyable {
                        return Err(verifier.error(StatusCode::READREF_RESOURCE_ERROR, offset));
                    }
                    verifier.stack.push(*inner);
                }
                _ => return Err(verifier.error(StatusCode::READREF_TYPE_MISMATCH_ERROR, offset)),
            }
        }

        Bytecode::WriteRef => {
            let ref_operand = verifier.stack.pop().unwrap();
            let val_operand = verifier.stack.pop().unwrap();
            let ref_inner_signature = match ref_operand {
                ST::MutableReference(inner) => *inner,
                _ => {
                    return Err(
                        verifier.error(StatusCode::WRITEREF_NO_MUTABLE_REFERENCE_ERROR, offset)
                    )
                }
            };
            let kind = verifier.resolver.kind(
                &ref_inner_signature,
                verifier.function_view.type_parameters(),
            );
            match kind {
                Kind::Resource | Kind::All => {
                    return Err(verifier.error(StatusCode::WRITEREF_RESOURCE_ERROR, offset))
                }
                Kind::Copyable => (),
            }
            if val_operand != ref_inner_signature {
                return Err(verifier.error(StatusCode::WRITEREF_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::CastU8 => {
            let operand = verifier.stack.pop().unwrap();
            if !operand.is_integer() {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
            verifier.stack.push(ST::U8);
        }
        Bytecode::CastU64 => {
            let operand = verifier.stack.pop().unwrap();
            if !operand.is_integer() {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
            verifier.stack.push(ST::U64);
        }
        Bytecode::CastU128 => {
            let operand = verifier.stack.pop().unwrap();
            if !operand.is_integer() {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
            verifier.stack.push(ST::U128);
        }

        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor => {
            let operand1 = verifier.stack.pop().unwrap();
            let operand2 = verifier.stack.pop().unwrap();
            if operand1.is_integer() && operand1 == operand2 {
                verifier.stack.push(operand1);
            } else {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Shl | Bytecode::Shr => {
            let operand1 = verifier.stack.pop().unwrap();
            let operand2 = verifier.stack.pop().unwrap();
            if operand2.is_integer() && operand1 == ST::U8 {
                verifier.stack.push(operand2);
            } else {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Or | Bytecode::And => {
            let operand1 = verifier.stack.pop().unwrap();
            let operand2 = verifier.stack.pop().unwrap();
            if operand1 == ST::Bool && operand2 == ST::Bool {
                verifier.stack.push(ST::Bool);
            } else {
                return Err(verifier.error(StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Not => {
            let operand = verifier.stack.pop().unwrap();
            if operand == ST::Bool {
                verifier.stack.push(ST::Bool);
            } else {
                return Err(verifier.error(StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Eq | Bytecode::Neq => {
            let operand1 = verifier.stack.pop().unwrap();
            let operand2 = verifier.stack.pop().unwrap();
            let kind1 = verifier
                .resolver
                .kind(&operand1, verifier.function_view.type_parameters());
            let is_copyable = kind1 == Kind::Copyable;
            if is_copyable && operand1 == operand2 {
                verifier.stack.push(ST::Bool);
            } else {
                return Err(verifier.error(StatusCode::EQUALITY_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
            let operand1 = verifier.stack.pop().unwrap();
            let operand2 = verifier.stack.pop().unwrap();
            if operand1.is_integer() && operand1 == operand2 {
                verifier.stack.push(ST::Bool)
            } else {
                return Err(verifier.error(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR, offset));
            }
        }

        Bytecode::MutBorrowGlobal(idx) => {
            borrow_global(verifier, offset, true, *idx, &Signature(vec![]))?
        }

        Bytecode::MutBorrowGlobalGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let type_inst = verifier.resolver.signature_at(struct_inst.type_parameters);
            borrow_global(verifier, offset, true, struct_inst.def, type_inst)?
        }

        Bytecode::ImmBorrowGlobal(idx) => {
            borrow_global(verifier, offset, false, *idx, &Signature(vec![]))?
        }

        Bytecode::ImmBorrowGlobalGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let type_inst = verifier.resolver.signature_at(struct_inst.type_parameters);
            borrow_global(verifier, offset, false, struct_inst.def, type_inst)?
        }

        Bytecode::Exists(idx) => {
            let struct_def = verifier.resolver.struct_def_at(*idx)?;
            exists(verifier, offset, struct_def)?
        }

        Bytecode::ExistsGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            exists(verifier, offset, struct_def)?
        }

        Bytecode::MoveFrom(idx) => move_from(verifier, offset, *idx, &Signature(vec![]))?,

        Bytecode::MoveFromGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let type_args = verifier.resolver.signature_at(struct_inst.type_parameters);
            move_from(verifier, offset, struct_inst.def, type_args)?
        }

        Bytecode::MoveTo(idx) => {
            let struct_def = verifier.resolver.struct_def_at(*idx)?;
            move_to(verifier, offset, struct_def, &Signature(vec![]))?
        }

        Bytecode::MoveToGeneric(idx) => {
            let struct_inst = verifier.resolver.struct_instantiation_at(*idx)?;
            let struct_def = verifier.resolver.struct_def_at(struct_inst.def)?;
            let type_args = verifier.resolver.signature_at(struct_inst.type_parameters);
            move_to(verifier, offset, struct_def, type_args)?
        }
    };
    Ok(())
}

//
// Helpers functions for types
//

fn materialize_type(struct_handle: StructHandleIndex, type_args: &Signature) -> SignatureToken {
    if type_args.is_empty() {
        ST::Struct(struct_handle)
    } else {
        ST::StructInstantiation(struct_handle, type_args.0.clone())
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
        Signer => Signer,
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
