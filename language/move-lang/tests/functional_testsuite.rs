// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use libra_types::account_address::AccountAddress as LibraAddress;
use move_lang::test_utils::stdlib_files;
use move_lang::{move_compile_no_report, shared::Address, to_bytecode::translate::CompiledUnit};
use std::{convert::TryFrom, io::Write, path::Path};
use tempfile::NamedTempFile;

struct MoveSourceCompiler {
    deps: Vec<String>,
    temp_files: Vec<NamedTempFile>,
}

impl MoveSourceCompiler {
    fn new(stdlib_modules_file_names: Vec<String>) -> Self {
        MoveSourceCompiler {
            deps: stdlib_modules_file_names,
            temp_files: vec![],
        }
    }
}

impl Compiler for MoveSourceCompiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String) -> ()>(
        &mut self,
        _log: Logger,
        address: LibraAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        let cur_file = NamedTempFile::new()?;
        cur_file.reopen()?.write_all(input.as_bytes())?;
        let cur_path = cur_file.path().to_str().unwrap().to_owned();
        self.temp_files.push(cur_file);

        let targets = &vec![cur_path.clone()];
        let sender = Some(Address::try_from(address.as_ref()).unwrap());
        let (files, units_or_errors) = move_compile_no_report(targets, &self.deps, sender)?;
        let unit = match units_or_errors {
            Err(errors) => {
                let error_buffer = move_lang::errors::report_errors_to_buffer(files, errors);
                bail!("{}", String::from_utf8(error_buffer).unwrap())
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
            CompiledUnit::Script(_, compiled_script) => ScriptOrModule::Script(compiled_script),
            CompiledUnit::Module(_, compiled_module) => {
                self.deps.push(cur_path);
                ScriptOrModule::Module(compiled_module)
            }
        })
    }
}

fn run_test(path: &Path) -> datatest_stable::Result<()> {
    let compiler = MoveSourceCompiler::new(stdlib_files());
    testsuite::functional_tests(compiler, path)
}

datatest_stable::harness!(run_test, "tests/functional", r".*\.move");
