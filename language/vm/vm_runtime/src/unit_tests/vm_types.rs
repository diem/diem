// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::value::{MutVal, Value};
use canonical_serialization::SimpleDeserializer;
use types::{account_config::AccountResource, byte_array::ByteArray};

#[test]
fn account_type() {
    // mimic an Account
    let authentication_key = ByteArray::new(vec![5u8; 32]);
    let balance = 128u64;
    let received_events_count = 8u64;
    let sent_events_count = 16u64;
    let sequence_number = 32u64;

    let mut account_fields: Vec<MutVal> = Vec::new();
    account_fields.push(MutVal::bytearray(authentication_key.clone()));
    let mut coin_fields: Vec<MutVal> = Vec::new();
    coin_fields.push(MutVal::u64(balance));
    account_fields.push(MutVal::struct_(coin_fields.clone()));
    account_fields.push(MutVal::u64(received_events_count));
    account_fields.push(MutVal::u64(sent_events_count));
    account_fields.push(MutVal::u64(sequence_number));

    let account = Value::Struct(account_fields);
    let blob = &account.simple_serialize().expect("blob must serialize");

    let account_resource: AccountResource =
        SimpleDeserializer::deserialize(blob).expect("must deserialize");
    assert_eq!(*account_resource.authentication_key(), authentication_key);
    assert_eq!(account_resource.balance(), balance);
    assert_eq!(
        account_resource.received_events_count(),
        received_events_count
    );
    assert_eq!(account_resource.sent_events_count(), sent_events_count);
    assert_eq!(account_resource.sequence_number(), sequence_number);
}
