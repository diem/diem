// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{bounds_error, bytecode_offset_err, verification_error},
    file_format::{
        Bytecode, CompiledModuleMut, Constant, FieldHandle, FieldInstantiation, FunctionDefinition,
        FunctionHandle, FunctionInstantiation, ModuleHandle, Signature, SignatureToken,
        StructDefInstantiation, StructDefinition, StructFieldInformation, StructHandle,
    },
    internals::ModuleIndex,
    IndexKind,
};
use libra_types::vm_error::{StatusCode, VMStatus};

pub struct BoundsChecker<'a> {
    module: &'a CompiledModuleMut,
}

impl<'a> BoundsChecker<'a> {
    pub fn new(module: &'a CompiledModuleMut) -> Self {
        Self { module }
    }

    pub fn verify(self) -> Vec<VMStatus> {
        let mut errors: Vec<VMStatus> = vec![];

        // TODO: this will not be true once we change CompiledScript and remove the
        // FunctionDefinition for `main`
        if self.module.module_handles.is_empty() {
            let status =
                verification_error(IndexKind::ModuleHandle, 0, StatusCode::NO_MODULE_HANDLES);
            errors.push(status);
        }

        for signature in &self.module.signatures {
            self.check_signature(signature, &mut errors);
        }
        for constant in &self.module.constant_pool {
            self.check_constant(constant, &mut errors);
        }
        for module_handle in &self.module.module_handles {
            self.check_module_handle(module_handle, &mut errors);
        }
        for struct_handle in &self.module.struct_handles {
            self.check_struct_handle(struct_handle, &mut errors);
        }
        for function_handle in &self.module.function_handles {
            self.check_function_handle(function_handle, &mut errors);
        }
        for field_handle in &self.module.field_handles {
            self.check_field_handle(field_handle, &mut errors);
        }
        for struct_instantiation in &self.module.struct_def_instantiations {
            self.check_struct_instantiation(struct_instantiation, &mut errors);
        }
        for function_instantiation in &self.module.function_instantiations {
            self.check_function_instantiation(function_instantiation, &mut errors);
        }
        for field_instantiation in &self.module.field_instantiations {
            self.check_field_instantiation(field_instantiation, &mut errors);
        }
        for struct_def in &self.module.struct_defs {
            self.check_struct_def(struct_def, &mut errors);
        }
        for function_def in &self.module.function_defs {
            self.check_function_def(function_def, &mut errors);
        }
        errors
    }

    fn check_module_handle(&self, module_handle: &ModuleHandle, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(
            &self.module.address_identifiers,
            module_handle.address,
            errors,
        );
        check_bounds_impl(&self.module.identifiers, module_handle.name, errors);
    }

    fn check_struct_handle(&self, struct_handle: &StructHandle, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(&self.module.module_handles, struct_handle.module, errors);
        check_bounds_impl(&self.module.identifiers, struct_handle.name, errors);
    }

    fn check_function_handle(&self, function_handle: &FunctionHandle, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(&self.module.module_handles, function_handle.module, errors);
        check_bounds_impl(&self.module.identifiers, function_handle.name, errors);
        check_bounds_impl(&self.module.signatures, function_handle.parameters, errors);
        check_bounds_impl(&self.module.signatures, function_handle.return_, errors);
        // function signature type paramters must be in bounds to the function type parameters
        let type_param_count = function_handle.type_parameters.len();
        if let Some(sig) = self
            .module
            .signatures
            .get(function_handle.parameters.into_index())
        {
            for ty in &sig.0 {
                self.check_type_parameter(ty, type_param_count, errors);
            }
        }
        if let Some(sig) = self
            .module
            .signatures
            .get(function_handle.return_.into_index())
        {
            for ty in &sig.0 {
                self.check_type_parameter(ty, type_param_count, errors);
            }
        }
    }

    fn check_field_handle(&self, field_handle: &FieldHandle, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(&self.module.struct_defs, field_handle.owner, errors);
        // field offset must be in bounds, struct def just checked above must exist
        if let Some(struct_def) = &self.module.struct_defs.get(field_handle.owner.into_index()) {
            let fields_count = match &struct_def.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            if field_handle.field as usize >= fields_count {
                errors.push(bounds_error(
                    IndexKind::MemberCount,
                    field_handle.field as usize,
                    fields_count,
                    StatusCode::INDEX_OUT_OF_BOUNDS,
                ));
            }
        }
    }

