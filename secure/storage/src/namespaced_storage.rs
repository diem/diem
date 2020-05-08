// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage, Value};

/// This provides a light wrapper around KV storages to support a namespace. That namespace is
/// effectively prefixing all keys with then namespace value and "/" so a namespace of foo and a
/// key of bar becomes "foo/bar". Without a namespace, the key would just be "bar". This matches
/// how this library implements namespaces for Vault.
pub struct NamespacedStorage<T> {
    namespace: String,
    inner: T,
}

impl<T: KVStorage> KVStorage for NamespacedStorage<T> {
    fn available(&self) -> bool {
        self.inner.available()
    }

    fn get(&self, key: &str) -> Result<GetResponse, Error> {
        self.inner.get(&self.ns_name(key))
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        self.inner.set(&self.ns_name(key), value)
    }

    /// Note: This is not a namespace function
    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.inner.reset_and_clear()
    }
}

impl<T> NamespacedStorage<T> {
    pub fn new(storage: T, namespace: String) -> Self {
        NamespacedStorage {
            namespace,
            inner: storage,
        }
    }

    fn ns_name(&self, key: &str) -> String {
        format!("{}/{}", self.namespace, key)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

impl<T: KVStorage> CryptoKVStorage for NamespacedStorage<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::OnDiskStorage;
    use libra_temppath::TempPath;

    #[test]
    fn test_different_namespaces() {
        let ns0 = "ns0";
        let ns1 = "ns1";
        let key = "key";

        let path_buf = TempPath::new().path().to_path_buf();

        let mut default = OnDiskStorage::new(path_buf.clone());

        let storage = OnDiskStorage::new(path_buf.clone());
        let mut nss0 = NamespacedStorage::new(storage, ns0.into());

        let storage = OnDiskStorage::new(path_buf);
        let mut nss1 = NamespacedStorage::new(storage, ns1.into());

        default.set(key, Value::U64(0)).unwrap();
        nss0.set(key, Value::U64(1)).unwrap();
        nss1.set(key, Value::U64(2)).unwrap();

        assert_eq!(default.get(key).unwrap().value, Value::U64(0));
        assert_eq!(nss0.get(key).unwrap().value, Value::U64(1));
        assert_eq!(nss1.get(key).unwrap().value, Value::U64(2));
    }

    #[test]
    fn test_shared_namespace() {
        let ns = "ns";
        let key = "key";

        let path_buf = TempPath::new().path().to_path_buf();

        let default = OnDiskStorage::new(path_buf.clone());

        let storage = OnDiskStorage::new(path_buf.clone());
        let mut nss = NamespacedStorage::new(storage, ns.into());

        let storage = OnDiskStorage::new(path_buf);
        let another_nss = NamespacedStorage::new(storage, ns.into());

        nss.set(key, Value::U64(1)).unwrap();
        default.get(key).unwrap_err();
        assert_eq!(nss.get(key).unwrap().value, Value::U64(1));
        assert_eq!(another_nss.get(key).unwrap().value, Value::U64(1));
    }
}
