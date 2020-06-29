// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_status::StatusCode;
use vm::{
    access::ModuleAccess,
    errors::{verification_error, PartialVMError},
    file_format::{Bytecode, CompiledModule, TableIndex},
    IndexKind,
};

pub struct UnusedEntryChecker<'a> {
    module: &'a CompiledModule,

    signatures: Vec<bool>,
}

impl<'a> UnusedEntryChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module,
            signatures: vec![false; module.signatures().len()],
        }
    }

    fn traverse_function_defs(&mut self) {
        use Bytecode::*;

        for func_def in self.module.function_defs() {
            let code = match &func_def.code {
                Some(code) => code,
                None => continue,
            };

            for bytecode in &code.code {
                match bytecode {
                    CallGeneric(idx) => {
                        let func_inst = self.module.function_instantiation_at(*idx);
                        self.signatures[func_inst.type_parameters.0 as usize] = true;
                    }
                    MutBorrowFieldGeneric(idx) | ImmBorrowFieldGeneric(idx) => {
                        let field_inst = self.module.field_instantiation_at(*idx);
                        self.signatures[field_inst.type_parameters.0 as usize] = true;
                    }
                    PackGeneric(idx)
                    | UnpackGeneric(idx)
                    | MutBorrowGlobalGeneric(idx)
                    | ImmBorrowGlobalGeneric(idx)
                    | ExistsGeneric(idx)
                    | MoveToGeneric(idx)
                    | MoveFromGeneric(idx) => {
                        let struct_inst = self.module.struct_instantiation_at(*idx);
                        self.signatures[struct_inst.type_parameters.0 as usize] = true;
                    }
                    _ => (),
                }
            }
        }
    }

    fn collect_errors<'b, F>(pool: &'b [bool], f: F) -> impl Iterator<Item = PartialVMError> + 'b
    where
        F: Fn(usize) -> PartialVMError + 'b,
    {
        pool.iter()
            .enumerate()
            .filter_map(move |(idx, visited)| if *visited { None } else { Some(f(idx)) })
    }

    pub fn verify(mut self) -> Vec<PartialVMError> {
        self.traverse_function_defs();

        Self::collect_errors(&self.signatures, |idx| {
            verification_error(
                StatusCode::UNUSED_LOCALS_SIGNATURE,
                IndexKind::Signature,
                idx as TableIndex,
            )
        })
        .collect()
    }
}
