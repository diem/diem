// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loaded_data::types::Type;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip(ty in any::<Type>()) {
        let bytes = lcs::to_bytes(&ty).unwrap();
        let s: Type = lcs::from_bytes(&bytes).unwrap();
        assert_eq!(ty, s);
    }
}
