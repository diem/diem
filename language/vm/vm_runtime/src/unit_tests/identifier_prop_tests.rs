// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::identifier::resource_storage_key;
use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use proptest::prelude::*;
use vm::{
    access::ModuleAccess,
    file_format::{CompiledModule, StructDefinitionIndex, TableIndex},
};

proptest! {
    #[test]
    fn identifier_serializer_roundtrip(module in CompiledModule::valid_strategy(20)) {
        let code_key = module.self_code_key();
        let deserialized_code_key = {
            let serialized_key = SimpleSerializer::<Vec<u8>>::serialize(&code_key).unwrap();
            SimpleDeserializer::deserialize(&serialized_key).expect("Deserialize should work")
        };
        prop_assert_eq!(code_key, deserialized_code_key);

        for i in 0..module.struct_defs().len() {
            let struct_key = resource_storage_key(&module, StructDefinitionIndex::new(i as TableIndex));
            let deserialized_struct_key = {
                let serialized_key = SimpleSerializer::<Vec<u8>>::serialize(&struct_key).unwrap();
                SimpleDeserializer::deserialize(&serialized_key).expect("Deserialize should work")
            };
            prop_assert_eq!(struct_key, deserialized_struct_key);
        }
    }
}
