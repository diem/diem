// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::DuplicationChecker;
use proptest::prelude::*;
use vm::file_format::CompiledModule;

proptest! {
    #[test]
    fn valid_duplication(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(DuplicationChecker::verify(&module).is_ok());
    }
}
