// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::ability_field_requirements;
use move_binary_format::file_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_ability_transitivity(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(ability_field_requirements::verify_module(&module).is_ok());
    }
}
