// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_types::transaction::{TransactionInfo, Version};
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(version in any::<Version>(), txn_info in any::<TransactionInfo>()) {
        assert_encode_decode::<TransactionInfoSchema>(&version, &txn_info);
    }
}
