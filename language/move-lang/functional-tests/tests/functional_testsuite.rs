// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use diem_types::account_address::AccountAddress as DiemAddress;
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use move_command_line_common::env::read_bool_env_var;
use move_lang::{
    self, compiled_unit::CompiledUnit, diagnostics, Compiler as MoveCompiler, Flags,
    FullyCompiledProgram, PASS_COMPILATION,
};
use once_cell::sync::Lazy;
use std::{fmt, io::Write, path::Path};
use tempfile::NamedTempFile;

pub const STD_LIB_DIR: &str = "../../diem-framework/modules";
pub const FUNCTIONAL_TEST_DIR: &str = "tests";

struct MoveSourceCompiler<'a> {
    pre_compiled_deps: &'a FullyCompiledProgram,
    deps: Vec<String>,
    temp_files: Vec<NamedTempFile>,
}

impl<'a> MoveSourceCompiler<'a> {
    fn new(pre_compiled_deps: &'a FullyCompiledProgram) -> Self {
        MoveSourceCompiler {
            pre_compiled_deps,
            deps: vec![],
            temp_files: vec![],
        }
    }

    fn move_compile_with_stdlib(
        &self,
        targets: &[String],
    ) -> anyhow::Result<(
        diagnostics::FilesSourceText,
        Result<Vec<CompiledUnit>, diagnostics::Diagnostics>,
    )> {
        let (files, comments_and_compiler_res) = MoveCompiler::new(targets, &self.deps)
            .set_pre_compiled_lib(self.pre_compiled_deps)
            .run::<PASS_COMPILATION>()?;
        match comments_and_compiler_res {
            Err(diags) => Ok((files, Err(diags))),
            Ok((_comments, move_compiler)) => {
                let (units, warnings) = move_compiler.into_compiled_units();
                if !warnings.is_empty() {
                    Ok((files, Err(warnings)))
                } else {
                    Ok((files, Ok(units)))
                }
            }
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

impl<'a> Compiler for MoveSourceCompiler<'a> {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String)>(
        &mut self,
        _log: Logger,
        _address: DiemAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        let cur_file = NamedTempFile::new()?;
        cur_file.reopen()?.write_all(input.as_bytes())?;
        let cur_path = cur_file.path().to_str().unwrap().to_owned();

        let targets = &vec![cur_path.clone()];
        let (mut files, units_or_diags) = self.move_compile_with_stdlib(targets)?;
        let unit = match units_or_diags {
            Err(diags) => {
                for (file_name, text) in &self.pre_compiled_deps.files {
                    // TODO This is bad. Rethink this when errors are redone
                    if !files.contains_key(file_name) {
                        files.insert(*file_name, text.clone());
                    }
                }

                let diags_buffer = if read_bool_env_var(move_command_line_common::testing::PRETTY) {
                    diagnostics::report_diagnostics_to_color_buffer(&files, diags)
                } else {
                    diagnostics::report_diagnostics_to_buffer(&files, diags)
                };
                return Err(
                    MoveSourceCompilerError(String::from_utf8(diags_buffer).unwrap()).into(),
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
            CompiledUnit::Script { script, .. } => ScriptOrModule::Script(None, script),
            CompiledUnit::Module { module, .. } => {
                cur_file.reopen()?.write_all(input.as_bytes())?;
                self.temp_files.push(cur_file);
                self.deps.push(cur_path);
                ScriptOrModule::Module(module)
            }
        })
    }

    fn use_compiled_genesis(&self) -> bool {
        false
    }
}

static DIEM_PRECOMPILED_STDLIB: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let program_res = move_lang::construct_pre_compiled_lib(
        &diem_framework::diem_stdlib_files(),
        None,
        Flags::empty().set_sources_shadow_deps(false),
    )
    .unwrap();
    match program_res {
        Ok(stdlib) => stdlib,
        Err((files, diags)) => {
            eprintln!("!!!Standard library failed to compile!!!");
            diagnostics::report_diagnostics(&files, diags)
        }
    }
});

fn functional_testsuite(path: &Path) -> datatest_stable::Result<()> {
    testsuite::functional_tests(MoveSourceCompiler::new(&*DIEM_PRECOMPILED_STDLIB), path)
}

datatest_stable::harness!(functional_testsuite, FUNCTIONAL_TEST_DIR, r".*\.move$");
