// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_global_constants::VALIDATOR_NETWORK_ADDRESS_KEYS;
use diem_infallible::RwLock;
use diem_secure_storage::{Error as StorageError, KVStorage, Storage};
use diem_types::{
    account_address::AccountAddress,
    network_address::{
        self,
        encrypted::{
            EncNetworkAddress, Key, KeyVersion, TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
        },
        NetworkAddress,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to deserialize address for account {0}: {1}")]
    AddressDeserialization(AccountAddress, String),
    #[error("Unable to decrypt address for account {0}: {1}")]
    DecryptionError(AccountAddress, String),
    #[error("Failed (de)serializing validator_network_address_keys")]
    BCSError(#[from] bcs::Error),
    #[error("NetworkAddress parse error {0}")]
    ParseError(#[from] network_address::ParseError),
    #[error("Failed reading validator_network_address_keys from storage")]
    StorageError(#[from] StorageError),
    #[error("The specified version does not exist in validator_network_address_keys: {0}")]
    VersionNotFound(KeyVersion),
}

pub struct Encryptor {
    storage: Storage,
    cached_keys: RwLock<Option<ValidatorKeys>>,
}

impl Encryptor {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            cached_keys: RwLock::new(None),
        }
    }

    /// This generates an empty encryptor for use in scenarios where encryption is not necessary.
    /// Any encryption operations (e.g., encrypt / decrypt) will return errors.
    pub fn empty() -> Self {
        let storage = Storage::InMemoryStorage(diem_secure_storage::InMemoryStorage::new());
        Encryptor::new(storage)
    }

    /// This generates an encryptor for use in testing scenarios. The encryptor is
    /// initialized with a test network encryption key.
    pub fn for_testing() -> Self {
        let mut encryptor = Self::empty();
        encryptor.initialize_for_testing().unwrap();
        encryptor
    }

    pub fn add_key(&mut self, version: KeyVersion, key: Key) -> Result<(), Error> {
        let mut keys = self.read()?;
        keys.keys.insert(version, StorageKey(key));
        self.write(&keys)
    }

    pub fn set_current_version(&mut self, version: KeyVersion) -> Result<(), Error> {
        let mut keys = self.read()?;
        if keys.keys.get(&version).is_some() {
            keys.current = version;
            self.write(&keys)
        } else {
            Err(Error::VersionNotFound(version))
        }
    }

    pub fn current_version(&self) -> Result<KeyVersion, Error> {
        self.read().map(|keys| keys.current)
    }

    pub fn encrypt(
        &self,
        network_addresses: &[NetworkAddress],
        account: AccountAddress,
        seq_num: u64,
    ) -> Result<Vec<u8>, Error> {
        let keys = self.read()?;
        let key = keys
            .keys
            .get(&keys.current)
            .ok_or(Error::VersionNotFound(keys.current))?;
        let mut enc_addrs = Vec::new();
        for (idx, addr) in network_addresses.iter().cloned().enumerate() {
            enc_addrs.push(addr.encrypt(&key.0, keys.current, &account, seq_num, idx as u32)?);
        }
        bcs::to_bytes(&enc_addrs).map_err(|e| e.into())
    }

    pub fn decrypt(
        &self,
        encrypted_network_addresses: &[u8],
        account: AccountAddress,
    ) -> Result<Vec<NetworkAddress>, Error> {
        let keys = self.read()?;
        let enc_addrs: Vec<EncNetworkAddress> = bcs::from_bytes(&encrypted_network_addresses)
            .map_err(|e| Error::AddressDeserialization(account, e.to_string()))?;
        let mut addrs = Vec::new();
        for (idx, enc_addr) in enc_addrs.iter().enumerate() {
            let key = keys
                .keys
                .get(&enc_addr.key_version())
                .ok_or_else(|| Error::VersionNotFound(enc_addr.key_version()))?;
            let addr = enc_addr
                .clone()
                .decrypt(&key.0, &account, idx as u32)
                .map_err(|e| Error::DecryptionError(account, e.to_string()))?;
            addrs.push(addr);
        }
        Ok(addrs)
    }

    pub fn initialize(&mut self) -> Result<(), Error> {
        self.write(&ValidatorKeys::default())
    }

    pub fn initialize_for_testing(&mut self) -> Result<(), Error> {
        self.initialize()?;
        self.add_key(
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            TEST_SHARED_VAL_NETADDR_KEY,
        )?;
        self.set_current_version(TEST_SHARED_VAL_NETADDR_KEY_VERSION)
    }

    fn read(&self) -> Result<ValidatorKeys, Error> {
        let result = self
            .storage
            .get::<ValidatorKeys>(VALIDATOR_NETWORK_ADDRESS_KEYS)
            .map(|v| v.value)
            .map_err(|e| e.into());

        match &result {
            Ok(keys) => {
                *self.cached_keys.write() = Some(keys.clone());
            }
            Err(err) => diem_logger::error!(
                "Unable to read {} from storage: {}",
                VALIDATOR_NETWORK_ADDRESS_KEYS,
                err
            ),
        }

        let keys = self.cached_keys.read();
        keys.as_ref().map_or(result, |v| Ok(v.clone()))
    }

    fn write(&mut self, keys: &ValidatorKeys) -> Result<(), Error> {
        self.storage
            .set(VALIDATOR_NETWORK_ADDRESS_KEYS, keys)
            .map_err(|e| e.into())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StorageKey(
    #[serde(
        serialize_with = "diem_secure_storage::to_base64",
        deserialize_with = "from_base64"
    )]
    Key,
);

pub fn to_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&base64::encode(bytes))
}

