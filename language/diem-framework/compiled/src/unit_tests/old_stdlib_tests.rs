// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{old_stdlib_modules, stdlib_modules, GitHash, StdLibOptions};

#[test]
fn old_stdlib_tests() {
    let master_modules = stdlib_modules(StdLibOptions::Compiled);
    let (_, old_modules) =
        old_stdlib_modules(GitHash::from_hex("5e81a74fcfae5e1be8632cb9dea5f0c7f1191050").unwrap())
            .unwrap();
    assert_eq!(master_modules.compiled_modules, &old_modules);
}
