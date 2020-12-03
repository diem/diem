// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::ResourceTransitiveChecker;
use proptest::prelude::*;
use vm::file_format::CompiledModule;

proptest! {
    #[test]
    fn valid_resource_transitivity(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(ResourceTransitiveChecker::verify_module(&module).is_ok());
    }
}
