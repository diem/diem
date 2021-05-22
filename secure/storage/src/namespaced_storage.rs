// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, GetResponse, KVStorage, PublicKeyResponse};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
};
use serde::{de::DeserializeOwned, Serialize};

/// This provides a light wrapper around KV storages to support a namespace. That namespace is
/// effectively prefixing all keys with then namespace value and "/" so a namespace of foo and a
/// key of bar becomes "foo/bar". Without a namespace, the key would just be "bar".
pub struct Namespaced<S> {
    namespace: String,
    inner: S,
}

impl<S> Namespaced<S> {
    pub fn new<N: Into<String>>(namespace: N, inner: S) -> Self {
        Self {
            namespace: namespace.into(),
            inner,
        }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    fn namespaced(&self, name: &str) -> String {
        format!("{}/{}", self.namespace, name)
    }
}

impl<S: KVStorage> KVStorage for Namespaced<S> {
    fn available(&self) -> Result<(), Error> {
        self.inner.available()
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<T>, Error> {
        self.inner.get(&self.namespaced(key))
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        self.inner.set(&self.namespaced(key), value)
    }

    /// Note: This is not a namespace function
    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.inner.reset_and_clear()
    }
}

impl<S: CryptoStorage> CryptoStorage for Namespaced<S> {
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.inner.create_key(&self.namespaced(name))
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        self.inner.export_private_key(&self.namespaced(name))
    }

    fn import_private_key(&mut self, name: &str, key: Ed25519PrivateKey) -> Result<(), Error> {
        self.inner.import_private_key(&self.namespaced(name), key)
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        self.inner
            .export_private_key_for_version(&self.namespaced(name), version)
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        self.inner.get_public_key(&self.namespaced(name))
    }

    fn get_public_key_previous_version(&self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.inner
            .get_public_key_previous_version(&self.namespaced(name))
    }

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.inner.rotate_key(&self.namespaced(name))
    }

    fn sign<T: CryptoHash + Serialize>(
        &self,
        name: &str,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        self.inner.sign(&self.namespaced(name), message)
    }

    fn sign_using_version<T: CryptoHash + Serialize>(
        &self,
        name: &str,
        version: Ed25519PublicKey,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        self.inner
            .sign_using_version(&self.namespaced(name), version, message)
    }
}

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
        let mut nss0 = Namespaced::new(ns0, OnDiskStorage::new(path_buf.clone()));
        let mut nss1 = Namespaced::new(ns1, OnDiskStorage::new(path_buf));

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

        let default = OnDiskStorage::new(path_buf.clone());
        let mut nss = Namespaced::new(ns, OnDiskStorage::new(path_buf.clone()));
        let another_nss = Namespaced::new(ns, OnDiskStorage::new(path_buf));

        nss.set(key, 1).unwrap();
        default.get::<u64>(key).unwrap_err();
        assert_eq!(nss.get::<u64>(key).unwrap().value, 1);
        assert_eq!(another_nss.get::<u64>(key).unwrap().value, 1);
    }
}
