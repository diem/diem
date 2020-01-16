// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::append_err_info,
    file_format::{
        Bytecode, CompiledModule, Kind, SignatureToken, StructFieldInformation, StructHandle,
        TypeSignature,
    },
    IndexKind,
};

pub struct SignatureChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> SignatureChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self { module }
    }

    pub fn verify(self) -> Vec<VMStatus> {
        self.verify_function_signatures()
            .chain(self.verify_fields())
            .chain(self.verify_code_units())
            .chain(self.legacy_verify_type_signatures())
            .collect()
    }

    /// This is a hack to satisfy the existing prop tests.
    /// TODO: Remove it once we rework prop tests.
    fn legacy_verify_type_signatures(&self) -> impl Iterator<Item = VMStatus> + '_ {
        use SignatureToken::*;

        self.module
            .type_signatures()
            .iter()
            .filter_map(|TypeSignature(ty)| match ty {
                Reference(inner) | MutableReference(inner) => match **inner {
                    Reference(_) | MutableReference(_) => {
                        Some(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN))
                    }
                    _ => None,
                },
                _ => None,
            })
    }

    fn verify_function_signatures(&self) -> impl Iterator<Item = VMStatus> + '_ {
        self.module
            .function_signatures()
            .iter()
            .enumerate()
            .flat_map(move |(idx, sig)| {
                let context = (self.module.struct_handles(), sig.type_formals.as_slice());
                let errors_return_types = sig.return_types.iter().flat_map(move |ty| {
                    check_signature(context, ty)
                        .into_iter()
                        .map(move |err| append_err_info(err, IndexKind::FunctionSignature, idx))
                });
                let errors_arg_types = sig.arg_types.iter().flat_map(move |ty| {
                    check_signature(context, ty)
                        .into_iter()
                        .map(move |err| append_err_info(err, IndexKind::FunctionSignature, idx))
                });
                errors_return_types.chain(errors_arg_types)
            })
    }

    fn verify_fields(&self) -> impl Iterator<Item = VMStatus> + '_ {
        self.module
            .struct_defs()
            .iter()
            .enumerate()
            .filter_map(
                move |(struct_def_idx, struct_def)| match struct_def.field_information {
                    StructFieldInformation::Native => None,
                    StructFieldInformation::Declared {
                        field_count,
                        fields,
                    } => {
                        let struct_handle = self.module.struct_handle_at(struct_def.struct_handle);
                        let start = fields.0 as usize;
                        let end = start + (field_count as usize);
                        let context = (
                            self.module.struct_handles(),
                            struct_handle.type_formals.as_slice(),
                        );

                        let errors = self.module.field_defs()[start..end]
                            .iter()
                            .enumerate()
                            .flat_map(move |(field_def_idx, field_def)| {
                                let ty = self.module.type_signature_at(field_def.signature);

                                check_signature_no_refs(context, &ty.0).into_iter().map(
                                    move |err| {
                                        append_err_info(
                                            append_err_info(
                                                append_err_info(
                                                    VMStatus::new(StatusCode::INVALID_FIELD_DEF)
                                                        .append(err),
                                                    IndexKind::TypeSignature,
                                                    field_def.signature.0 as usize,
                                                ),
                                                IndexKind::FieldDefinition,
                                                field_def_idx,
                                            ),
                                            IndexKind::StructDefinition,
                                            struct_def_idx,
                                        )
                                    },
                                )
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
                let func_sig = self.module.function_signature_at(func_handle.signature);
                let context = (
                    self.module.struct_handles(),
                    func_sig.type_formals.as_slice(),
                );
                let locals_idx = func_def.code.locals;
                let locals = &self.module.locals_signature_at(locals_idx).0;
                let errors_locals = locals.iter().flat_map(move |ty| {
                    check_signature(context, ty).into_iter().map(move |err| {
                        append_err_info(
                            append_err_info(err, IndexKind::LocalsSignature, locals_idx.0 as usize),
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
                                Call(idx, type_actuals_idx) => {
                                    let func_handle = self.module.function_handle_at(*idx);
                                    let func_sig =
                                        self.module.function_signature_at(func_handle.signature);
                                    let type_actuals =
                                        &self.module.locals_signature_at(*type_actuals_idx).0;
                                    check_generic_instance(
                                        context,
                                        &func_sig.type_formals,
                                        type_actuals,
                                    )
                                }
                                Pack(idx, type_actuals_idx) | Unpack(idx, type_actuals_idx) => {
                                    let struct_def = self.module.struct_def_at(*idx);
                                    let struct_handle =
                                        self.module.struct_handle_at(struct_def.struct_handle);
                                    let type_actuals =
                                        &self.module.locals_signature_at(*type_actuals_idx).0;
                                    check_generic_instance(
                                        context,
                                        &struct_handle.type_formals,
                                        type_actuals,
                                    )
                                }
                                Exists(idx, type_actuals_idx)
                                | MoveFrom(idx, type_actuals_idx)
                                | MoveToSender(idx, type_actuals_idx)
                                | ImmBorrowGlobal(idx, type_actuals_idx)
                                | MutBorrowGlobal(idx, type_actuals_idx) => {
                                    let struct_def = self.module.struct_def_at(*idx);
                                    let struct_handle =
                                        self.module.struct_handle_at(struct_def.struct_handle);
                                    let type_actuals =
                                        &self.module.locals_signature_at(*type_actuals_idx).0;
                                    check_generic_instance(
                                        context,
                                        &struct_handle.type_formals,
                                        type_actuals,
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
}

// Checks if the given types are well defined and satisfy the given kind constraints in the given
// context.
fn check_generic_instance(
    context: (&[StructHandle], &[Kind]),
    constraints: &[Kind],
    type_actuals: &[SignatureToken],
) -> Vec<VMStatus> {
    let mut errors: Vec<_> = type_actuals
        .iter()
        .flat_map(|ty| check_signature_no_refs(context, ty))
        .collect();

    if constraints.len() != type_actuals.len() {
        errors.push(
            VMStatus::new(StatusCode::NUMBER_OF_TYPE_ACTUALS_MISMATCH).with_message(format!(
                "expected {} type actuals got {}",
                constraints.len(),
                type_actuals.len()
            )),
        );
        return errors;
    }

    let kinds = type_actuals
        .iter()
        .map(|ty| SignatureToken::kind(context, ty));
    errors.extend(
        constraints
            .iter()
            .zip(kinds)
            .zip(type_actuals.iter())
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
fn check_signature_no_refs(
    context: (&[StructHandle], &[Kind]),
    ty: &SignatureToken,
) -> Vec<VMStatus> {
    use SignatureToken::*;

    let (struct_handles, _) = context;

    match ty {
        U8 | U64 | U128 | Bool | ByteArray | Address | TypeParameter(_) => vec![],
        Reference(_) | MutableReference(_) => {
            // TODO: Prop tests expect us to NOT check the inner types.
            // Revisit this once we rework prop tests.
            vec![VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN)
                .with_message("reference not allowed".to_string())]
        }
        Struct(idx, type_actuals) => {
            let sh = &struct_handles[idx.0 as usize];
            check_generic_instance(context, &sh.type_formals, type_actuals)
        }
    }
}

/// Checks if the given type is well defined in the given context. References are only permitted
/// at the top level.
fn check_signature(context: (&[StructHandle], &[Kind]), ty: &SignatureToken) -> Vec<VMStatus> {
    use SignatureToken::*;

    match ty {
        Reference(inner) | MutableReference(inner) => check_signature_no_refs(context, inner),
        _ => check_signature_no_refs(context, ty),
    }
}