pub fn from_base64<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    base64::decode(s)
        .map_err(serde::de::Error::custom)
        .and_then(|v| {
            std::convert::TryInto::try_into(v.as_slice()).map_err(serde::de::Error::custom)
        })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorKeys {
    keys: HashMap<KeyVersion, StorageKey>,
    current: KeyVersion,
}

impl Default for ValidatorKeys {
    fn default() -> Self {
        ValidatorKeys {
            current: 0,
            keys: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diem_secure_storage::{InMemoryStorage, Namespaced};
    use rand::{rngs::OsRng, Rng, RngCore, SeedableRng};

    #[test]
    fn e2e() {
        let storage = Storage::InMemoryStorage(InMemoryStorage::new());
        let mut encryptor = Encryptor::new(storage);
        encryptor.initialize().unwrap();

        let mut rng = rand::rngs::StdRng::from_seed(OsRng.gen());
        let mut key = [0; network_address::encrypted::KEY_LEN];
        rng.fill_bytes(&mut key);
        encryptor.add_key(0, key).unwrap();
        encryptor.set_current_version(0).unwrap();
        rng.fill_bytes(&mut key);
        encryptor.add_key(1, key).unwrap();
        encryptor.set_current_version(1).unwrap();
        rng.fill_bytes(&mut key);
        encryptor.add_key(4, key).unwrap();
        encryptor.set_current_version(4).unwrap();

        encryptor.set_current_version(5).unwrap_err();

        let addr = std::str::FromStr::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
        let addrs = vec![addr];
        let account = AccountAddress::random();

        let enc_addrs = encryptor.encrypt(&addrs, account, 5).unwrap();
        let dec_addrs = encryptor.decrypt(&enc_addrs, account).unwrap();
        assert_eq!(addrs, dec_addrs);

        let another_account = AccountAddress::random();
        encryptor.decrypt(&enc_addrs, another_account).unwrap_err();
    }

    #[test]
    fn cache_test() {
        // Prepare some initial data and verify e2e
        let mut encryptor = Encryptor::for_testing();
        let addr = std::str::FromStr::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
        let addrs = vec![addr];
        let account = AccountAddress::random();

        let enc_addrs = encryptor.encrypt(&addrs, account, 0).unwrap();
        let dec_addrs = encryptor.decrypt(&enc_addrs, account).unwrap();
        assert_eq!(addrs, dec_addrs);

        // Reset storage and we should use cache
        encryptor.storage = Storage::from(InMemoryStorage::new());
        let enc_addrs = encryptor.encrypt(&addrs, account, 1).unwrap();
        let dec_addrs = encryptor.decrypt(&enc_addrs, account).unwrap();
        assert_eq!(addrs, dec_addrs);

        // Reset cache and we should get an err
        *encryptor.cached_keys.write() = None;
        encryptor.encrypt(&addrs, account, 1).unwrap_err();
        encryptor.decrypt(&enc_addrs, account).unwrap_err();
    }

    // The only purpose of this test is to generate a baseline for vault
    #[ignore]
    #[test]
    fn initializer() {
        let storage = Storage::from(Namespaced::new(
            "network_address_encryption_keys",
            Box::new(Storage::VaultStorage(
                diem_secure_storage::VaultStorage::new(
                    "http://127.0.0.1:8200".to_string(),
                    "root_token".to_string(),
                    None,
                    None,
                    true,
                    None,
                    None,
                ),
            )),
        ));
        let mut encryptor = Encryptor::new(storage);
        encryptor.initialize().unwrap();
        let mut rng = rand::rngs::StdRng::from_seed(OsRng.gen());
        let mut key = [0; network_address::encrypted::KEY_LEN];
        rng.fill_bytes(&mut key);
        encryptor.add_key(0, key).unwrap();
        encryptor.set_current_version(0).unwrap();
    }
}
