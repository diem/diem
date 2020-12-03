// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{
        bounds_error, offset_out_of_bounds as offset_out_of_bounds_error, verification_error,
        PartialVMError, PartialVMResult,
    },
    file_format::{
        Bytecode, CodeOffset, CompiledModuleMut, Constant, FieldHandle, FieldInstantiation,
        FunctionDefinition, FunctionDefinitionIndex, FunctionHandle, FunctionInstantiation,
        ModuleHandle, Signature, SignatureToken, StructDefInstantiation, StructDefinition,
        StructFieldInformation, StructHandle, TableIndex,
    },
    internals::ModuleIndex,
    IndexKind,
};
use move_core_types::vm_status::StatusCode;
use std::u8;

pub struct BoundsChecker<'a> {
    module: &'a CompiledModuleMut,
    current_function: Option<FunctionDefinitionIndex>,
}

impl<'a> BoundsChecker<'a> {
    pub fn verify(module: &'a CompiledModuleMut) -> PartialVMResult<()> {
        let mut bounds_check = Self {
            module,
            current_function: None,
        };
        // TODO: this will not be true once we change CompiledScript and remove the
        // FunctionDefinition for `main`
        if bounds_check.module.module_handles.is_empty() {
            let status =
                verification_error(StatusCode::NO_MODULE_HANDLES, IndexKind::ModuleHandle, 0);
            return Err(status);
        }

        for signature in &bounds_check.module.signatures {
            bounds_check.check_signature(signature)?
        }
        for constant in &bounds_check.module.constant_pool {
            bounds_check.check_constant(constant)?
        }
        for module_handle in &bounds_check.module.module_handles {
            bounds_check.check_module_handle(module_handle)?
        }
        for struct_handle in &bounds_check.module.struct_handles {
            bounds_check.check_struct_handle(struct_handle)?
        }
        for function_handle in &bounds_check.module.function_handles {
            bounds_check.check_function_handle(function_handle)?
        }
        for field_handle in &bounds_check.module.field_handles {
            bounds_check.check_field_handle(field_handle)?
        }
        for struct_instantiation in &bounds_check.module.struct_def_instantiations {
            bounds_check.check_struct_instantiation(struct_instantiation)?
        }
        for function_instantiation in &bounds_check.module.function_instantiations {
            bounds_check.check_function_instantiation(function_instantiation)?
        }
        for field_instantiation in &bounds_check.module.field_instantiations {
            bounds_check.check_field_instantiation(field_instantiation)?
        }
        for struct_def in &bounds_check.module.struct_defs {
            bounds_check.check_struct_def(struct_def)?
        }
        for (function_def_idx, function_def) in bounds_check.module.function_defs.iter().enumerate()
        {
            bounds_check.check_function_def(function_def_idx, function_def)?
        }
        Ok(())
    }

