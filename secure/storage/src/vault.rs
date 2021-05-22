// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, GetResponse, KVStorage, PublicKeyResponse};
use chrono::DateTime;
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
};
use diem_infallible::RwLock;
use diem_time_service::{TimeService, TimeServiceTrait};
use diem_vault_client::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(any(test, feature = "testing"))]
use diem_vault_client::ReadResponse;

/// VaultStorage utilizes Vault for maintaining encrypted, authenticated data for Diem. This
/// version currently matches the behavior of OnDiskStorage and InMemoryStorage. In the future,
/// Vault will be able to create keys, sign messages, and handle permissions across different
/// services. The specific vault service leveraged herein is called KV (Key Value) Secrets Engine -
/// Version 2 (https://www.vaultproject.io/api/secret/kv/kv-v2.html). So while Diem Secure Storage
/// calls pointers to data keys, Vault has actually a secret that contains multiple key value
/// pairs.
pub struct VaultStorage {
    client: Client,
    time_service: TimeService,
    namespace: Option<String>,
    renew_ttl_secs: Option<u32>,
    next_renewal: AtomicU64,
    use_cas: bool,
    secret_versions: RwLock<HashMap<String, u32>>,
}

impl VaultStorage {
    pub fn new(
        host: String,
        token: String,
        namespace: Option<String>,
        certificate: Option<String>,
        renew_ttl_secs: Option<u32>,
        use_cas: bool,
        connection_timeout_ms: Option<u64>,
        response_timeout_ms: Option<u64>,
    ) -> Self {
        Self {
            client: Client::new(
                host,
                token,
                certificate,
                connection_timeout_ms,
                response_timeout_ms,
            ),
            time_service: TimeService::real(),
            namespace,
            renew_ttl_secs,
            next_renewal: AtomicU64::new(0),
            use_cas,
            secret_versions: RwLock::new(HashMap::new()),
        }
    }

