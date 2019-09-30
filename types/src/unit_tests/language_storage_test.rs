use crate::language_storage::ModuleId;
use libra_canonical_serialization::test_helper::assert_canonical_encode_decode;
use libra_proto_conv::test_helper::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_module_id_protobuf_roundtrip(module_id in any::<ModuleId>()) {
        assert_protobuf_encode_decode(&module_id);
    }

    #[test]
    fn test_module_id_canonical_roundtrip(module_id in any::<ModuleId>()) {
        assert_canonical_encode_decode(&module_id);
    }
}
