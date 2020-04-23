// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::prelude::*;
use vm::file_format::{empty_module, CompiledModule, CompiledModuleMut, Signature, SignatureToken};

proptest! {
    #[test]
    fn serializer_roundtrip(module in CompiledModule::valid_strategy(20)) {
        let mut serialized = Vec::with_capacity(2048);
        module.serialize(&mut serialized).expect("serialization should work");

        let deserialized_module = CompiledModule::deserialize(&serialized)
            .expect("deserialization should work");
        prop_assert_eq!(module, deserialized_module);
    }
}

proptest! {
    // Generating arbitrary compiled modules is really slow, possibly because of
    // https://github.com/AltSysrq/proptest/issues/143.
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Make sure that garbage inputs don't crash the serializer and deserializer.
    #[test]
    fn garbage_inputs(module in any_with::<CompiledModuleMut>(16)) {
        let mut serialized = Vec::with_capacity(65536);
        module.serialize(&mut serialized).expect("serialization should work");

        let deserialized_module = CompiledModuleMut::deserialize_no_check_bounds(&serialized)
            .expect("deserialization should work");
        prop_assert_eq!(module, deserialized_module);
    }
}

#[test]
fn serialize_and_deserialize_deeply_nested_types() {
    const RECURSION_DEPTH: usize = 10000;

    let mut m = empty_module();
    let mut ty = SignatureToken::U8;
    // TODO: turn the number of iterations up once we fix Drop for SignatureToken.
    for _ in 0..RECURSION_DEPTH {
        ty = SignatureToken::Vector(Box::new(ty));
    }
    m.signatures.push(Signature(vec![ty]));

    // TODO: Run bounds checker here once we get that fixed .
    let mut serialized = Vec::with_capacity(2048);
    m.serialize(&mut serialized)
        .expect("serialization should work");

    CompiledModuleMut::deserialize_no_check_bounds(&serialized)
        .expect("deserialization should work");
}