    // Made into an accessor so we can get auto-renewal
    fn client(&self) -> &Client {
        if self.renew_ttl_secs.is_some() {
            let now = self.time_service.now_secs();
            let next_renewal = self.next_renewal.load(Ordering::Relaxed);
            if now >= next_renewal {
                let result = self.client.renew_token_self(self.renew_ttl_secs);
                if let Ok(ttl) = result {
                    let next_renewal = now + (ttl as u64) / 2;
                    self.next_renewal.store(next_renewal, Ordering::Relaxed);
                } else if let Err(e) = result {
                    diem_logger::error!("Unable to renew lease: {}", e.to_string());
                }
            }
        }
        &self.client
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_kv(&self, path: &str) -> Result<(), Error> {
        let secrets = self.client().list_secrets(path)?;
        for secret in secrets {
            if secret.ends_with('/') {
                self.reset_kv(&secret)?;
            } else {
                self.client()
                    .delete_secret(&format!("{}{}", path, secret))?;
            }
        }
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_crypto(&self) -> Result<(), Error> {
        let keys = match self.client().list_keys() {
            Ok(keys) => keys,
            // No keys were found, so there's no need to reset.
            Err(diem_vault_client::Error::NotFound(_, _)) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        for key in keys {
            self.client().delete_key(&key)?;
        }
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn revoke_token_self(&self) -> Result<(), Error> {
        Ok(self.client.revoke_token_self()?)
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn get_all_key_versions(
        &self,
        name: &str,
    ) -> Result<Vec<ReadResponse<Ed25519PublicKey>>, Error> {
        Ok(self.client().read_ed25519_key(name)?)
    }

    fn key_version(&self, name: &str, version: &Ed25519PublicKey) -> Result<u32, Error> {
        let pubkeys = self.client().read_ed25519_key(name)?;
        let pubkey = pubkeys.iter().find(|pubkey| version == &pubkey.value);
        Ok(pubkey
            .ok_or_else(|| Error::KeyVersionNotFound(name.into(), version.to_string()))?
            .version)
    }

    fn crypto_name(&self, name: &str) -> String {
        self.name(name).replace('/', "__")
    }

    fn secret_name(&self, name: &str) -> String {
        self.name(name)
    }

    fn name(&self, name: &str) -> String {
        if let Some(namespace) = &self.namespace {
            format!("{}/{}", namespace, name)
        } else {
            name.into()
        }
    }
}

impl KVStorage for VaultStorage {
    fn available(&self) -> Result<(), Error> {
        if !self.client().unsealed()? {
            Err(Error::InternalError("Vault is not unsealed".into()))
        } else {
            Ok(())
        }
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<T>, Error> {
        let secret = self.secret_name(key);
        let resp = self.client().read_secret(&secret, key)?;
        let last_update = DateTime::parse_from_rfc3339(&resp.creation_time)?.timestamp() as u64;
        let value: T = serde_json::from_value(resp.value)?;
        self.secret_versions
            .write()
            .insert(key.to_string(), resp.version);
        Ok(GetResponse { last_update, value })
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        let secret = self.secret_name(key);
        let version = if self.use_cas {
            self.secret_versions.read().get(key).copied()
        } else {
            None
        };
        let new_version =
            self.client()
                .write_secret(&secret, key, &serde_json::to_value(&value)?, version)?;
        self.secret_versions
            .write()
            .insert(key.to_string(), new_version);
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.secret_versions.write().clear();
        self.reset_kv("")?;
        self.reset_crypto()?;
        Ok(())
    }
}

impl CryptoStorage for VaultStorage {
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        let ns_name = self.crypto_name(name);
        match self.get_public_key(name) {
            Ok(_) => return Err(Error::KeyAlreadyExists(ns_name)),
            Err(Error::KeyNotSet(_)) => (/* Expected this for new keys! */),
            Err(e) => return Err(e),
        }

        self.client().create_ed25519_key(&ns_name, true)?;
        self.get_public_key(name).map(|v| v.public_key)
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        let name = self.crypto_name(name);
        Ok(self.client().export_ed25519_key(&name, None)?)
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        let name = self.crypto_name(name);
        let vers = self.key_version(&name, &version)?;
        Ok(self.client().export_ed25519_key(&name, Some(vers))?)
    }

    fn import_private_key(&mut self, name: &str, key: Ed25519PrivateKey) -> Result<(), Error> {
        let ns_name = self.crypto_name(name);
        match self.get_public_key(name) {
            Ok(_) => return Err(Error::KeyAlreadyExists(ns_name)),
            Err(Error::KeyNotSet(_)) => (/* Expected this for new keys! */),
            Err(e) => return Err(e),
        }

        self.client()
            .import_ed25519_key(&ns_name, &key)
            .map_err(|e| e.into())
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        let name = self.crypto_name(name);
        let resp = self.client().read_ed25519_key(&name)?;
        let mut last_key = resp.first().ok_or(Error::KeyNotSet(name))?;
        for key in &resp {
            last_key = if last_key.version > key.version {
                last_key
            } else {
                key
            }
        }

        Ok(PublicKeyResponse {
            last_update: DateTime::parse_from_rfc3339(&last_key.creation_time)?.timestamp() as u64,
            public_key: last_key.value.clone(),
        })
    }

    fn get_public_key_previous_version(&self, name: &str) -> Result<Ed25519PublicKey, Error> {
        let name = self.crypto_name(name);
        let pubkeys = self.client().read_ed25519_key(&name)?;
        let highest_version = pubkeys.iter().map(|pubkey| pubkey.version).max();
        match highest_version {
            Some(version) => {
                let pubkey = pubkeys.iter().find(|pubkey| pubkey.version == version - 1);
                Ok(pubkey
                    .ok_or_else(|| Error::KeyVersionNotFound(name, "previous version".into()))?
                    .value
                    .clone())
            }
            None => Err(Error::KeyVersionNotFound(name, "previous version".into())),
        }
    }

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        let ns_name = self.crypto_name(name);
        self.client().rotate_key(&ns_name)?;
        Ok(self.client().trim_key_versions(&ns_name)?)
    }

    fn sign<T: CryptoHash + Serialize>(
        &self,
        name: &str,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        let name = self.crypto_name(name);
        let mut bytes = <T::Hasher as diem_crypto::hash::CryptoHasher>::seed().to_vec();
        bcs::serialize_into(&mut bytes, &message).map_err(|e| {
            Error::InternalError(format!(
                "Serialization of signable material should not fail, yet returned Error:{}",
                e
            ))
        })?;
        Ok(self.client().sign_ed25519(&name, &bytes, None)?)
    }

    fn sign_using_version<T: CryptoHash + Serialize>(
        &self,
        name: &str,
        version: Ed25519PublicKey,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        let name = self.crypto_name(name);
        let vers = self.key_version(&name, &version)?;
        let mut bytes = <T::Hasher as diem_crypto::hash::CryptoHasher>::seed().to_vec();
        bcs::serialize_into(&mut bytes, &message).map_err(|e| {
            Error::InternalError(format!(
                "Serialization of signable material should not fail, yet returned Error:{}",
                e
            ))
        })?;
        Ok(self.client().sign_ed25519(&name, &bytes, Some(vers))?)
    }
}
