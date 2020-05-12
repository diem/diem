// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::{append_err_info, VMResult},
    file_format::{
        Bytecode, CompiledModule, Kind, SignatureIndex, SignatureToken, StructFieldInformation,
        TableIndex,
    },
    IndexKind,
};

pub struct SignatureChecker<'a> {
    module: &'a CompiledModule,
    // The kinds of the type paramters of the current module member being checked.
    // The kinds are then used to validate constraints of instantiated types.
    // See`check_generic_instance`
    // For the specific use cases:
    // - When checking the signature pools directly, there are no type paramters.
    // - When checking function signatures, the type paramters belong to that function
    // - When checking fields, the type parameters belong to the containing struct
    // - When checking code bodies, the type parameters belong to the containing function definition
    signature_context: Option<Vec<Kind>>,
    // TODO can cache results of signatures checked
}

impl<'a> SignatureChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module,
            signature_context: None,
        }
    }

    pub fn verify(mut self) -> VMResult<()> {
        self.verify_signature_pool()?;
        self.verify_function_signatures()?;
        self.verify_fields()?;
        self.verify_code_units()
    }

    fn verify_signature_pool(&mut self) -> VMResult<()> {
        // The signature pool does not have type parameters
        self.signature_context = None;
        for i in 0..self.module.signatures().len() {
            self.check_signature(SignatureIndex::new(i as TableIndex))?
        }
        Ok(())
    }

    fn verify_function_signatures(&mut self) -> VMResult<()> {
        for (idx, fh) in self.module.function_handles().iter().enumerate() {
            // Update the type parameter kinds to the function being checked
            self.signature_context = Some(fh.type_parameters.clone());
            self.check_signature(fh.return_).map_err(|err| {
                let err = append_err_info(err, IndexKind::Signature, fh.return_.0 as usize);
                append_err_info(err, IndexKind::FunctionDefinition, idx)
            })?;
            self.check_signature(fh.parameters).map_err(|err| {
                let err = append_err_info(err, IndexKind::Signature, fh.parameters.0 as usize);
                append_err_info(err, IndexKind::FunctionDefinition, idx)
            })?;
        }
        Ok(())
    }

    fn verify_fields(&mut self) -> VMResult<()> {
        for (struct_def_idx, struct_def) in self.module.struct_defs().iter().enumerate() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => continue,
                StructFieldInformation::Declared(fields) => fields,
            };
            let struct_handle = self.module.struct_handle_at(struct_def.struct_handle);
            // Update the type parameter kinds to the struct being checked
            self.signature_context = Some(struct_handle.type_parameters.clone());
            for (field_offset, field_def) in fields.iter().enumerate() {
                self.check_signature_token(&field_def.signature.0)
                    .map_err(|err| {
                        let err = VMStatus::new(StatusCode::INVALID_FIELD_DEF).append(err);
                        let err = append_err_info(err, IndexKind::FieldDefinition, field_offset);
                        append_err_info(err, IndexKind::StructDefinition, struct_def_idx)
                    })?
            }
        }
        Ok(())
    }

    fn verify_code_units(&mut self) -> VMResult<()> {
        use Bytecode::*;
        for (func_def_idx, func_def) in self.module.function_defs().iter().enumerate() {
            let code = match &func_def.code {
                Some(code) => code,
                None => continue,
            };

            // Check if the types of the locals are well defined.
            let func_handle = self.module.function_handle_at(func_def.function);
            let type_parameters = &func_handle.type_parameters;
            // Update the type parameter kinds to the function being checked
            self.signature_context = Some(type_parameters.clone());

            self.check_signature(code.locals).map_err(|err| {
                let err = append_err_info(err, IndexKind::Signature, code.locals.0 as usize);
                append_err_info(err, IndexKind::FunctionDefinition, func_def_idx)
            })?;

            // Check if the type actuals in certain bytecode instructions are well defined.
            for (offset, instr) in code.code.iter().enumerate() {
                let result = match instr {
                    CallGeneric(idx) => {
                        let func_inst = self.module.function_instantiation_at(*idx);
                        let func_handle = self.module.function_handle_at(func_inst.handle);
                        let type_arguments = &self.module.signature_at(func_inst.type_parameters).0;
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
                    | MoveToSenderGeneric(idx)
                    | ImmBorrowGlobalGeneric(idx)
                    | MutBorrowGlobalGeneric(idx) => {
                        let struct_inst = self.module.struct_instantiation_at(*idx);
                        let struct_def = self.module.struct_def_at(struct_inst.def);
                        let struct_handle = self.module.struct_handle_at(struct_def.struct_handle);
                        let type_arguments =
                            &self.module.signature_at(struct_inst.type_parameters).0;
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
                    let err =
                        err.append_message_with_separator(' ', format!("at offset {} ", offset));
                    append_err_info(err, IndexKind::FunctionDefinition, func_def_idx)
                })?
            }
        }
        Ok(())
    }

    /// Checks if the given type is well defined in the given context.
    /// References are only permitted at the top level.
    fn check_signature(&mut self, idx: SignatureIndex) -> VMResult<()> {
        for token in &self.module.signature_at(idx).0 {
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
    fn check_signature_tokens(&self, tys: &[SignatureToken]) -> VMResult<()> {
        for ty in tys {
            self.check_signature_token(ty)?
        }
        Ok(())
    }

    /// Checks if the given type is well defined in the given context.
    /// No references are permitted.
    fn check_signature_token(&self, ty: &SignatureToken) -> VMResult<()> {
        use SignatureToken::*;
        match ty {
            U8 | U64 | U128 | Bool | Address | Signer | Struct(_) | TypeParameter(_) => Ok(()),
            Reference(_) | MutableReference(_) => {
                // TODO: Prop tests expect us to NOT check the inner types.
                // Revisit this once we rework prop tests.
                Err(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN)
                    .with_message("reference not allowed".to_string()))
            }
            Vector(ty) => self.check_signature_token(ty),
            StructInstantiation(idx, type_arguments) => {
                self.check_signature_tokens(type_arguments)?;
                let sh = self.module.struct_handle_at(*idx);
                // Check that the instantiation satifies the `idx` struct's constraints
                // Cannot be checked completely if we do not know the constraints of type parameters
                // i.e. it cannot be checked unless we are inside some module member. The only case
                // where that happens is when checking the signature pool itself
                if let Some(kinds) = &self.signature_context {
                    self.check_generic_instance(type_arguments, &sh.type_parameters, kinds)?
                }
                Ok(())
            }
        }
    }

    // Checks if the given types are well defined and satisfy the given kind constraints in
    // the given context.
    fn check_generic_instance(
        &self,
        type_arguments: &[SignatureToken],
        scope_kinds: &[Kind],
        global_kinds: &[Kind],
    ) -> VMResult<()> {
        let kinds = type_arguments
            .iter()
            .map(|ty| kind(self.module, ty, global_kinds));
        for ((c, k), ty) in scope_kinds.iter().zip(kinds).zip(type_arguments) {
            if !k.is_sub_kind_of(*c) {
                return Err(
                    VMStatus::new(StatusCode::CONTRAINT_KIND_MISMATCH).with_message(format!(
                        "expected kind {:?} got type actual {:?} with incompatible kind {:?}",
                        c, ty, k
                    )),
                );
            }
        }
        Ok(())
    }
}

//
// Helpers functions for signatures
//

pub(crate) fn kind(module: &CompiledModule, ty: &SignatureToken, constraints: &[Kind]) -> Kind {
    use SignatureToken::*;

    match ty {
        // The primitive types & references have kind unrestricted.
        Bool | U8 | U64 | U128 | Address | Reference(_) | MutableReference(_) => Kind::Copyable,
        Signer => Kind::Resource,
        TypeParameter(idx) => constraints[*idx as usize],
        Vector(ty) => kind(module, ty, constraints),
        Struct(idx) => {
            let sh = module.struct_handle_at(*idx);
            if sh.is_nominal_resource {
                Kind::Resource
            } else {
                Kind::Copyable
            }
        }
        StructInstantiation(idx, type_args) => {
            let sh = module.struct_handle_at(*idx);
            if sh.is_nominal_resource {
                return Kind::Resource;
            }
            // Gather the kinds of the type actuals.
            let kinds = type_args
                .iter()
                .map(|ty| kind(module, ty, constraints))
                .collect::<Vec<_>>();
            // Derive the kind of the struct.
            //   - If any of the type actuals is `all`, then the struct is `all`.
            //     - `all` means some part of the type can be either `resource` or
            //       `unrestricted`.
            //     - Therefore it is also impossible to determine the kind of the type as a
            //       whole, and thus `all`.
            //   - If none of the type actuals is `all`, then the struct is a resource if
            //     and only if one of the type actuals is `resource`.
            kinds.iter().cloned().fold(Kind::Copyable, Kind::join)
        }
    }
}
