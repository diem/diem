// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tests::suite, vault::VaultStorage, Capability, Error, Identity, KVStorage, Permission, Policy,
    Value,
};

const VAULT_HOST: &str = "http://localhost:8200";
const VAULT_ROOT_TOKEN: &str = "root_token";

/// A test for verifying VaultStorage properly implements the LibraSecureStorage API. This test
/// depends on running Vault, which can be done by using the provided docker run script in
/// `docker/vault/run.sh`
// There can only be one vault test as reset called on one instance can break another test
#[test]
#[ignore]
fn execute_storage_tests_vault() {
    let mut storage = VaultStorage::new(VAULT_HOST.into(), VAULT_ROOT_TOKEN.into(), None);
    storage.reset_and_clear().unwrap();

    test_vault(&mut storage);
    suite::execute_all_storage_tests(&mut storage);

    let mut storage0 = VaultStorage::new(
        VAULT_HOST.into(),
        VAULT_ROOT_TOKEN.into(),
        Some("test0".into()),
    );
    let mut storage1 = VaultStorage::new(
        VAULT_HOST.into(),
        VAULT_ROOT_TOKEN.into(),
        Some("test1".into()),
    );

    test_vault(&mut storage0);
    test_vault(&mut storage1);
    suite::execute_all_storage_tests(&mut storage0);
    suite::execute_all_storage_tests(&mut storage1);
    storage.reset_and_clear().unwrap();
}

/// Creates and returns a new VaultStorage instance for testing purposes.
pub fn test_vault(storage: &mut VaultStorage) {
    // TODO(davidiw,joshlind): evaluate other systems and determine if create_token can be on the
    // Storage / KV interface. And then refactor this method to make it cleaner and easier to reason
    // about.
    let reader: String = "reader".into();
    let writer: String = "writer".into();

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

    let writer_token = storage.create_token(vec![&writer]).unwrap();
    let mut writer = VaultStorage::new(VAULT_HOST.into(), writer_token, storage.namespace());
    assert_eq!(writer.get("anyone"), Ok(Value::U64(1)));
    assert_eq!(writer.get("root"), Err(Error::PermissionDenied));
    assert_eq!(writer.get("partial"), Ok(Value::U64(3)));
    assert_eq!(writer.get("full"), Ok(Value::U64(4)));

    let reader_token = storage.create_token(vec![&reader]).unwrap();
    let mut reader = VaultStorage::new(VAULT_HOST.into(), reader_token, storage.namespace());
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
}
