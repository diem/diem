// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::DuplicationChecker;
use proptest::prelude::*;
use vm::{checks::BoundsChecker, file_format::CompiledModule};

proptest! {
    #[test]
    fn valid_duplication(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(BoundsChecker::new(&module).verify().is_empty());
        let duplication_checker = DuplicationChecker::new(&module);
        prop_assert!(!duplication_checker.verify().is_empty());
    }
}
