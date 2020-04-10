// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loaded_data::types::FatType;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip(ty in any::<FatType>()) {
        let bytes = lcs::to_bytes(&ty).unwrap();
        let s: FatType = lcs::from_bytes(&bytes).unwrap();
        assert_eq!(ty, s);
    }
}
