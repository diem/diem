// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use move_model::{run_bytecode_model_builder, run_model_builder};
use move_prover_test_utils::baseline_test::verify_or_update_baseline;

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let targets = vec![path.to_str().unwrap().to_string()];
    let env = run_model_builder(&targets, &[])?;
    let diags = if env.has_errors() {
        let mut writer = Buffer::no_color();
        env.report_errors(&mut writer);
        String::from_utf8_lossy(&writer.into_inner()).to_string()
    } else {
        // check that translating from bytecodes also works + yields similar results
        let modules = env.get_bytecode_modules().cloned().collect();
        let bytecode_env = run_bytecode_model_builder(modules)?;
        assert_eq!(bytecode_env.get_module_count(), env.get_module_count());
        for m in bytecode_env.get_modules() {
            let other_m = env.get_module(m.get_id());
            assert_eq!(m.get_function_count(), other_m.get_function_count());
            assert_eq!(m.get_struct_count(), other_m.get_struct_count());
        }

        "All good, no errors!".to_string()
    };
    let baseline_path = path.with_extension("exp");
    verify_or_update_baseline(baseline_path.as_path(), &diags)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
