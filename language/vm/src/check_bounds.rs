// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{append_err_info, bounds_error, bytecode_offset_err, verification_error, VMResult},
    file_format::{
        Bytecode, CompiledModuleMut, Constant, FieldHandle, FieldInstantiation, FunctionDefinition,
        FunctionHandle, FunctionInstantiation, ModuleHandle, Signature, SignatureToken,
        StructDefInstantiation, StructDefinition, StructFieldInformation, StructHandle,
    },
    internals::ModuleIndex,
    IndexKind,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use std::u8;

pub struct BoundsChecker<'a> {
    module: &'a CompiledModuleMut,
}

impl<'a> BoundsChecker<'a> {
    pub fn new(module: &'a CompiledModuleMut) -> Self {
        Self { module }
    }

    pub fn verify(self) -> VMResult<()> {
        // TODO: this will not be true once we change CompiledScript and remove the
        // FunctionDefinition for `main`
        if self.module.module_handles.is_empty() {
            let status =
                verification_error(IndexKind::ModuleHandle, 0, StatusCode::NO_MODULE_HANDLES);
            return Err(status);
        }

        for signature in &self.module.signatures {
            self.check_signature(signature)?
        }
        for constant in &self.module.constant_pool {
            self.check_constant(constant)?
        }
        for module_handle in &self.module.module_handles {
            self.check_module_handle(module_handle)?
        }
        for struct_handle in &self.module.struct_handles {
            self.check_struct_handle(struct_handle)?
        }
        for function_handle in &self.module.function_handles {
            self.check_function_handle(function_handle)?
        }
        for field_handle in &self.module.field_handles {
            self.check_field_handle(field_handle)?
        }
        for struct_instantiation in &self.module.struct_def_instantiations {
            self.check_struct_instantiation(struct_instantiation)?
        }
        for function_instantiation in &self.module.function_instantiations {
            self.check_function_instantiation(function_instantiation)?
        }
        for field_instantiation in &self.module.field_instantiations {
            self.check_field_instantiation(field_instantiation)?
        }
        for struct_def in &self.module.struct_defs {
            self.check_struct_def(struct_def)?
        }
        for (function_def_idx, function_def) in self.module.function_defs.iter().enumerate() {
            self.check_function_def(function_def_idx, function_def)?
        }
        Ok(())
    }

    fn check_module_handle(&self, module_handle: &ModuleHandle) -> VMResult<()> {
        check_bounds_impl(&self.module.address_identifiers, module_handle.address)?;
        check_bounds_impl(&self.module.identifiers, module_handle.name)
    }

    fn check_struct_handle(&self, struct_handle: &StructHandle) -> VMResult<()> {
        check_bounds_impl(&self.module.module_handles, struct_handle.module)?;
        check_bounds_impl(&self.module.identifiers, struct_handle.name)
    }

    fn check_function_handle(&self, function_handle: &FunctionHandle) -> VMResult<()> {
        check_bounds_impl(&self.module.module_handles, function_handle.module)?;
        check_bounds_impl(&self.module.identifiers, function_handle.name)?;
        check_bounds_impl(&self.module.signatures, function_handle.parameters)?;
        check_bounds_impl(&self.module.signatures, function_handle.return_)?;
        // function signature type paramters must be in bounds to the function type parameters
        let type_param_count = function_handle.type_parameters.len();
        if let Some(sig) = self
            .module
            .signatures
            .get(function_handle.parameters.into_index())
        {
            for ty in &sig.0 {
                self.check_type_parameter(ty, type_param_count)?
            }
        }
        if let Some(sig) = self
            .module
            .signatures
            .get(function_handle.return_.into_index())
        {
            for ty in &sig.0 {
                self.check_type_parameter(ty, type_param_count)?
            }
        }
        Ok(())
    }

    fn check_field_handle(&self, field_handle: &FieldHandle) -> VMResult<()> {
        check_bounds_impl(&self.module.struct_defs, field_handle.owner)?;
        // field offset must be in bounds, struct def just checked above must exist
        if let Some(struct_def) = &self.module.struct_defs.get(field_handle.owner.into_index()) {
            let fields_count = match &struct_def.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            if field_handle.field as usize >= fields_count {
                return Err(bounds_error(
                    IndexKind::MemberCount,
                    field_handle.field as usize,
                    fields_count,
                    StatusCode::INDEX_OUT_OF_BOUNDS,
                ));
            }
        }
        Ok(())
    }

    fn check_struct_instantiation(
        &self,
        struct_instantiation: &StructDefInstantiation,
    ) -> VMResult<()> {
        check_bounds_impl(&self.module.struct_defs, struct_instantiation.def)?;
        check_bounds_impl(
            &self.module.signatures,
            struct_instantiation.type_parameters,
        )
    }

    fn check_function_instantiation(
        &self,
        function_instantiation: &FunctionInstantiation,
    ) -> VMResult<()> {
        check_bounds_impl(&self.module.function_handles, function_instantiation.handle)?;
        check_bounds_impl(
            &self.module.signatures,
            function_instantiation.type_parameters,
        )
    }

