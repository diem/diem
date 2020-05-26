// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use libra_types::account_address::AccountAddress as LibraAddress;
use move_core_types::fs::{FileName, AFS, FS};
use move_lang::{
    compiled_unit::CompiledUnit,
    move_compile_no_report,
    shared::Address,
    test_utils::{read_bool_var, stdlib_files, FUNCTIONAL_TEST_DIR},
};
use std::{convert::TryFrom, fmt, fs, path::Path};

struct MoveSourceCompiler {
    deps: Vec<String>,
    fs: AFS,
}

impl MoveSourceCompiler {
    fn new(stdlib_modules_file_names: Vec<String>) -> Self {
        let fs = AFS::in_memory();
        for stdlib_modules_file_names in &stdlib_modules_file_names {
            fs.store(
                FileName::new(&stdlib_modules_file_names),
                fs::read_to_string(&stdlib_modules_file_names).unwrap(),
            )
            .unwrap();
        }

        MoveSourceCompiler {
            deps: stdlib_modules_file_names,
            fs,
        }
    }
}

#[derive(Debug)]
struct MoveSourceCompilerError(pub String);

impl fmt::Display for MoveSourceCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n\n{}", self.0)
    }
}

impl std::error::Error for MoveSourceCompilerError {}

impl Compiler for MoveSourceCompiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String) -> ()>(
        &mut self,
        _log: Logger,
        address: LibraAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        let target = FileName::new(&format!("target_{}.move", rand::random::<u64>()));
        self.fs.store(target, input.to_owned())?;

        let sender_addr = Address::try_from(address.as_ref()).unwrap();
        let targets = &vec![target.name()];
        let sender = Some(sender_addr);
        let (files, units_or_errors) =
            move_compile_no_report(targets, &self.deps, sender, &self.fs)?;
        let unit = match units_or_errors {
            Err(errors) => {
                let error_buffer = if read_bool_var(testsuite::PRETTY) {
                    move_lang::errors::report_errors_to_color_buffer(files, errors)
                } else {
                    move_lang::errors::report_errors_to_buffer(files, errors)
                };
                return Err(
                    MoveSourceCompilerError(String::from_utf8(error_buffer).unwrap()).into(),
                );
            }
            Ok(mut units) => {
                let len = units.len();
                if len != 1 {
                    bail!("Invalid input. Expected 1 compiled unit but got {}", len)
                }
                units.pop().unwrap()
            }
        };

        Ok(match unit {
            CompiledUnit::Script { script, .. } => ScriptOrModule::Script(script),
            CompiledUnit::Module { module, .. } => {
                let input = format!("address {} {{\n{}\n}}", sender_addr, input);
                self.fs.delete(target)?;
                self.fs.store(target, input)?;
                self.deps.push(target.name());
                ScriptOrModule::Module(module)
            }
        })
    }

    fn use_staged_genesis(&self) -> bool {
        false
    }
}

fn functional_testsuite(path: &Path) -> datatest_stable::Result<()> {
    testsuite::functional_tests(MoveSourceCompiler::new(stdlib_files()), path)
}

datatest_stable::harness!(functional_testsuite, FUNCTIONAL_TEST_DIR, r".*\.move");
