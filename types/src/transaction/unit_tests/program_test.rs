// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_helpers::assert_canonical_encode_decode;
use crate::transaction::{program::Program, transaction_argument::TransactionArgument};
use proptest::prelude::*;

proptest! {
    #[test]
    fn program_round_trip_canonical_serialization(program in any::<Program>()) {
        assert_canonical_encode_decode(program);
    }

    #[test]
    fn transaction_arguments_round_trip_canonical_serialization(
        transaction_argument in any::<TransactionArgument>()
    ) {
        assert_canonical_encode_decode(transaction_argument);
    }
}
