// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_unit_test::UnitTestingConfig;

move_unit_test::register_move_unit_tests!(
    UnitTestingConfig::default_with_bound(Some(100_000)),
    ".",
    r".*\.move$"
);
