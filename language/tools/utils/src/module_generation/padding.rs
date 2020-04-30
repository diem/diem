// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module_generation::{options::ModuleGeneratorOptions, utils::random_string};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use rand::{rngs::StdRng, Rng, SeedableRng};
use vm::file_format::{Bytecode, CompiledModuleMut, Signature};

///////////////////////////////////////////////////////////////////////////
// Padding of tables in compiled modules
///////////////////////////////////////////////////////////////////////////

pub struct Pad {
    gen: StdRng,
    table_size: usize,
    options: ModuleGeneratorOptions,
}

impl Pad {
    pub fn pad(table_size: usize, module: &mut CompiledModuleMut, options: ModuleGeneratorOptions) {
        let seed: [u8; 32] = [1; 32];
        let mut slf = Self {
            gen: StdRng::from_seed(seed),
            table_size,
            options,
        };
        slf.pad_cosntant_table(module);
        slf.pad_identifier_table(module);
        slf.pad_address_identifier_table(module);
        slf.pad_signatures(module);
        slf.pad_function_bodies(module);
    }

    fn pad_cosntant_table(&mut self, module: &mut CompiledModuleMut) {
        // TODO actual constant generation
        module.constant_pool = vec![]
    }

    fn pad_identifier_table(&mut self, module: &mut CompiledModuleMut) {
        module.identifiers = (0..(self.table_size + module.identifiers.len()))
            .map(|_| {
                let len = self.gen.gen_range(10, self.options.max_string_size);
                Identifier::new(random_string(&mut self.gen, len)).unwrap()
            })
            .collect()
    }

    fn pad_address_identifier_table(&mut self, module: &mut CompiledModuleMut) {
        module.address_identifiers = (0..(self.table_size + module.address_identifiers.len()))
            .map(|_| AccountAddress::random())
            .collect()
    }

    fn pad_function_bodies(&mut self, module: &mut CompiledModuleMut) {
        for fdef in module.function_defs.iter_mut() {
            if let Some(code) = &mut fdef.code {
                code.code = vec![
                    Bytecode::LdTrue,
                    Bytecode::LdTrue,
                    Bytecode::Pop,
                    Bytecode::Pop,
                    Bytecode::Ret,
                ]
            }
        }
    }

    // Ensure that locals signatures always contain an empty signature
    fn pad_signatures(&mut self, module: &mut CompiledModuleMut) {
        if module.signatures.iter().all(|v| !v.is_empty()) {
            module.signatures.push(Signature(Vec::new()));
        }
    }
}
