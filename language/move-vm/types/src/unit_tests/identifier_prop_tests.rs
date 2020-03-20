// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::prelude::*;
use vm::file_format::CompiledModule;

proptest! {
    #[test]
    fn identifier_serializer_roundtrip(module in CompiledModule::valid_strategy(20)) {
        let module_id = module.self_id();
        let deserialized_module_id = {
            let serialized_key = lcs::to_bytes(&module_id).unwrap();
            lcs::from_bytes(&serialized_key).expect("Deserialize should work")
        };
        prop_assert_eq!(module_id, deserialized_module_id);
    }
}
