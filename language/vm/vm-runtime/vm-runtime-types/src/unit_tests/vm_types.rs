// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::value::{Struct, Value};
use libra_types::{account_config::AccountResource, byte_array::ByteArray, event::EventKey};

#[test]
fn account_type() {
    // mimic an Account
    let authentication_key = ByteArray::new(vec![5u8; 32]);
    let balance = 128u64;
    let received_events_count = 8u64;
    let sent_events_count = 16u64;
    let sequence_number = 32u64;
    let sent_events_key = ByteArray::new(EventKey::random().to_vec());
    let recv_events_key = ByteArray::new(EventKey::random().to_vec());
    let evt_count = 32u64;

    let mut account_fields: Vec<Value> = Vec::new();
    account_fields.push(Value::byte_array(authentication_key.clone()));
    let mut coin_fields: Vec<Value> = Vec::new();
    coin_fields.push(Value::u64(balance));
    account_fields.push(Value::struct_(Struct::new(coin_fields)));
    account_fields.push(Value::bool(false));
    account_fields.push(Value::bool(false));
    account_fields.push(Value::struct_(Struct::new(vec![
        Value::u64(received_events_count),
        Value::byte_array(recv_events_key.clone()),
    ])));
    account_fields.push(Value::struct_(Struct::new(vec![
        Value::u64(sent_events_count),
        Value::byte_array(sent_events_key.clone()),
    ])));
    account_fields.push(Value::u64(sequence_number));
    account_fields.push(Value::struct_(Struct::new(vec![Value::u64(evt_count)])));

    let account = Value::struct_(Struct::new(account_fields));
    let blob = &account.simple_serialize().expect("blob must serialize");

    let account_resource: AccountResource = lcs::from_bytes(&blob).expect("must deserialize");
    assert_eq!(*account_resource.authentication_key(), authentication_key);
    assert_eq!(account_resource.balance(), balance);
    assert_eq!(
        account_resource.sent_events().key().as_bytes(),
        sent_events_key.as_bytes()
    );
    assert_eq!(
        account_resource.received_events().count(),
        received_events_count
    );
    assert_eq!(
        account_resource.received_events().key().as_bytes(),
        recv_events_key.as_bytes()
    );
    assert_eq!(account_resource.sent_events().count(), sent_events_count);
    assert_eq!(account_resource.sequence_number(), sequence_number);
}
