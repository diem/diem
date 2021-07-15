// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_unit_test::UnitTestingConfig;
use structopt::*;

pub fn main() {
    let args = UnitTestingConfig::from_args();

    let test_plan = args.build_test_plan();
    if let Some(test_plan) = test_plan {
        args.run_and_report_unit_tests(test_plan, None, std::io::stdout())
            .unwrap();
    }
}
