// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::append_err_info,
    file_format::{Bytecode, CompiledModule, Kind, SignatureToken, StructFieldInformation},
    IndexKind,
};

pub struct SignatureChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> SignatureChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self { module }
    }

    // TODO: should probably verify the Signature pool in general
    pub fn verify(self) -> Vec<VMStatus> {
        self.verify_signatures()
            .chain(self.verify_function_signatures())
            .chain(self.verify_fields())
            .chain(self.verify_code_units())
            .collect()
    }

    fn verify_signatures(&self) -> impl Iterator<Item = VMStatus> + '_ {
        use SignatureToken::*;

        self.module.signatures().iter().filter_map(|sig| {
            for ty in &sig.0 {
                match ty {
                    Reference(inner) | MutableReference(inner) => match **inner {
                        Reference(_) | MutableReference(_) => {
                            return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN));
                        }
                        _ => (),
                    },
                    Vector(inner) => match **inner {
                        Reference(_) | MutableReference(_) => {
                            return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN));
                        }
                        _ => (), // TODO: vector of vectors
                    },
                    _ => (),
                }
            }
            None
        })
    }

    fn verify_function_signatures(&self) -> impl Iterator<Item = VMStatus> + '_ {
        self.module
            .function_handles()
            .iter()
            .enumerate()
            .flat_map(move |(idx, fh)| {
                let type_parameters = fh.type_parameters.as_slice();

                let return_ = self.module.signature_at(fh.return_);
                let errors_return = return_.0.iter().flat_map(move |ty| {
                    self.check_signature(ty, type_parameters)
                        .into_iter()
                        .map(move |err| append_err_info(err, IndexKind::Signature, idx))
                });
                let parameters = self.module.signature_at(fh.parameters);
                let errors_params = parameters.0.iter().flat_map(move |ty| {
                    self.check_signature(ty, type_parameters)
                        .into_iter()
                        .map(move |err| append_err_info(err, IndexKind::Signature, idx))
                });
                errors_return.chain(errors_params)
            })
    }

    fn verify_fields(&self) -> impl Iterator<Item = VMStatus> + '_ {
        self.module
            .struct_defs()
            .iter()
            .enumerate()
            .filter_map(
                move |(struct_def_idx, struct_def)| match &struct_def.field_information {
                    StructFieldInformation::Native => None,
                    StructFieldInformation::Declared(fields) => {
                        let struct_handle = self.module.struct_handle_at(struct_def.struct_handle);
                        let type_parameters = &struct_handle.type_parameters;

                        let errors =
                            fields
                                .iter()
                                .enumerate()
                                .flat_map(move |(field_offset, field_def)| {
                                    let ty = &field_def.signature;
                                    self.check_signature_no_refs(&ty.0, type_parameters)
                                        .into_iter()
                                        .map(move |err| {
                                            append_err_info(
                                                append_err_info(
                                                    append_err_info(
                                                        VMStatus::new(
                                                            StatusCode::INVALID_FIELD_DEF,
                                                        )
                                                        .append(err),
                                                        IndexKind::FieldDefinition,
                                                        field_offset,
                                                    ),
                                                    IndexKind::FieldDefinition,
                                                    field_offset,
                                                ),
                                                IndexKind::StructDefinition,
                                                struct_def_idx,
                                            )
                                        })
                                });
                        Some(errors)
                    }
                },
            )
            .flatten()
    }

    fn verify_code_units(&self) -> impl Iterator<Item = VMStatus> + '_ {
        use Bytecode::*;

        self.module
            .function_defs()
            .iter()
            .enumerate()
            .filter_map(move |(func_def_idx, func_def)| {
                // Nothing to check for native functions so skipping.
                if func_def.is_native() {
                    return None;
                }

                // Check if the types of the locals are well defined.
                let func_handle = self.module.function_handle_at(func_def.function);
                let type_parameters = func_handle.type_parameters.as_slice();

                let locals_idx = func_def.code.locals;
                let locals = &self.module.signature_at(locals_idx).0;
                let errors_locals = locals.iter().flat_map(move |ty| {
                    self.check_signature(ty, type_parameters)
                        .into_iter()
                        .map(move |err| {
                            append_err_info(
                                append_err_info(err, IndexKind::Signature, locals_idx.0 as usize),
                                IndexKind::FunctionDefinition,
                                func_def_idx,
                            )
                        })
                });

                // Check if the type actuals in certain bytecode instructions are well defined.
                let errors_bytecodes =
                    func_def
                        .code
                        .code
                        .iter()
                        .enumerate()
                        .flat_map(move |(offset, instr)| {
                            let errors = match instr {
                                CallGeneric(idx) => {
                                    let func_inst = self.module.function_instantiation_at(*idx);
                                    let func_handle =
                                        self.module.function_handle_at(func_inst.handle);
                                    let type_arguments = self
                                        .module
                                        .signature_at(func_inst.type_parameters)
                                        .0
                                        .as_slice();
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
                                    let struct_handle =
                                        self.module.struct_handle_at(struct_def.struct_handle);
                                    let type_arguments =
                                        &self.module.signature_at(struct_inst.type_parameters).0;
                                    self.check_generic_instance(
                                        type_arguments,
                                        struct_handle.type_parameters.as_slice(),
                                        type_parameters,
                                    )
                                }
                                _ => vec![],
                            };
                            errors.into_iter().map(move |err| {
                                append_err_info(
                                    err.append_message_with_separator(
                                        ' ',
                                        format!("at offset {} ", offset),
                                    ),
                                    IndexKind::FunctionDefinition,
                                    func_def_idx,
                                )
                            })
                        });

                Some(errors_locals.chain(errors_bytecodes))
            })
            .flatten()
    }

    // Checks if the given types are well defined and satisfy the given kind constraints in
    // the given context.
    fn check_generic_instance(
        &self,
        type_arguments: &[SignatureToken],
        scope_kinds: &[Kind],
        global_kinds: &[Kind],
    ) -> Vec<VMStatus> {
        let mut errors: Vec<_> = type_arguments
            .iter()
            .flat_map(|ty| self.check_signature_no_refs(ty, global_kinds))
            .collect();

        let kinds = type_arguments
            .iter()
            .map(|ty| kind(self.module, ty, global_kinds));
        errors.extend(
            scope_kinds
                .iter()
                .zip(kinds)
                .zip(type_arguments.iter())
                .filter_map(|((c, k), ty)| {
                    if k.is_sub_kind_of(*c) {
                        return None;
                    }
                    Some(
                        VMStatus::new(StatusCode::CONTRAINT_KIND_MISMATCH).with_message(format!(
                            "expected kind {:?} got type actual {:?} with incompatible kind {:?}",
                            c, ty, k
                        )),
                    )
                }),
        );
        errors
    }

    /// Checks if the given type is well defined in the given context. No references are permitted.
    fn check_signature_no_refs(&self, ty: &SignatureToken, kinds: &[Kind]) -> Vec<VMStatus> {
        use SignatureToken::*;

        match ty {
            U8 | U64 | U128 | Bool | Address | Struct(_) | TypeParameter(_) => vec![],
            Reference(_) | MutableReference(_) => {
                // TODO: Prop tests expect us to NOT check the inner types.
                // Revisit this once we rework prop tests.
                vec![VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN)
                    .with_message("reference not allowed".to_string())]
            }
            Vector(ty) => self.check_signature_no_refs(ty, kinds),
            StructInstantiation(idx, type_arguments) => {
                let sh = self.module.struct_handle_at(*idx);
                self.check_generic_instance(type_arguments, &sh.type_parameters, kinds)
            }
        }
    }

    /// Checks if the given type is well defined in the given context. References are only permitted
    /// at the top level.
    fn check_signature(&self, ty: &SignatureToken, kinds: &[Kind]) -> Vec<VMStatus> {
        use SignatureToken::*;

        match ty {
            Reference(inner) | MutableReference(inner) => {
                self.check_signature_no_refs(inner, kinds)
            }
            _ => self.check_signature_no_refs(ty, kinds),
        }
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
