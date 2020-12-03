// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

/// This provides a light wrapper around KV storages to support a namespace. That namespace is
/// effectively prefixing all keys with then namespace value and "/" so a namespace of foo and a
/// key of bar becomes "foo/bar". Without a namespace, the key would just be "bar". This matches
/// how this library implements namespaces for Vault.
pub struct NamespacedStorage {
    namespace: String,
    inner: Box<Storage>,
}

impl KVStorage for NamespacedStorage {
    fn available(&self) -> Result<(), Error> {
        self.inner.available()
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<T>, Error> {
        self.inner.get(&self.ns_name(key))
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        self.inner.set(&self.ns_name(key), value)
    }

    /// Note: This is not a namespace function
    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.inner.reset_and_clear()
    }
}

impl NamespacedStorage {
    pub fn new(storage: Storage, namespace: String) -> Self {
        NamespacedStorage {
            namespace,
            inner: Box::new(storage),
        }
    }

    fn ns_name(&self, key: &str) -> String {
        format!("{}/{}", self.namespace, key)
    }
}

impl CryptoKVStorage for NamespacedStorage {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::OnDiskStorage;
    use diem_temppath::TempPath;

    #[test]
    fn test_different_namespaces() {
        let ns0 = "ns0";
        let ns1 = "ns1";
        let key = "key";

        let path_buf = TempPath::new().path().to_path_buf();

        let mut default = OnDiskStorage::new(path_buf.clone());

        let storage = Storage::OnDiskStorage(OnDiskStorage::new(path_buf.clone()));
        let mut nss0 = NamespacedStorage::new(storage, ns0.into());

        let storage = Storage::OnDiskStorage(OnDiskStorage::new(path_buf));
        let mut nss1 = NamespacedStorage::new(storage, ns1.into());

        default.set(key, 0).unwrap();
        nss0.set(key, 1).unwrap();
        nss1.set(key, 2).unwrap();

        assert_eq!(default.get::<u64>(key).unwrap().value, 0);
        assert_eq!(nss0.get::<u64>(key).unwrap().value, 1);
        assert_eq!(nss1.get::<u64>(key).unwrap().value, 2);
    }

    #[test]
    fn test_shared_namespace() {
        let ns = "ns";
        let key = "key";

        let path_buf = TempPath::new().path().to_path_buf();

        let default = Storage::OnDiskStorage(OnDiskStorage::new(path_buf.clone()));

        let storage = Storage::OnDiskStorage(OnDiskStorage::new(path_buf.clone()));
        let mut nss = NamespacedStorage::new(storage, ns.into());

        let storage = Storage::OnDiskStorage(OnDiskStorage::new(path_buf));
        let another_nss = NamespacedStorage::new(storage, ns.into());

        nss.set(key, 1).unwrap();
        default.get::<u64>(key).unwrap_err();
        assert_eq!(nss.get::<u64>(key).unwrap().value, 1);
        assert_eq!(another_nss.get::<u64>(key).unwrap().value, 1);
    }
}
