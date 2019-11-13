// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::ModuleId;
use crate::test_helpers::assert_canonical_encode_decode;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_module_id_protobuf_roundtrip(module_id in any::<ModuleId>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ModuleId, ModuleId>(&module_id);
    }

    #[test]
    fn test_module_id_canonical_roundtrip(module_id in any::<ModuleId>()) {
        assert_canonical_encode_decode(module_id);
    }
}
