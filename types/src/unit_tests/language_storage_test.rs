// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{language_storage::ModuleId, test_helpers::assert_canonical_encode_decode};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_module_id_canonical_roundtrip(module_id in any::<ModuleId>()) {
        assert_canonical_encode_decode(module_id);
    }
}