    fn check_field_instantiation(&self, field_instantiation: &FieldInstantiation) -> VMResult<()> {
        check_bounds_impl(&self.module.field_handles, field_instantiation.handle)?;
        check_bounds_impl(&self.module.signatures, field_instantiation.type_parameters)
    }

    fn check_signature(&self, signature: &Signature) -> VMResult<()> {
        for ty in &signature.0 {
            self.check_type(ty)?
        }
        Ok(())
    }

    fn check_constant(&self, constant: &Constant) -> VMResult<()> {
        self.check_type(&constant.type_)
    }

    fn check_struct_def(&self, struct_def: &StructDefinition) -> VMResult<()> {
        check_bounds_impl(&self.module.struct_handles, struct_def.struct_handle)?;
        // check signature (type) and type parameter for the field type
        if let StructFieldInformation::Declared(fields) = &struct_def.field_information {
            let type_param_count = self
                .module
                .struct_handles
                .get(struct_def.struct_handle.into_index())
                .map_or(0, |sh| sh.type_parameters.len());
            // field signatures are inlined
            for field in fields {
                check_bounds_impl(&self.module.identifiers, field.name)?;
                self.check_type(&field.signature.0)?;
                self.check_type_parameter(&field.signature.0, type_param_count)?;
            }
        }
        Ok(())
    }

    fn check_function_def(
        &self,
        function_def_idx: usize,
        function_def: &FunctionDefinition,
    ) -> VMResult<()> {
        check_bounds_impl(&self.module.function_handles, function_def.function)?;
        for ty in &function_def.acquires_global_resources {
            check_bounds_impl(&self.module.struct_defs, *ty)?;
        }
        self.check_code(function_def_idx, function_def)
    }

    fn check_code(
        &self,
        function_def_idx: usize,
        function_def: &FunctionDefinition,
    ) -> VMResult<()> {
        let code_unit = match &function_def.code {
            Some(code) => code,
            None => return Ok(()),
        };
        check_bounds_impl(&self.module.signatures, code_unit.locals)?;

        debug_assert!(function_def.function.into_index() < self.module.function_handles.len());
        let function_handle = &self.module.function_handles[function_def.function.into_index()];
        let type_param_count = function_handle.type_parameters.len();

        debug_assert!(function_handle.parameters.into_index() < self.module.signatures.len());
        let parameters = &self.module.signatures[function_handle.parameters.into_index()];

        let locals = &self
            .module
            .signatures
            .get(code_unit.locals.into_index())
            .as_ref()
            .unwrap()
            .0;

        // check if the number of parameters + locals is less than u8::MAX
        let locals_count = locals.len() + parameters.len();

        if locals_count > (u8::MAX as usize) + 1 {
            return Err(append_err_info(
                VMStatus::new(StatusCode::TOO_MANY_LOCALS),
                IndexKind::FunctionDefinition,
                function_def_idx,
            ));
        }

        // if there are locals check that the type parameters in local signature are in bounds.
        for local in locals {
            self.check_type_parameter(local, type_param_count)?
        }

        // check bytecodes
        let code_len = code_unit.code.len();
        for (bytecode_offset, bytecode) in code_unit.code.iter().enumerate() {
            use self::Bytecode::*;

            match bytecode {
                LdConst(idx) => {
                    check_code_unit_bounds_impl(&self.module.constant_pool, bytecode_offset, *idx)?
                }
                MutBorrowField(idx) | ImmBorrowField(idx) => {
                    check_code_unit_bounds_impl(&self.module.field_handles, bytecode_offset, *idx)?
                }
                MutBorrowFieldGeneric(idx) | ImmBorrowFieldGeneric(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.field_instantiations,
                        bytecode_offset,
                        *idx,
                    )?;
                    // check type parameters in borrow are bound to the function type parameters
                    if let Some(field_inst) = self.module.field_instantiations.get(idx.into_index())
                    {
                        if let Some(sig) = self
                            .module
                            .signatures
                            .get(field_inst.type_parameters.into_index())
                        {
                            for ty in &sig.0 {
                                self.check_type_parameter(ty, type_param_count)?
                            }
                        }
                    }
                }
                Call(idx) => check_code_unit_bounds_impl(
                    &self.module.function_handles,
                    bytecode_offset,
                    *idx,
                )?,
                CallGeneric(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.function_instantiations,
                        bytecode_offset,
                        *idx,
                    )?;
                    // check type parameters in call are bound to the function type parameters
                    if let Some(func_inst) =
                        self.module.function_instantiations.get(idx.into_index())
                    {
                        if let Some(sig) = self
                            .module
                            .signatures
                            .get(func_inst.type_parameters.into_index())
                        {
                            for ty in &sig.0 {
                                self.check_type_parameter(ty, type_param_count)?
                            }
                        }
                    }
                }
                Pack(idx) | Unpack(idx) | Exists(idx) | ImmBorrowGlobal(idx)
                | MutBorrowGlobal(idx) | MoveFrom(idx) | MoveToSender(idx) => {
                    check_code_unit_bounds_impl(&self.module.struct_defs, bytecode_offset, *idx)?
                }
                PackGeneric(idx)
                | UnpackGeneric(idx)
                | ExistsGeneric(idx)
                | ImmBorrowGlobalGeneric(idx)
                | MutBorrowGlobalGeneric(idx)
                | MoveFromGeneric(idx)
                | MoveToSenderGeneric(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.struct_def_instantiations,
                        bytecode_offset,
                        *idx,
                    )?;
                    // check type parameters in type operations are bound to the function type parameters
                    if let Some(struct_inst) =
                        self.module.struct_def_instantiations.get(idx.into_index())
                    {
                        if let Some(sig) = self
                            .module
                            .signatures
                            .get(struct_inst.type_parameters.into_index())
                        {
                            for ty in &sig.0 {
                                self.check_type_parameter(ty, type_param_count)?
                            }
                        }
                    }
                }
                // Instructions that refer to this code block.
                BrTrue(offset) | BrFalse(offset) | Branch(offset) => {
                    let offset = *offset as usize;
                    if offset >= code_len {
                        return Err(bytecode_offset_err(
                            IndexKind::CodeDefinition,
                            offset,
                            code_len,
                            bytecode_offset,
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                        ));
                    }
                }
                // Instructions that refer to the locals.
                CopyLoc(idx) | MoveLoc(idx) | StLoc(idx) | MutBorrowLoc(idx)
                | ImmBorrowLoc(idx) => {
                    let idx = *idx as usize;
                    if idx >= locals_count {
                        return Err(bytecode_offset_err(
                            IndexKind::LocalPool,
                            idx,
                            locals_count,
                            bytecode_offset,
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                        ));
                    }
                }

