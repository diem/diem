// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::FakeExecutor;
use canonical_serialization::SimpleDeserializer;
use types::{
    access_path::VALIDATOR_SET_ACCESS_PATH, validator_public_keys::ValidatorPublicKeys,
    validator_set::ValidatorSet,
};

#[test]
fn load_genesis_validator_set() {
    let executor = FakeExecutor::from_genesis_file();
    let validator_set_bytes = executor
        .read_from_access_path(&VALIDATOR_SET_ACCESS_PATH)
        .unwrap();
    let validator_set: ValidatorSet =
        SimpleDeserializer::deserialize(&validator_set_bytes).unwrap();
    let expected_payload: Vec<ValidatorPublicKeys> = vec![];
    assert_eq!(validator_set.payload(), expected_payload.as_slice());
}
