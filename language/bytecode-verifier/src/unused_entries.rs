// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::verification_error,
    file_format::{Bytecode, CompiledModule, StructFieldInformation},
    IndexKind,
};

pub struct UnusedEntryChecker<'a> {
    module: &'a CompiledModule,

    field_defs: Vec<bool>,
    locals_signatures: Vec<bool>,
    type_signatures: Vec<bool>,
}

impl<'a> UnusedEntryChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module,
            field_defs: vec![false; module.field_defs().len()],
            locals_signatures: vec![false; module.locals_signatures().len()],
            type_signatures: vec![false; module.type_signatures().len()],
        }
    }

    fn traverse_function_defs(&mut self) {
        use Bytecode::*;

        for func_def in self.module.function_defs() {
            if func_def.is_native() {
                continue;
            }
            self.locals_signatures[func_def.code.locals.0 as usize] = true;

            for bytecode in &func_def.code.code {
                match bytecode {
                    Call(_, idx)
                    | Pack(_, idx)
                    | Unpack(_, idx)
                    | MutBorrowGlobal(_, idx)
                    | ImmBorrowGlobal(_, idx)
                    | Exists(_, idx)
                    | MoveToSender(_, idx)
                    | MoveFrom(_, idx) => {
                        self.locals_signatures[idx.0 as usize] = true;
                    }
                    _ => (),
                }
            }
        }
    }

    fn traverse_struct_defs(&mut self) {
        for struct_def in self.module.struct_defs() {
            match struct_def.field_information {
                StructFieldInformation::Native => (),
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => {
                    let start = fields.0 as usize;
                    let end = start + (field_count as usize);

                    for i in start..end {
                        self.field_defs[i] = true;

                        let field_def = &self.module.field_defs()[i];
                        self.type_signatures[field_def.signature.0 as usize] = true;
                    }
                }
            }
        }
    }

    fn collect_errors<'b, F>(pool: &'b [bool], f: F) -> impl Iterator<Item = VMStatus> + 'b
    where
        F: Fn(usize) -> VMStatus + 'b,
    {
        pool.iter()
            .enumerate()
            .filter_map(move |(idx, visited)| if *visited { None } else { Some(f(idx)) })
    }

    pub fn verify(mut self) -> Vec<VMStatus> {
        self.traverse_struct_defs();
        self.traverse_function_defs();

        let iter_field_defs = Self::collect_errors(&self.field_defs, |idx| {
            verification_error(IndexKind::FieldDefinition, idx, StatusCode::UNUSED_FIELD)
        });

        let iter_locals_signatures = Self::collect_errors(&self.locals_signatures, |idx| {
            verification_error(
                IndexKind::LocalsSignature,
                idx,
                StatusCode::UNUSED_LOCALS_SIGNATURE,
            )
        });

        let iter_type_signatures = Self::collect_errors(&self.type_signatures, |idx| {
            verification_error(
                IndexKind::TypeSignature,
                idx,
                StatusCode::UNUSED_TYPE_SIGNATURE,
            )
        });

        iter_field_defs
            .chain(iter_locals_signatures)
            .chain(iter_type_signatures)
            .collect()
    }
}
