// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{bounds_error, bytecode_offset_err, verification_error},
    file_format::{
        Bytecode, CompiledModuleMut, FieldDefinition, FunctionDefinition, FunctionHandle,
        FunctionSignature, LocalsSignature, ModuleHandle, SignatureToken, StructDefinition,
        StructFieldInformation, StructHandle, TypeSignature,
    },
    internals::ModuleIndex,
    IndexKind,
};
use types::vm_error::{StatusCode, VMStatus};

pub struct BoundsChecker<'a> {
    module: &'a CompiledModuleMut,
}

impl<'a> BoundsChecker<'a> {
    pub fn new(module: &'a CompiledModuleMut) -> Self {
        Self { module }
    }

    pub fn verify(self) -> Vec<VMStatus> {
        let mut errors: Vec<Vec<_>> = vec![];

        // A module (or script) must always have at least one module handle. (For modules the first
        // handle should be the same as the sender -- the bytecode verifier is unaware of
        // transactions so it does not perform this check.
        if self.module.module_handles.is_empty() {
            let status =
                verification_error(IndexKind::ModuleHandle, 0, StatusCode::NO_MODULE_HANDLES);
            errors.push(vec![status]);
        }

        errors.push(Self::verify_impl(
            IndexKind::ModuleHandle,
            self.module.module_handles.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::StructHandle,
            self.module.struct_handles.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::FunctionHandle,
            self.module.function_handles.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::StructDefinition,
            self.module.struct_defs.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::FieldDefinition,
            self.module.field_defs.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::FunctionDefinition,
            self.module.function_defs.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::TypeSignature,
            self.module.type_signatures.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::FunctionSignature,
            self.module.function_signatures.iter(),
            self.module,
        ));
        errors.push(Self::verify_impl(
            IndexKind::LocalsSignature,
            self.module.locals_signatures.iter(),
            self.module,
        ));

        let errors: Vec<_> = errors.into_iter().flatten().collect();
        if !errors.is_empty() {
            return errors;
        }

        // Code unit checking needs to be done once the rest of the module is validated.
        self.module
            .function_defs
            .iter()
            .enumerate()
            .map(|(idx, elem)| {
                elem.check_code_unit_bounds(self.module)
                    .into_iter()
                    .map(move |err| {
                        err.append_message(format!(
                            " at index {} while indexing {}",
                            idx,
                            IndexKind::FunctionDefinition
                        ))
                    })
            })
            .flatten()
            .collect()
    }

    #[inline]
    fn verify_impl(
        kind: IndexKind,
        iter: impl Iterator<Item = impl BoundsCheck>,
        module: &CompiledModuleMut,
    ) -> Vec<VMStatus> {
        iter.enumerate()
            .map(move |(idx, elem)| {
                elem.check_bounds(module).into_iter().map(move |err| {
                    err.append_message(format!(" at index {} while indexing {}", idx, kind))
                })
            })
            .flatten()
            .collect()
    }
}

pub trait BoundsCheck {
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus>;
}

#[inline]
fn check_bounds_impl<T, I>(pool: &[T], idx: I) -> Option<VMStatus>
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        let status = bounds_error(I::KIND, idx, len, StatusCode::INDEX_OUT_OF_BOUNDS);
        Some(status)
    } else {
        None
    }
}

#[inline]
fn check_code_unit_bounds_impl<T, I>(pool: &[T], bytecode_offset: usize, idx: I) -> Option<VMStatus>
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        let status = bytecode_offset_err(
            I::KIND,
            idx,
            len,
            bytecode_offset,
            StatusCode::INDEX_OUT_OF_BOUNDS,
        );
        Some(status)
    } else {
        None
    }
}

impl BoundsCheck for &ModuleHandle {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.address_pool, self.address),
            check_bounds_impl(&module.identifiers, self.name),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl BoundsCheck for &StructHandle {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.module_handles, self.module),
            check_bounds_impl(&module.identifiers, self.name),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl BoundsCheck for &FunctionHandle {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.module_handles, self.module),
            check_bounds_impl(&module.identifiers, self.name),
            check_bounds_impl(&module.function_signatures, self.signature),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl BoundsCheck for &StructDefinition {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.struct_handles, self.struct_handle),
            match &self.field_information {
                StructFieldInformation::Native => None,
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => module.check_field_range(*field_count, *fields),
            },
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl BoundsCheck for &FieldDefinition {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.struct_handles, self.struct_),
            check_bounds_impl(&module.identifiers, self.name),
            check_bounds_impl(&module.type_signatures, self.signature),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl BoundsCheck for &FunctionDefinition {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        vec![
            check_bounds_impl(&module.function_handles, self.function),
            if self.is_native() {
                None
            } else {
                check_bounds_impl(&module.locals_signatures, self.code.locals)
            },
        ]
        .into_iter()
        .flatten()
        .chain(
            self.acquires_global_resources
                .iter()
                .flat_map(|idx| check_bounds_impl(&module.struct_defs, *idx)),
        )
        .collect()
    }
}

