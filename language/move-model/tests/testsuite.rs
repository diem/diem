// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan_reporting::{diagnostic::Severity, term::termcolor::Buffer};
use move_binary_format::{
    access::ModuleAccess,
    file_format::{FunctionDefinitionIndex, StructDefinitionIndex},
};
use move_command_line_common::testing::EXP_EXT;
use move_model::{run_bytecode_model_builder, run_model_builder};
use move_prover_test_utils::baseline_test::verify_or_update_baseline;
use std::path::Path;

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let targets = vec![path.to_str().unwrap().to_string()];
    let env = run_model_builder(&targets, &[])?;
    let diags = if env.diag_count(Severity::Warning) > 0 {
        let mut writer = Buffer::no_color();
        env.report_diag(&mut writer, Severity::Warning);
        String::from_utf8_lossy(&writer.into_inner()).to_string()
    } else {
        // check that translating from bytecodes also works + yields similar results
        let modules = env.get_bytecode_modules();
        let bytecode_env = run_bytecode_model_builder(modules)?;
        assert_eq!(bytecode_env.get_module_count(), env.get_module_count());
        for m in bytecode_env.get_modules() {
            let raw_module = m.get_verified_module();
            let other_m = env
                .find_module_by_language_storage_id(&raw_module.self_id())
                .expect("Module not found");
            assert_eq!(m.get_function_count(), other_m.get_function_count());
            // other_m can have ghost structs, so only check that we have at least as many
            // structs as in bytecode.
            assert!(m.get_struct_count() <= other_m.get_struct_count());
            for (i, _) in raw_module.struct_defs().iter().enumerate() {
                let idx = StructDefinitionIndex(i as u16);
                let s = m.get_struct_by_def_idx(idx);
                let other_s = other_m.get_struct_by_def_idx(idx);
                assert_eq!(s.get_field_count(), other_s.get_field_count());
                for f in s.get_fields() {
                    let other_f = other_s.get_field_by_offset(f.get_offset());
                    assert_eq!(f.get_identifier(), other_f.get_identifier());
                }
            }
            for (i, _) in raw_module.function_defs().iter().enumerate() {
                let idx = FunctionDefinitionIndex(i as u16);
                let fun = m.get_function(m.try_get_function_id(idx).expect("Function not found"));
                let other_fun = other_m.get_function(
                    other_m
                        .try_get_function_id(idx)
                        .expect("Function not found"),
                );
                assert_eq!(fun.get_identifier(), other_fun.get_identifier())
            }
        }

        "All good, no errors!".to_string()
    };
    let baseline_path = path.with_extension(EXP_EXT);
    verify_or_update_baseline(baseline_path.as_path(), &diags)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