    fn check_struct_instantiation(
        &self,
        struct_instantiation: &StructDefInstantiation,
        errors: &mut Vec<VMStatus>,
    ) {
        check_bounds_impl(&self.module.struct_defs, struct_instantiation.def, errors);
        check_bounds_impl(
            &self.module.signatures,
            struct_instantiation.type_parameters,
            errors,
        );
    }

    fn check_function_instantiation(
        &self,
        function_instantiation: &FunctionInstantiation,
        errors: &mut Vec<VMStatus>,
    ) {
        check_bounds_impl(
            &self.module.function_handles,
            function_instantiation.handle,
            errors,
        );
        check_bounds_impl(
            &self.module.signatures,
            function_instantiation.type_parameters,
            errors,
        );
    }

    fn check_field_instantiation(
        &self,
        field_instantiation: &FieldInstantiation,
        errors: &mut Vec<VMStatus>,
    ) {
        check_bounds_impl(
            &self.module.field_handles,
            field_instantiation.handle,
            errors,
        );
        check_bounds_impl(
            &self.module.signatures,
            field_instantiation.type_parameters,
            errors,
        );
    }

    fn check_signature(&self, signature: &Signature, errors: &mut Vec<VMStatus>) {
        for ty in &signature.0 {
            self.check_type(ty, errors);
        }
    }

    fn check_constant(&self, constant: &Constant, errors: &mut Vec<VMStatus>) {
        self.check_type(&constant.type_, errors);
    }

    fn check_struct_def(&self, struct_def: &StructDefinition, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(
            &self.module.struct_handles,
            struct_def.struct_handle,
            errors,
        );
        // check signature (type) and type parameter for the field type
        if let StructFieldInformation::Declared(fields) = &struct_def.field_information {
            let type_param_count = self
                .module
                .struct_handles
                .get(struct_def.struct_handle.into_index())
                .map_or(0, |sh| sh.type_parameters.len());
            // field signatures are inlined
            for field in fields {
                check_bounds_impl(&self.module.identifiers, field.name, errors);
                self.check_type(&field.signature.0, errors);
                self.check_type_parameter(&field.signature.0, type_param_count, errors);
            }
        }
    }

    fn check_function_def(&self, function_def: &FunctionDefinition, errors: &mut Vec<VMStatus>) {
        check_bounds_impl(&self.module.function_handles, function_def.function, errors);
        for ty in &function_def.acquires_global_resources {
            check_bounds_impl(&self.module.struct_defs, *ty, errors);
        }
        self.check_code(function_def, errors);
    }

