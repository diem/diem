// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tests::suite, vault::VaultStorage, Capability, Error, Identity, Permission, Policy, Storage,
    Value,
};

/// A test for verifying VaultStorage properly implements the LibraSecureStorage API. This test
/// depends on running Vault, which can be done by using the provided docker run script in
/// `docker/vault/run.sh`
/// Note: because we only run a single vault backend for testing, we execute all tests sequentially
/// using the method below.
///
/// TODO(joshlind): see how we can refactor this to run each test individually. This would better
/// separate the testing of individual behaviours and help make failures easier to reason about.
/// It would also be good to avoid having to call reset_vault_storage between tests..
#[ignore]
#[test]
fn all_tests() {
    let mut storage = Box::new(setup_vault());

    // Test key/value related operations
    suite::run_test_suite(storage.as_mut(), "VaultStorage");
    reset_vault_storage(storage.as_mut());

    // Test cryptographic key related operations
    suite::create_get_and_test_key_pair(storage.as_mut());
    reset_vault_storage(storage.as_mut());

    suite::create_key_pair_twice(storage.as_mut());
    reset_vault_storage(storage.as_mut());

    suite::get_uncreated_key_pair(storage.as_mut());
    reset_vault_storage(storage.as_mut());
}

fn setup_vault() -> VaultStorage {
    let host = "http://localhost:8200".to_string();
    let token = "root_token".to_string();
    let mut storage = VaultStorage::new(host.clone(), token);
    reset_vault_storage(&mut storage);

    // @TODO davidiw, the following needs to be made generic but right now the creation of a token
    // / service is very backend specific.
    let reader: String = "reader".to_string();
    let writer: String = "writer".to_string();

    let anyone = Policy::public();
    let root = Policy::new(vec![]);
    let partial = Policy::new(vec![
        Permission::new(Identity::User(reader.clone()), vec![Capability::Read]),
        Permission::new(
            Identity::User(writer.clone()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);
    let full = Policy::new(vec![
        Permission::new(
            Identity::User(reader.clone()),
            vec![Capability::Read, Capability::Write],
        ),
        Permission::new(
            Identity::User(writer.clone()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);

    // Initialize data and policies

    storage.create("anyone", Value::U64(1), &anyone).unwrap();
    storage.create("root", Value::U64(2), &root).unwrap();
    storage.create("partial", Value::U64(3), &partial).unwrap();
    storage.create("full", Value::U64(4), &full).unwrap();

    // Verify initial reading works correctly

    assert_eq!(storage.get("anyone"), Ok(Value::U64(1)));
    assert_eq!(storage.get("root"), Ok(Value::U64(2)));
    assert_eq!(storage.get("partial"), Ok(Value::U64(3)));
    assert_eq!(storage.get("full"), Ok(Value::U64(4)));

    let writer_token = storage.client.create_token(vec![&writer]).unwrap();
    let mut writer = VaultStorage::new(host.clone(), writer_token);
    assert_eq!(writer.get("anyone"), Ok(Value::U64(1)));
    assert_eq!(writer.get("root"), Err(Error::PermissionDenied));
    assert_eq!(writer.get("partial"), Ok(Value::U64(3)));
    assert_eq!(writer.get("full"), Ok(Value::U64(4)));

    let reader_token = storage.client.create_token(vec![&reader]).unwrap();
    let mut reader = VaultStorage::new(host, reader_token);
    assert_eq!(reader.get("anyone"), Ok(Value::U64(1)));
    assert_eq!(reader.get("root"), Err(Error::PermissionDenied));
    assert_eq!(reader.get("partial"), Ok(Value::U64(3)));
    assert_eq!(reader.get("full"), Ok(Value::U64(4)));

    // Attempt writes followed by reads for correctness

    writer.set("anyone", Value::U64(5)).unwrap();
    assert_eq!(
        writer.set("root", Value::U64(6)),
        Err(Error::PermissionDenied)
    );
    writer.set("partial", Value::U64(7)).unwrap();
    writer.set("full", Value::U64(8)).unwrap();

    assert_eq!(storage.get("anyone"), Ok(Value::U64(5)));
    assert_eq!(storage.get("root"), Ok(Value::U64(2)));
    assert_eq!(storage.get("partial"), Ok(Value::U64(7)));
    assert_eq!(storage.get("full"), Ok(Value::U64(8)));

    reader.set("anyone", Value::U64(9)).unwrap();
    assert_eq!(
        reader.set("root", Value::U64(10)),
        Err(Error::PermissionDenied)
    );
    assert_eq!(
        reader.set("partial", Value::U64(11)),
        Err(Error::PermissionDenied)
    );
    reader.set("full", Value::U64(12)).unwrap();

    assert_eq!(storage.get("anyone"), Ok(Value::U64(9)));
    assert_eq!(storage.get("root"), Ok(Value::U64(2)));
    assert_eq!(storage.get("partial"), Ok(Value::U64(7)));
    assert_eq!(storage.get("full"), Ok(Value::U64(12)));

    reset_vault_storage(&mut storage);
    storage
}

/// Resets the internal state of vault for testing purposes (i.e., clears all stored data).
fn reset_vault_storage(storage: &mut VaultStorage) {
    storage
        .reset()
        .expect("Failed to reset vault storage state!");
}
