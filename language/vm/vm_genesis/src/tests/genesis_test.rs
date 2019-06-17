// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::encode_genesis_transaction_with_validator;
use canonical_serialization::SimpleDeserializer;
use crypto::signing::generate_keypair;
use proptest::{collection::vec, prelude::*};
use types::{
    access_path::VALIDATOR_SET_ACCESS_PATH, transaction::TransactionPayload,
    validator_public_keys::ValidatorPublicKeys, validator_set::ValidatorSet, write_set::WriteOp,
};

proptest! {
    #[test]
    fn test_validator_set_roundtrip(keys in vec(any::<ValidatorPublicKeys>(), 0..10)) {
         let (priv_key, pub_key) = generate_keypair();
        let writeset = match encode_genesis_transaction_with_validator(&priv_key, pub_key, keys.clone()).payload() {
            TransactionPayload::WriteSet(ws) => ws.clone(),
            _ => panic!("Unexpected Transaction"),
        };
        let (_, validator_entry) = writeset.iter().find(
            |(ap, _)| *ap == *VALIDATOR_SET_ACCESS_PATH
        ).cloned().unwrap();
        let validator_set_bytes = match validator_entry {
            WriteOp::Value(blob) => blob,
            _ => panic!("Unexpected WriteOp"),
        };
        let validator_set: ValidatorSet =
            SimpleDeserializer::deserialize(&validator_set_bytes).unwrap();

        prop_assert_eq!(validator_set.payload(), keys.as_slice());
    }
}
