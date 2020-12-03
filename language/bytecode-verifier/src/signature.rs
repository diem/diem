// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use crate::binary_views::BinaryIndexedView;
use diem_types::vm_status::StatusCode;
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        Bytecode, CodeUnit, CompiledModule, CompiledScript, FunctionDefinition, FunctionHandle,
        Kind, Signature, SignatureIndex, SignatureToken, StructDefinition, StructFieldInformation,
        TableIndex,
    },
    IndexKind,
};

pub struct SignatureChecker<'a> {
    resolver: BinaryIndexedView<'a>,
}

impl<'a> SignatureChecker<'a> {
    pub fn verify_module(module: &'a CompiledModule) -> VMResult<()> {
        Self::verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    fn verify_module_impl(module: &'a CompiledModule) -> PartialVMResult<()> {
        let sig_check = Self {
            resolver: BinaryIndexedView::Module(module),
        };
        sig_check.verify_signature_pool(module.signatures())?;
        sig_check.verify_function_signatures(module.function_handles())?;
        sig_check.verify_fields(module.struct_defs())?;
        sig_check.verify_code_units(module.function_defs())
    }

    pub fn verify_script(module: &'a CompiledScript) -> VMResult<()> {
        Self::verify_script_impl(module).map_err(|e| e.finish(Location::Script))
    }

    fn verify_script_impl(script: &'a CompiledScript) -> PartialVMResult<()> {
        let sig_check = Self {
            resolver: BinaryIndexedView::Script(script),
        };
        sig_check.verify_signature_pool(script.signatures())?;
        sig_check.verify_function_signatures(script.function_handles())?;
        sig_check.verify_code(script.code(), &script.as_inner().type_parameters)
    }

    fn verify_signature_pool(&self, signatures: &[Signature]) -> PartialVMResult<()> {
        for i in 0..signatures.len() {
            self.check_signature(SignatureIndex::new(i as TableIndex))?
        }
        Ok(())
    }

    fn verify_function_signatures(
        &self,
        function_handles: &[FunctionHandle],
    ) -> PartialVMResult<()> {
        let err_handler = |err: PartialVMError, idx| {
            err.at_index(IndexKind::Signature, idx as TableIndex)
                .at_index(IndexKind::FunctionHandle, idx as TableIndex)
        };

        for (idx, fh) in function_handles.iter().enumerate() {
            // Update the type parameter kinds to the function being checked
            self.check_signature(fh.return_)
                .map_err(|err| err_handler(err, idx))?;
            self.check_instantiation(fh.return_, &fh.type_parameters)
                .map_err(|err| err_handler(err, idx))?;
            self.check_signature(fh.parameters)
                .map_err(|err| err_handler(err, idx))?;
            self.check_instantiation(fh.parameters, &fh.type_parameters)
                .map_err(|err| err_handler(err, idx))?;
            if !fh.type_parameters.is_empty() {}
        }
        Ok(())
    }

    fn verify_fields(&self, struct_defs: &[StructDefinition]) -> PartialVMResult<()> {
        for (struct_def_idx, struct_def) in struct_defs.iter().enumerate() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => continue,
                StructFieldInformation::Declared(fields) => fields,
            };
            let struct_handle = self.resolver.struct_handle_at(struct_def.struct_handle);
            // Update the type parameter kinds to the struct being checked
            let err_handler = |err: PartialVMError, idx| {
                err.at_index(IndexKind::FieldDefinition, idx as TableIndex)
                    .at_index(IndexKind::StructDefinition, struct_def_idx as TableIndex)
            };
            for (field_offset, field_def) in fields.iter().enumerate() {
                self.check_signature_token(&field_def.signature.0)
                    .map_err(|err| err_handler(err, field_offset))?;
                self.check_type_instantiation(
                    &field_def.signature.0,
                    &struct_handle.type_parameters,
                )
                .map_err(|err| err_handler(err, field_offset))?;
            }
        }
        Ok(())
    }

    fn verify_code_units(&self, function_defs: &[FunctionDefinition]) -> PartialVMResult<()> {
        for (func_def_idx, func_def) in function_defs.iter().enumerate() {
            // skip native functions
            let code = match &func_def.code {
                Some(code) => code,
                None => continue,
            };
            let func_handle = self.resolver.function_handle_at(func_def.function);
            self.verify_code(code, &func_handle.type_parameters)
                .map_err(|err| {
                    err.at_index(IndexKind::Signature, code.locals.0)
                        .at_index(IndexKind::FunctionDefinition, func_def_idx as TableIndex)
                })?
        }
        Ok(())
    }

    fn verify_code(&self, code: &CodeUnit, type_parameters: &[Kind]) -> PartialVMResult<()> {
        self.check_signature(code.locals)?;
        self.check_instantiation(code.locals, type_parameters)?;

        // Check if the type actuals in certain bytecode instructions are well defined.
        use Bytecode::*;
        for (offset, instr) in code.code.iter().enumerate() {
            let result = match instr {
                CallGeneric(idx) => {
                    let func_inst = self.resolver.function_instantiation_at(*idx);
                    let func_handle = self.resolver.function_handle_at(func_inst.handle);
                    let type_arguments = &self.resolver.signature_at(func_inst.type_parameters).0;
                    self.check_signature_tokens(type_arguments)?;
                    self.check_generic_instance(
                        type_arguments,
                        &func_handle.type_parameters,
                        type_parameters,
                    )
                }
                PackGeneric(idx)
                | UnpackGeneric(idx)
                | ExistsGeneric(idx)
                | MoveFromGeneric(idx)
                | ImmBorrowGlobalGeneric(idx)
                | MutBorrowGlobalGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    let struct_def = self.resolver.struct_def_at(struct_inst.def)?;
                    let struct_handle = self.resolver.struct_handle_at(struct_def.struct_handle);
                    let type_arguments = &self.resolver.signature_at(struct_inst.type_parameters).0;
                    self.check_signature_tokens(type_arguments)?;
                    self.check_generic_instance(
                        type_arguments,
                        &struct_handle.type_parameters,
                        type_parameters,
                    )
                }
                _ => Ok(()),
            };
            result.map_err(|err| {
                err.append_message_with_separator(' ', format!("at offset {} ", offset))
            })?
        }
        Ok(())
    }

    /// Checks if the given type is well defined in the given context.
    /// References are only permitted at the top level.
    fn check_signature(&self, idx: SignatureIndex) -> PartialVMResult<()> {
        for token in &self.resolver.signature_at(idx).0 {
            match token {
                SignatureToken::Reference(inner) | SignatureToken::MutableReference(inner) => {
                    self.check_signature_token(inner)?
                }
                _ => self.check_signature_token(token)?,
            }
        }
        Ok(())
    }

    /// Checks if the given types are well defined in the given context.
    /// No references are permitted.
    fn check_signature_tokens(&self, tys: &[SignatureToken]) -> PartialVMResult<()> {
        for ty in tys {
            self.check_signature_token(ty)?
        }
        Ok(())
    }

    /// Checks if the given type is well defined in the given context.
    /// No references are permitted.
    fn check_signature_token(&self, ty: &SignatureToken) -> PartialVMResult<()> {
        use SignatureToken::*;
        match ty {
            U8 | U64 | U128 | Bool | Address | Signer | Struct(_) | TypeParameter(_) => Ok(()),
            Reference(_) | MutableReference(_) => {
                // TODO: Prop tests expect us to NOT check the inner types.
                // Revisit this once we rework prop tests.
                Err(PartialVMError::new(StatusCode::INVALID_SIGNATURE_TOKEN)
                    .with_message("reference not allowed".to_string()))
            }
            Vector(ty) => self.check_signature_token(ty),
            StructInstantiation(_, type_arguments) => self.check_signature_tokens(type_arguments),
        }
    }

    fn check_instantiation(&self, idx: SignatureIndex, kinds: &[Kind]) -> PartialVMResult<()> {
        for ty in &self.resolver.signature_at(idx).0 {
            self.check_type_instantiation(ty, kinds)?
        }
        Ok(())
    }

    fn check_type_instantiation(&self, ty: &SignatureToken, kinds: &[Kind]) -> PartialVMResult<()> {
        match ty {
            SignatureToken::StructInstantiation(idx, type_arguments) => {
                // Check that the instantiation satisfies the `idx` struct's constraints
                // Cannot be checked completely if we do not know the constraints of type parameters
                // i.e. it cannot be checked unless we are inside some module member. The only case
                // where that happens is when checking the signature pool itself
                let sh = self.resolver.struct_handle_at(*idx);
                self.check_generic_instance(type_arguments, &sh.type_parameters, kinds)
            }
            _ => Ok(()),
        }
    }

    // Checks if the given types are well defined and satisfy the given kind constraints in
    // the given context.
    fn check_generic_instance(
        &self,
        type_arguments: &[SignatureToken],
        scope_kinds: &[Kind],
        global_kinds: &[Kind],
    ) -> PartialVMResult<()> {
        let kinds = type_arguments
            .iter()
            .map(|ty| self.resolver.kind(ty, global_kinds));
        for ((c, k), ty) in scope_kinds.iter().zip(kinds).zip(type_arguments) {
            if !k.is_sub_kind_of(*c) {
                return Err(PartialVMError::new(StatusCode::CONSTRAINT_KIND_MISMATCH)
                    .with_message(format!(
                        "expected kind {:?} got type actual {:?} with incompatible kind {:?}",
                        c, ty, k
                    )));
            }
        }
        Ok(())
    }
}
