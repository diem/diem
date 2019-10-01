use canonical_serialization::test_helper::assert_canonical_encode_decode;
use crate::block_metadata::BlockMetaData;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_block_metadata_canonical_serialization(data in any::<BlockMetaData>()) {
        assert_canonical_encode_decode(&data);
    }
}