                // List out the other options explicitly so there's a compile error if a new
                // bytecode gets added.
                FreezeRef | Pop | Ret | LdU8(_) | LdU64(_) | LdU128(_) | CastU8 | CastU64
                | CastU128 | LdTrue | LdFalse | ReadRef | WriteRef | Add | Sub | Mul | Mod
                | Div | BitOr | BitAnd | Xor | Shl | Shr | Or | And | Not | Eq | Neq | Lt | Gt
                | Le | Ge | Abort | GetTxnSenderAddress | Nop => (),
            }
        }
        Ok(())
    }

    fn check_type(&self, ty: &SignatureToken) -> VMResult<()> {
        use self::SignatureToken::*;

        for ty in ty.preorder_traversal() {
            match ty {
                Bool | U8 | U64 | U128 | Address | TypeParameter(_) | Reference(_)
                | MutableReference(_) | Vector(_) => (),
                Struct(idx) => {
                    check_bounds_impl(&self.module.struct_handles, *idx)?;
                    if let Some(sh) = self.module.struct_handles.get(idx.into_index()) {
                        if !sh.type_parameters.is_empty() {
                            return Err(VMStatus::new(
                                StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
                            )
                            .with_message(format!(
                                "expected {} type parameters got 0 (Struct)",
                                sh.type_parameters.len(),
                            )));
                        }
                    }
                }
                StructInstantiation(idx, type_params) => {
                    check_bounds_impl(&self.module.struct_handles, *idx)?;
                    if let Some(sh) = self.module.struct_handles.get(idx.into_index()) {
                        if sh.type_parameters.len() != type_params.len() {
                            return Err(VMStatus::new(
                                StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
                            )
                            .with_message(format!(
                                "expected {} type parameters got {}",
                                sh.type_parameters.len(),
                                type_params.len(),
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn check_type_parameter(&self, ty: &SignatureToken, type_param_count: usize) -> VMResult<()> {
        use self::SignatureToken::*;

        for ty in ty.preorder_traversal() {
            match ty {
                SignatureToken::TypeParameter(idx) => {
                    if *idx as usize >= type_param_count {
                        return Err(bounds_error(
                            IndexKind::TypeParameter,
                            *idx as usize,
                            type_param_count,
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                        ));
                    }
                }

                Bool
                | U8
                | U64
                | U128
                | Address
                | Struct(_)
                | Reference(_)
                | MutableReference(_)
                | Vector(_)
                | StructInstantiation(_, _) => (),
            }
        }
        Ok(())
    }
}

fn check_bounds_impl<T, I>(pool: &[T], idx: I) -> VMResult<()>
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        Err(bounds_error(
            I::KIND,
            idx,
            len,
            StatusCode::INDEX_OUT_OF_BOUNDS,
        ))
    } else {
        Ok(())
    }
}

fn check_code_unit_bounds_impl<T, I>(pool: &[T], bytecode_offset: usize, idx: I) -> VMResult<()>
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        Err(bytecode_offset_err(
            I::KIND,
            idx,
            len,
            bytecode_offset,
            StatusCode::INDEX_OUT_OF_BOUNDS,
        ))
    } else {
        Ok(())
    }
}
