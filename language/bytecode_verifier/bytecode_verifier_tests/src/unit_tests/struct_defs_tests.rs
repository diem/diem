// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_bytecode_verifier::RecursiveStructDefChecker;
use libra_vm::file_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_recursive_struct_defs(module in CompiledModule::valid_strategy(20)) {
        let recursive_checker = RecursiveStructDefChecker::new(&module);
        prop_assert!(recursive_checker.verify().is_empty());
    }
}
