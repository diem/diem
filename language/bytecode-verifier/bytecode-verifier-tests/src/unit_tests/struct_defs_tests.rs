// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::RecursiveStructDefChecker;
use move_binary_format::file_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_recursive_struct_defs(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(RecursiveStructDefChecker::verify_module(&module).is_ok());
    }
}
