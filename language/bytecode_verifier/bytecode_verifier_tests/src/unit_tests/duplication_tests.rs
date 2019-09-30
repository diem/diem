// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_bytecode_verifier::DuplicationChecker;
use libra_vm::file_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_duplication(module in CompiledModule::valid_strategy(20)) {
        let duplication_checker = DuplicationChecker::new(&module);
        prop_assert!(!duplication_checker.verify().is_empty());
    }
}
