// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_global_constants::VALIDATOR_NETWORK_ADDRESS_KEYS;
use libra_network_address::{
    encrypted::{EncNetworkAddress, Key, KeyVersion},
    NetworkAddress,
};
use libra_secure_storage::{Error as StorageError, KVStorage, Storage};
use move_core_types::account_address::AccountAddress;
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
    LCSError(#[from] lcs::Error),
    #[error("Failed reading validator_network_address_keys from storage")]
    StorageError(#[from] StorageError),
    #[error("The specified version does not exist in validator_network_address_keys: {0}")]
    VersionNotFound(KeyVersion),
}

pub struct Encryptor {
    storage: Storage,
}

impl Encryptor {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
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
        network_address: &NetworkAddress,
        account: AccountAddress,
        seq_num: u64,
    ) -> Result<Vec<u8>, Error> {
        let keys = self.read()?;
        let key = keys
            .keys
            .get(&keys.current)
            .ok_or_else(|| Error::VersionNotFound(keys.current))?;
        let raw_addr = std::convert::TryFrom::<&NetworkAddress>::try_from(network_address)?;
        // TODO(davidiw) soon we'll take in a set, for now we assume there's only one addr
        let encrypted_key =
            EncNetworkAddress::encrypt(raw_addr, &key.0, keys.current, &account, seq_num, 0);
        lcs::to_bytes(&encrypted_key).map_err(|e| e.into())
    }

    pub fn decrypt(
        &self,
        encrypted_network_address: &[u8],
        account: AccountAddress,
    ) -> Result<NetworkAddress, Error> {
        let keys = self.read()?;
        let enc_addr: EncNetworkAddress = lcs::from_bytes(&encrypted_network_address)
            .map_err(|e| Error::AddressDeserialization(account, e.to_string()))?;
        let key = keys
            .keys
            .get(&keys.current)
            .ok_or_else(|| Error::VersionNotFound(enc_addr.key_version()))?;
        // TODO(davidiw) soon we'll take in a set, for now we assume there's only one addr
        let raw_addr = enc_addr
            .decrypt(&key.0, &account, 0)
            .map_err(|e| Error::DecryptionError(account, e.to_string()))?;
        std::convert::TryFrom::<_>::try_from(&raw_addr).map_err(|e: lcs::Error| e.into())
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
        self.storage
            .get::<ValidatorKeys>(VALIDATOR_NETWORK_ADDRESS_KEYS)
            .map(|v| v.value)
            .map_err(|e| e.into())
    }

    fn write(&mut self, keys: &ValidatorKeys) -> Result<(), Error> {
        self.storage
            .set(VALIDATOR_NETWORK_ADDRESS_KEYS, keys)
            .map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StorageKey(
    #[serde(
        serialize_with = "libra_secure_storage::to_base64",
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
        .and_then(|v| std::convert::TryInto::try_into(v.as_slice()).map_err(serde::de::Error::custom))
}

#[derive(Debug, Deserialize, Serialize)]
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
    use libra_secure_storage::InMemoryStorage;
    use rand::{rngs::OsRng, Rng, RngCore, SeedableRng};

    #[test]
    fn e2e() {
        let storage = Storage::InMemoryStorage(InMemoryStorage::new());
        let mut encryptor = Encryptor::new(storage);
        encryptor.initialize().unwrap();

        let mut rng = rand::rngs::StdRng::from_seed(OsRng.gen());
        let mut key = [0; libra_network_address::encrypted::KEY_LEN];
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
        let account = AccountAddress::random();

        let enc_addr = encryptor.encrypt(&addr, account, 5).unwrap();
        let dec_addr = encryptor.decrypt(&enc_addr, account).unwrap();
        assert_eq!(addr, dec_addr);

        let another_account = AccountAddress::random();
        encryptor.decrypt(&enc_addr, another_account).unwrap_err();
    }

    // The only purpose of this test is to generate a baseline for vault
    #[ignore]
    #[test]
    fn initializer() {
        let storage = Storage::VaultStorage(libra_secure_storage::VaultStorage::new(
            "http://127.0.0.1:8200".to_string(),
            "root_token".to_string(),
            Some("network_address_encryption_keys".to_string()),
            None,
            None,
        ));
        let mut encryptor = Encryptor::new(storage);
        encryptor.initialize().unwrap();
        let mut rng = rand::rngs::StdRng::from_seed(OsRng.gen());
        let mut key = [0; libra_network_address::encrypted::KEY_LEN];
        rng.fill_bytes(&mut key);
        encryptor.add_key(0, key).unwrap();
        encryptor.set_current_version(0).unwrap();
    }
}