    fn check_code(&self, function_def: &FunctionDefinition, errors: &mut Vec<VMStatus>) {
        if function_def.is_native() {
            return;
        }
        let code_unit = &function_def.code;
        check_bounds_impl(&self.module.signatures, code_unit.locals, errors);

        let type_param_count = match self
            .module
            .function_handles
            .get(function_def.function.into_index())
        {
            Some(fh) => fh.type_parameters.len(),
            None => 0,
        };
        // if there are locals check that the type parameters in local signature are in bounds.
        let locals = match &self.module.signatures.get(code_unit.locals.into_index()) {
            Some(locals) => &locals.0,
            None => return, // stop now
        };
        let locals_count = locals.len();
        for local in locals {
            self.check_type_parameter(local, type_param_count, errors);
        }

        // check bytecodes
        let code_len = code_unit.code.len();
        for (bytecode_offset, bytecode) in code_unit.code.iter().enumerate() {
            use self::Bytecode::*;

            match bytecode {
                LdConst(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.constant_pool,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
                }
                MutBorrowField(idx) | ImmBorrowField(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.field_handles,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
                }
                MutBorrowFieldGeneric(idx) | ImmBorrowFieldGeneric(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.field_instantiations,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
                    // check type parameters in borrow are bound to the function type parameters
                    if let Some(field_inst) = self.module.field_instantiations.get(idx.into_index())
                    {
                        if let Some(sig) = self
                            .module
                            .signatures
                            .get(field_inst.type_parameters.into_index())
                        {
                            for ty in &sig.0 {
                                self.check_type_parameter(ty, type_param_count, errors);
                            }
                        }
                    }
                }
                Call(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.function_handles,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
                }
                CallGeneric(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.function_instantiations,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
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
                                self.check_type_parameter(ty, type_param_count, errors);
                            }
                        }
                    }
                }
                Pack(idx) | Unpack(idx) | Exists(idx) | ImmBorrowGlobal(idx)
                | MutBorrowGlobal(idx) | MoveFrom(idx) | MoveToSender(idx) => {
                    check_code_unit_bounds_impl(
                        &self.module.struct_defs,
                        bytecode_offset,
                        *idx,
                        errors,
                    );
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
                        errors,
                    );
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
                                self.check_type_parameter(ty, type_param_count, errors);
                            }
                        }
                    }
                }
                // Instructions that refer to this code block.
                BrTrue(offset) | BrFalse(offset) | Branch(offset) => {
                    let offset = *offset as usize;
                    if offset >= code_len {
                        errors.push(bytecode_offset_err(
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
                        errors.push(bytecode_offset_err(
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
                | Le | Ge | Abort | GetTxnGasUnitPrice | GetTxnMaxGasUnits | GetGasRemaining
                | GetTxnSenderAddress | GetTxnSequenceNumber | GetTxnPublicKey | Nop => (),
            }
        }
    }

    fn check_type(&self, type_: &SignatureToken, errors: &mut Vec<VMStatus>) {
        use self::SignatureToken::*;

        match type_ {
            Bool | U8 | U64 | U128 | Address | TypeParameter(_) => (),
            Reference(ty) | MutableReference(ty) | Vector(ty) => self.check_type(ty, errors),
            Struct(idx) => {
                check_bounds_impl(&self.module.struct_handles, *idx, errors);
                if let Some(sh) = self.module.struct_handles.get(idx.into_index()) {
                    if !sh.type_parameters.is_empty() {
                        errors.push(
                            VMStatus::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH)
                                .with_message(format!(
                                    "expected {} type parameters got 0 (Struct)",
                                    sh.type_parameters.len(),
                                )),
                        );
                    }
                }
            }
            StructInstantiation(idx, type_params) => {
                check_bounds_impl(&self.module.struct_handles, *idx, errors);
                if let Some(sh) = self.module.struct_handles.get(idx.into_index()) {
                    if sh.type_parameters.len() != type_params.len() {
                        errors.push(
                            VMStatus::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH)
                                .with_message(format!(
                                    "expected {} type parameters got {}",
                                    sh.type_parameters.len(),
                                    type_params.len(),
                                )),
                        );
                    }
                }
            }
        }
    }

    fn check_type_parameter(
        &self,
        type_: &SignatureToken,
        type_param_count: usize,
        errors: &mut Vec<VMStatus>,
    ) {
        use self::SignatureToken::*;

        match type_ {
            Bool | U8 | U64 | U128 | Address | Struct(_) => (),
            Reference(ty) | MutableReference(ty) | Vector(ty) => {
                self.check_type_parameter(ty, type_param_count, errors)
            }
            // TODO: is this correct?
            SignatureToken::StructInstantiation(_, type_params) => {
                for type_param in type_params {
                    self.check_type_parameter(type_param, type_param_count, errors);
                }
            }
            SignatureToken::TypeParameter(idx) => {
                if *idx as usize >= type_param_count {
                    errors.push(bounds_error(
                        IndexKind::TypeParameter,
                        *idx as usize,
                        type_param_count,
                        StatusCode::INDEX_OUT_OF_BOUNDS,
                    ));
                }
            }
        }
    }
}

fn check_bounds_impl<T, I>(pool: &[T], idx: I, errors: &mut Vec<VMStatus>)
where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        errors.push(bounds_error(
            I::KIND,
            idx,
            len,
            StatusCode::INDEX_OUT_OF_BOUNDS,
        ));
    }
}

fn check_code_unit_bounds_impl<T, I>(
    pool: &[T],
    bytecode_offset: usize,
    idx: I,
    errors: &mut Vec<VMStatus>,
) where
    I: ModuleIndex,
{
    let idx = idx.into_index();
    let len = pool.len();
    if idx >= len {
        errors.push(bytecode_offset_err(
            I::KIND,
            idx,
            len,
            bytecode_offset,
            StatusCode::INDEX_OUT_OF_BOUNDS,
        ));
    }
}