impl BoundsCheck for &TypeSignature {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        self.0.check_bounds(module).into_iter().collect()
    }
}

impl BoundsCheck for &FunctionSignature {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        self.return_types
            .iter()
            .filter_map(|token| token.check_bounds(module))
            .chain(
                self.arg_types
                    .iter()
                    .filter_map(|token| token.check_bounds(module)),
            )
            .collect()
    }
}

impl BoundsCheck for &LocalsSignature {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        self.0
            .iter()
            .filter_map(|token| token.check_bounds(module))
            .collect()
    }
}

impl SignatureToken {
    #[inline]
    fn check_bounds(&self, module: &CompiledModuleMut) -> Option<VMStatus> {
        self.struct_index()
            .and_then(|sh_idx| check_bounds_impl(&module.struct_handles, sh_idx))
    }
}

impl FunctionDefinition {
    // This is implemented separately because it depends on the locals signature index being
    // checked.
    fn check_code_unit_bounds(&self, module: &CompiledModuleMut) -> Vec<VMStatus> {
        if self.is_native() {
            return vec![];
        }

        let locals_len = module.locals_signatures[self.code.locals.0 as usize]
            .0
            .len();

        let code = &self.code.code;
        let code_len = code.len();

        code.iter()
            .enumerate()
            .filter_map(|(bytecode_offset, bytecode)| {
                use self::Bytecode::*;

                match bytecode {
                    // Instructions that refer to other pools.
                    LdAddr(idx) => {
                        check_code_unit_bounds_impl(&module.address_pool, bytecode_offset, *idx)
                    }
                    LdByteArray(idx) => {
                        check_code_unit_bounds_impl(&module.byte_array_pool, bytecode_offset, *idx)
                    }
                    LdStr(idx) => {
                        check_code_unit_bounds_impl(&module.user_strings, bytecode_offset, *idx)
                    }
                    MutBorrowField(idx) | ImmBorrowField(idx) => {
                        check_code_unit_bounds_impl(&module.field_defs, bytecode_offset, *idx)
                    }
                    Call(idx, _) => {
                        check_code_unit_bounds_impl(&module.function_handles, bytecode_offset, *idx)
                    } // FIXME: check bounds for type actuals?
                    Pack(idx, _)
                    | Unpack(idx, _)
                    | Exists(idx, _)
                    | MutBorrowGlobal(idx, _)
                    | ImmBorrowGlobal(idx, _)
                    | MoveFrom(idx, _)
                    | MoveToSender(idx, _) => {
                        check_code_unit_bounds_impl(&module.struct_defs, bytecode_offset, *idx)
                    }
                    // Instructions that refer to this code block.
                    BrTrue(offset) | BrFalse(offset) | Branch(offset) => {
                        let offset = *offset as usize;
                        if offset >= code_len {
                            let status = bytecode_offset_err(
                                IndexKind::CodeDefinition,
                                offset,
                                code_len,
                                bytecode_offset,
                                StatusCode::INDEX_OUT_OF_BOUNDS,
                            );
                            Some(status)
                        } else {
                            None
                        }
                    }
                    // Instructions that refer to the locals.
                    CopyLoc(idx) | MoveLoc(idx) | StLoc(idx) | MutBorrowLoc(idx)
                    | ImmBorrowLoc(idx) => {
                        let idx = *idx as usize;
                        if idx >= locals_len {
                            let status = bytecode_offset_err(
                                IndexKind::LocalPool,
                                idx,
                                locals_len,
                                bytecode_offset,
                                StatusCode::INDEX_OUT_OF_BOUNDS,
                            );
                            Some(status)
                        } else {
                            None
                        }
                    }

                    // List out the other options explicitly so there's a compile error if a new
                    // bytecode gets added.
                    FreezeRef | Pop | Ret | LdConst(_) | LdTrue | LdFalse | ReadRef | WriteRef
                    | Add | Sub | Mul | Mod | Div | BitOr | BitAnd | Xor | Or | And | Not | Eq
                    | Neq | Lt | Gt | Le | Ge | Abort | GetTxnGasUnitPrice | GetTxnMaxGasUnits
                    | GetGasRemaining | GetTxnSenderAddress | CreateAccount
                    | GetTxnSequenceNumber | GetTxnPublicKey => None,
                }
            })
            .collect()
    }
}