    fn check_module_handle(&self, module_handle: &ModuleHandle) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.address_identifiers, module_handle.address)?;
        check_bounds_impl(&self.module.identifiers, module_handle.name)
    }

    fn check_struct_handle(&self, struct_handle: &StructHandle) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.module_handles, struct_handle.module)?;
        check_bounds_impl(&self.module.identifiers, struct_handle.name)
    }

    fn check_function_handle(&self, function_handle: &FunctionHandle) -> PartialVMResult<()> {
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

    fn check_field_handle(&self, field_handle: &FieldHandle) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.struct_defs, field_handle.owner)?;
        // field offset must be in bounds, struct def just checked above must exist
        if let Some(struct_def) = &self.module.struct_defs.get(field_handle.owner.into_index()) {
            let fields_count = match &struct_def.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            if field_handle.field as usize >= fields_count {
                return Err(bounds_error(
                    StatusCode::INDEX_OUT_OF_BOUNDS,
                    IndexKind::MemberCount,
                    field_handle.field,
                    fields_count,
                ));
            }
        }
        Ok(())
    }

    fn check_struct_instantiation(
        &self,
        struct_instantiation: &StructDefInstantiation,
    ) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.struct_defs, struct_instantiation.def)?;
        check_bounds_impl(
            &self.module.signatures,
            struct_instantiation.type_parameters,
        )
    }

    fn check_function_instantiation(
        &self,
        function_instantiation: &FunctionInstantiation,
    ) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.function_handles, function_instantiation.handle)?;
        check_bounds_impl(
            &self.module.signatures,
            function_instantiation.type_parameters,
        )
    }

    fn check_field_instantiation(
        &self,
        field_instantiation: &FieldInstantiation,
    ) -> PartialVMResult<()> {
        check_bounds_impl(&self.module.field_handles, field_instantiation.handle)?;
        check_bounds_impl(&self.module.signatures, field_instantiation.type_parameters)
    }

    fn check_signature(&self, signature: &Signature) -> PartialVMResult<()> {
        for ty in &signature.0 {
            self.check_type(ty)?
        }
        Ok(())
    }

    fn check_constant(&self, constant: &Constant) -> PartialVMResult<()> {
        self.check_type(&constant.type_)
    }

    fn check_struct_def(&self, struct_def: &StructDefinition) -> PartialVMResult<()> {
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
        &mut self,
        function_def_idx: usize,
        function_def: &FunctionDefinition,
    ) -> PartialVMResult<()> {
        self.current_function = Some(FunctionDefinitionIndex(function_def_idx as TableIndex));
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
    ) -> PartialVMResult<()> {
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
            return Err(verification_error(
                StatusCode::TOO_MANY_LOCALS,
                IndexKind::FunctionDefinition,
                function_def_idx as TableIndex,
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
                LdConst(idx) => self.check_code_unit_bounds_impl(
                    &self.module.constant_pool,
                    *idx,
                    bytecode_offset,
                )?,
                MutBorrowField(idx) | ImmBorrowField(idx) => self.check_code_unit_bounds_impl(
                    &self.module.field_handles,
                    *idx,
                    bytecode_offset,
                )?,
                MutBorrowFieldGeneric(idx) | ImmBorrowFieldGeneric(idx) => {
                    self.check_code_unit_bounds_impl(
                        &self.module.field_instantiations,
                        *idx,
                        bytecode_offset,
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
                Call(idx) => self.check_code_unit_bounds_impl(
                    &self.module.function_handles,
                    *idx,
                    bytecode_offset,
                )?,
                CallGeneric(idx) => {
                    self.check_code_unit_bounds_impl(
                        &self.module.function_instantiations,
                        *idx,
                        bytecode_offset,
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
                | MutBorrowGlobal(idx) | MoveFrom(idx) | MoveTo(idx) => self
                    .check_code_unit_bounds_impl(&self.module.struct_defs, *idx, bytecode_offset)?,
                PackGeneric(idx)
                | UnpackGeneric(idx)
                | ExistsGeneric(idx)
                | ImmBorrowGlobalGeneric(idx)
                | MutBorrowGlobalGeneric(idx)
                | MoveFromGeneric(idx)
                | MoveToGeneric(idx) => {
                    self.check_code_unit_bounds_impl(
                        &self.module.struct_def_instantiations,
                        *idx,
                        bytecode_offset,
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
                        return Err(self.offset_out_of_bounds(
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                            IndexKind::CodeDefinition,
                            offset,
                            code_len,
                            bytecode_offset as CodeOffset,
                        ));
                    }
                }
                // Instructions that refer to the locals.
                CopyLoc(idx) | MoveLoc(idx) | StLoc(idx) | MutBorrowLoc(idx)
                | ImmBorrowLoc(idx) => {
                    let idx = *idx as usize;
                    if idx >= locals_count {
                        return Err(self.offset_out_of_bounds(
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                            IndexKind::LocalPool,
                            idx,
                            locals_count,
                            bytecode_offset as CodeOffset,
                        ));
                    }
                }

                // List out the other options explicitly so there's a compile error if a new
                // bytecode gets added.
                FreezeRef | Pop | Ret | LdU8(_) | LdU64(_) | LdU128(_) | CastU8 | CastU64
                | CastU128 | LdTrue | LdFalse | ReadRef | WriteRef | Add | Sub | Mul | Mod
                | Div | BitOr | BitAnd | Xor | Shl | Shr | Or | And | Not | Eq | Neq | Lt | Gt
                | Le | Ge | Abort | Nop => (),
            }
        }
        Ok(())
    }

    fn check_type(&self, ty: &SignatureToken) -> PartialVMResult<()> {
        use self::SignatureToken::*;

        for ty in ty.preorder_traversal() {
            match ty {
                Bool | U8 | U64 | U128 | Address | Signer | TypeParameter(_) | Reference(_)
                | MutableReference(_) | Vector(_) => (),
                Struct(idx) => {
                    check_bounds_impl(&self.module.struct_handles, *idx)?;
                    if let Some(sh) = self.module.struct_handles.get(idx.into_index()) {
                        if !sh.type_parameters.is_empty() {
                            return Err(PartialVMError::new(
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
                            return Err(PartialVMError::new(
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

    fn check_type_parameter(
        &self,
        ty: &SignatureToken,
        type_param_count: usize,
    ) -> PartialVMResult<()> {
        use self::SignatureToken::*;

        for ty in ty.preorder_traversal() {
            match ty {
                SignatureToken::TypeParameter(idx) => {
                    if *idx as usize >= type_param_count {
                        return Err(bounds_error(
                            StatusCode::INDEX_OUT_OF_BOUNDS,
                            IndexKind::TypeParameter,
                            *idx,
                            type_param_count,
                        ));
                    }
                }

                Bool
                | U8
                | U64
                | U128
                | Address
                | Signer
                | Struct(_)
                | Reference(_)
                | MutableReference(_)
                | Vector(_)
                | StructInstantiation(_, _) => (),
            }
        }
        Ok(())
    }

    fn check_code_unit_bounds_impl<T, I>(
        &self,
        pool: &[T],
        idx: I,
        bytecode_offset: usize,
    ) -> PartialVMResult<()>
    where
        I: ModuleIndex,
    {
        let idx = idx.into_index();
        let len = pool.len();
        if idx >= len {
            Err(self.offset_out_of_bounds(
                StatusCode::INDEX_OUT_OF_BOUNDS,
                I::KIND,
                idx,
                len,
                bytecode_offset as CodeOffset,
            ))
        } else {
            Ok(())
        }
    }

    fn offset_out_of_bounds(
        &self,
        status: StatusCode,
        kind: IndexKind,
        target_offset: usize,
        target_pool_len: usize,
        cur_bytecode_offset: CodeOffset,
    ) -> PartialVMError {
        match self.current_function {
            None => {
                let msg = format!("Indexing into bytecode {} during bounds checking but 'current_function' was not set", cur_bytecode_offset);
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(msg)
            }
            Some(current_function_index) => offset_out_of_bounds_error(
                status,
                kind,
                target_offset,
                target_pool_len,
                current_function_index,
                cur_bytecode_offset,
            ),
        }
    }
}

fn check_bounds_impl<T, I>(pool: &[T], idx: I) -> PartialVMResult<()>
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        Err(bounds_error(
            StatusCode::INDEX_OUT_OF_BOUNDS,
            I::KIND,
            idx as TableIndex,
            len,
        ))
    } else {
        Ok(())
    }
}
