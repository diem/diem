// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use spec_lang::run_spec_lang_compiler;
use test_utils::{baseline_test::verify_or_update_baseline, DEFAULT_SENDER};

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let targets = vec![path.to_str().unwrap().to_string()];
    let deps = vec![];
    let address_opt = Some(DEFAULT_SENDER);

    let env = run_spec_lang_compiler(targets, deps, address_opt)?;
    let diags = if env.has_errors() {
        let mut writer = Buffer::no_color();
        env.report_errors(&mut writer);
        String::from_utf8_lossy(&writer.into_inner()).to_string()
    } else {
        "All good, no errors!".to_string()
    };
    let baseline_path = path.with_extension("exp");
    verify_or_update_baseline(baseline_path.as_path(), &diags)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
