// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_bytecode_verifier::ResourceTransitiveChecker;
use libra_vm::file_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_resource_transitivity(module in CompiledModule::valid_strategy(20)) {
        let resource_checker = ResourceTransitiveChecker::new(&module);
        prop_assert!(resource_checker.verify().is_empty());
    }
}
