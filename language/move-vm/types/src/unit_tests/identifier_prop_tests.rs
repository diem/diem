// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::identifier::resource_storage_key;
use proptest::prelude::*;
use vm::{
    access::ModuleAccess,
    file_format::{CompiledModule, StructDefinitionIndex, TableIndex},
};

proptest! {
    #[test]
    fn identifier_serializer_roundtrip(module in CompiledModule::valid_strategy(20)) {
        let module_id = module.self_id();
        let deserialized_module_id = {
            let serialized_key = lcs::to_bytes(&module_id).unwrap();
            lcs::from_bytes(&serialized_key).expect("Deserialize should work")
        };
        prop_assert_eq!(module_id, deserialized_module_id);

        for i in 0..module.struct_defs().len() {
            let struct_key = resource_storage_key(&module, StructDefinitionIndex::new(i as TableIndex), vec![]);
            let deserialized_struct_key = {
                let serialized_key = lcs::to_bytes(&struct_key).unwrap();
                lcs::from_bytes(&serialized_key).expect("Deserialize should work")
            };
            prop_assert_eq!(struct_key, deserialized_struct_key);
        }
    }
}
