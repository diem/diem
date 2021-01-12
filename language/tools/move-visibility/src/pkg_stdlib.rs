// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeSet, path::Path};

use stdlib::{build_stdlib, compile_script, script_files};
use vm::file_format::{CompiledModule, CompiledScript};

use crate::pkg::Package;
use vm::access::ModuleAccess;

// Exclude these files for call-dependency analysis
const BASE_MODULE_NAMES: [&str; 11] = [
    // core basics
    "Errors",
    "Event",
    "FixedPoint32",
    "Hash",
    "BCS",
    "Option",
    "Vector",
    "Signer",
    // diem basics
    "Roles",
    "CoreAddresses",
    "DiemTimestamp",
];

pub struct PackageStdlib {
    modules: Vec<CompiledModule>,
    scripts: Vec<(String, CompiledScript)>,
}

impl PackageStdlib {
    pub fn new() -> Self {
        let module_exclusion: BTreeSet<&str> = BASE_MODULE_NAMES.iter().copied().collect();

        let modules = build_stdlib()
            .into_iter()
            .filter_map(|(_, module)| {
                if module_exclusion.contains(module.name().as_str()) {
                    None
                } else {
                    Some(module)
                }
            })
            .collect();

        let scripts = script_files()
            .into_iter()
            .map(|file| {
                let script_bytes = compile_script(file.clone());
                (
                    Path::new(&file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    CompiledScript::deserialize(&script_bytes).unwrap(),
                )
            })
            .collect();

        Self { modules, scripts }
    }
}

impl Package for PackageStdlib {
    fn get_modules(&self) -> &[CompiledModule] {
        &self.modules
    }

    fn get_scripts(&self) -> &[(String, CompiledScript)] {
        &self.scripts
    }
}
