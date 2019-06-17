// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::ResourceTransitiveChecker;
use proptest::prelude::*;
use vm::{checks::BoundsChecker, file_format::CompiledModule};

proptest! {
    #[test]
    fn valid_resource_transitivity(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(BoundsChecker::new(&module).verify().is_empty());
        let resource_checker = ResourceTransitiveChecker::new(&module);
        prop_assert!(resource_checker.verify().is_empty());
    }
}
