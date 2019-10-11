use crate::language_storage::ModuleId;
use canonical_serialization::test_helper::assert_canonical_encode_decode;
use proptest::prelude::*;
use prost_ext::test_helpers::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn test_module_id_protobuf_roundtrip(module_id in any::<ModuleId>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ModuleId, ModuleId>(&module_id);
    }

    #[test]
    fn test_module_id_canonical_roundtrip(module_id in any::<ModuleId>()) {
        assert_canonical_encode_decode(&module_id);
    }
}
