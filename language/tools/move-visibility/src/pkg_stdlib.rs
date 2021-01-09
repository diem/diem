// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::{collections::BTreeSet, fs::File, path::Path};

use stdlib::{build_stdlib, compile_script, script_files};
use vm::file_format::{CompiledModule, CompiledScript};

use crate::pkg::Package;
use std::io::Write;
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

    pub fn run(&self, workdir: &Path) -> Result<()> {
        let graph = self.build_viz_graph();

        // dump the graph
        let mut file_graph = File::create(workdir.join("graph.dot"))?;
        file_graph.write_all(graph.to_dot().as_bytes())?;

        // dump friendship candidates
        let (fl_m2m, fl_m2f, fl_f2m, fl_f2f, _) = graph.friends();

        let mut file_m2m = File::create(workdir.join("viz_m2m.txt"))?;
        for (module_id, friends_m2m) in fl_m2m {
            writeln!(file_m2m, "{}:", module_id.name())?;
            for friend_module_id in friends_m2m {
                writeln!(file_m2m, "\t{}", friend_module_id.name())?;
            }
        }

        let mut file_m2f = File::create(workdir.join("viz_m2f.txt"))?;
        for (module_id, friends_m2f) in fl_m2f {
            writeln!(file_m2f, "{}:", module_id.name())?;
            for (friend_module_id, friend_function_name) in friends_m2f {
                writeln!(
                    file_m2f,
                    "\t{}::{}",
                    friend_module_id.name(),
                    friend_function_name
                )?;
            }
        }

        let mut file_f2m = File::create(workdir.join("viz_f2m.txt"))?;
        for ((module_id, function_name), friends_f2m) in fl_f2m {
            writeln!(file_f2m, "{}::{}:", module_id.name(), function_name)?;
            for friend_module_id in friends_f2m {
                writeln!(file_f2m, "\t{}", friend_module_id.name())?;
            }
        }

        let mut file_f2f = File::create(workdir.join("viz_f2f.txt"))?;
        for ((module_id, function_name), friends_f2f) in fl_f2f {
            writeln!(file_f2f, "{}::{}:", module_id.name(), function_name)?;
            for (friend_module_id, friend_function_name) in friends_f2f {
                writeln!(
                    file_f2f,
                    "\t{}::{}",
                    friend_module_id.name(),
                    friend_function_name
                )?;
            }
        }

        Ok(())
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
