// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Capability, CryptoStorage, Error, GetResponse, Identity, KVStorage, Policy, PublicKeyResponse,
    Storage, Value,
};
use chrono::DateTime;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue,
};
use libra_vault_client::{self as vault, Client};

const LIBRA_DEFAULT: &str = "libra_default";

/// VaultStorage utilizes Vault for maintaining encrypted, authenticated data for Libra. This
/// version currently matches the behavior of OnDiskStorage and InMemoryStorage. In the future,
/// Vault will be able to create keys, sign messages, and handle permissions across different
/// services. The specific vault service leveraged herein is called KV (Key Value) Secrets Engine -
/// Version 2 (https://www.vaultproject.io/api/secret/kv/kv-v2.html). So while Libra Secure Storage
/// calls pointers to data keys, Vault has actually a secret that contains multiple key value
/// pairs.
pub struct VaultStorage {
    client: Client,
    namespace: Option<String>,
}

impl VaultStorage {
    pub fn new(host: String, token: String, namespace: Option<String>) -> Self {
        Self {
            client: Client::new(host, token),
            namespace,
        }
    }

    pub fn namespace(&self) -> Option<String> {
        self.namespace.clone()
    }

    /// Erase all secrets and keys from the vault storage. If a namespace was specified on vault
    /// storage creation, only the secrets associated with that namespace are removed. Use with
    /// caution.
    pub fn reset(&self) -> Result<(), Error> {
        if let Some(namespace) = &self.namespace {
            let key_path = format!("{}/", namespace);
            let crypto_path = format!("{}__", namespace);
            self.reset_kv(&key_path)?;
            self.reset_crypto(&crypto_path)?;
            self.reset_policies(&key_path)?;
            self.reset_policies(&crypto_path)
        } else {
            self.reset_kv("")?;
            self.reset_crypto("")?;
            self.reset_policies("")
        }
    }

    fn reset_kv(&self, path: &str) -> Result<(), Error> {
        let secrets = self.client.list_secrets(path)?;
        for secret in secrets {
            if secret.ends_with('/') {
                self.reset_kv(&secret)?;
            } else {
                self.client.delete_secret(&format!("{}{}", path, secret))?;
            }
        }
        Ok(())
    }

    fn reset_crypto(&self, prefix: &str) -> Result<(), Error> {
        let keys = match self.client.list_keys() {
            Ok(keys) => keys,
            // No keys were found, so there's no need to reset.
            Err(libra_vault_client::Error::NotFound(_, _)) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        for key in keys {
            if (!key.contains("__") && prefix.is_empty()) || key.starts_with(prefix) {
                self.client.delete_key(&key)?;
            }
        }
        Ok(())
    }

    fn reset_policies(&self, prefix: &str) -> Result<(), Error> {
        let policies = match self.client.list_policies() {
            Ok(policies) => policies,
            Err(libra_vault_client::Error::NotFound(_, _)) => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        for policy in policies {
            // Never touch the default or root policy
            if policy == "default" || policy == "root" {
                continue;
            }

            let ns_policy = policy.contains("__") || policy.contains('/');
            if (!ns_policy && prefix.is_empty()) || policy.starts_with(prefix) {
                self.client.delete_policy(&policy)?;
            }
        }
        Ok(())
    }

    /// Creates a token but uses the namespace for policies
    pub fn create_token(&self, mut policies: Vec<&str>) -> Result<String, Error> {
        policies.push(LIBRA_DEFAULT);
        let result = if let Some(ns) = &self.namespace {
            let policies: Vec<_> = policies.iter().map(|p| format!("{}/{}", ns, p)).collect();
            self.client
                .create_token(policies.iter().map(|p| &**p).collect())?
        } else {
            self.client.create_token(policies)?
        };
        Ok(result)
    }

    /// Retrieves a key from a given secret. Libra Secure Storage inserts each key into its own
    /// distinct secret store and thus the secret and key have the same identifier.
    fn get_secret(&self, key: &str) -> Result<GetResponse, Error> {
        let secret = self.secret_name(key);
        let resp = self.client.read_secret(&secret, key)?;
        let last_update = DateTime::parse_from_rfc3339(&resp.creation_time)?.timestamp() as u64;
        let value = Value::from_base64(&resp.value)?;
        Ok(GetResponse { last_update, value })
    }

    /// Inserts a key, value pair into a secret that shares the name of the key.
    fn set_secret(&self, key: &str, value: Value) -> Result<(), Error> {
        let secret = self.secret_name(key);
        self.client
            .write_secret(&secret, key, &value.to_base64()?)?;
        Ok(())
    }

    /// Create a new policy in Vault, see the explanation for Policy for how the data is
    /// structured. Vault does not distingush a create and update. An update must first read the
    /// existing policy, amend the contents,  and then be applied via this API.
    fn set_policy(
        &self,
        policy_name: &str,
        engine: &VaultEngine,
        key: &str,
        capabilities: &[Capability],
    ) -> Result<(), Error> {
        let policy_name = self.name(policy_name, engine);

        let mut vault_policy = self.client.read_policy(&policy_name).unwrap_or_default();
        let mut core_capabilities = Vec::new();
        for capability in capabilities {
            match capability {
                Capability::Export => {
                    let export_capability = vec![vault::Capability::Read];
                    let export_policy = format!("transit/export/signing-key/{}", key);
                    vault_policy.add_policy(&export_policy, export_capability);
                }
                Capability::Read => core_capabilities.push(vault::Capability::Read),
                Capability::Rotate => {
                    let rotate_capability = vec![vault::Capability::Update];
                    let rotate_policy = format!("transit/keys/{}/rotate", key);
                    vault_policy.add_policy(&rotate_policy, rotate_capability);
                }
                Capability::Sign => {
                    let sign_capability = vec![vault::Capability::Update];
                    let sign_policy = format!("transit/sign/{}", key);
                    vault_policy.add_policy(&sign_policy, sign_capability);
                }
                Capability::Write => core_capabilities.push(vault::Capability::Update),
            }
        }

        let path = format!("{}/{}", engine.to_policy_path(), self.name(key, engine));
        vault_policy.add_policy(&path, core_capabilities);
        self.client.set_policy(&policy_name, &vault_policy)?;
        Ok(())
    }

    /// Public convenience function to return a new Vault based Storage.
    pub fn new_storage(host: String, token: String, namespace: Option<String>) -> Box<dyn Storage> {
        Box::new(VaultStorage::new(host, token, namespace))
    }

    fn key_version(&self, name: &str, version: &Ed25519PublicKey) -> Result<u32, Error> {
        let pubkeys = self.client.read_ed25519_key(name)?;
        let pubkey = pubkeys.iter().find(|pubkey| version == &pubkey.value);
        Ok(pubkey
            .ok_or_else(|| Error::KeyVersionNotFound(name.into()))?
            .version)
    }

    fn set_policies(&self, name: &str, engine: &VaultEngine, policy: &Policy) -> Result<(), Error> {
        for perm in &policy.permissions {
            match &perm.id {
                Identity::User(id) => self.set_policy(id, engine, name, &perm.capabilities)?,
                Identity::Anyone => {
                    self.set_policy(LIBRA_DEFAULT, engine, name, &perm.capabilities)?
                }
                Identity::NoOne => (),
            };
        }
        Ok(())
    }

    fn crypto_name(&self, name: &str) -> String {
        self.name(name, &VaultEngine::Transit)
    }

    fn secret_name(&self, name: &str) -> String {
        self.name(name, &VaultEngine::KVSecrets)
    }

    fn name(&self, name: &str, engine: &VaultEngine) -> String {
        if let Some(namespace) = &self.namespace {
            format!("{}{}{}", namespace, engine.ns_seperator(), name)
        } else {
            name.into()
        }
    }
}

impl KVStorage for VaultStorage {
    fn available(&self) -> bool {
        self.client.unsealed().unwrap_or(false) && self.client.transit_enabled().unwrap_or(false)
    }

    fn create(&mut self, key: &str, value: Value, policy: &Policy) -> Result<(), Error> {
        // Vault internally does not distinguish creation versus update except by permissions. So we
        // simulate that by first getting the key. If it doesn't exist, we're okay.
        match self.get_secret(&key) {
            Ok(_) => return Err(Error::KeyAlreadyExists(key.to_string())),
            Err(Error::KeyNotSet(_)) => (/* Expected this for new keys! */),
            Err(e) => return Err(e),
        }

        self.set_secret(&key, value)?;
        self.set_policies(key, &VaultEngine::KVSecrets, policy)?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<GetResponse, Error> {
        self.get_secret(&key)
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        // Vault internally does not distinguish create versus udpate except by permissions. So we
        // simulate that by first getting the key. If it exists, we can update it.
        self.get_secret(&key)?;
        self.set_secret(&key, value)?;
        Ok(())
    }

    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.reset()
    }
}

impl CryptoStorage for VaultStorage {
    fn create_key(&mut self, name: &str, policy: &Policy) -> Result<Ed25519PublicKey, Error> {
        let ns_name = self.crypto_name(name);
        match self.get_public_key(name) {
            Ok(_) => return Err(Error::KeyAlreadyExists(ns_name)),
            Err(Error::KeyNotSet(_)) => (/* Expected this for new keys! */),
            Err(e) => return Err(e),
        }

        self.client.create_ed25519_key(&ns_name, true)?;
        self.set_policies(&ns_name, &VaultEngine::Transit, policy)?;
        self.get_public_key(name).map(|v| v.public_key)
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        let name = self.crypto_name(name);
        Ok(self.client.export_ed25519_key(&name, None)?)
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        let name = self.crypto_name(name);
        let vers = self.key_version(&name, &version)?;
        Ok(self.client.export_ed25519_key(&name, Some(vers))?)
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        let name = self.crypto_name(name);
        let resp = self.client.read_ed25519_key(&name)?;
        let mut last_key = resp.first().ok_or_else(|| Error::KeyNotSet(name))?;
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

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        let ns_name = self.crypto_name(name);
        self.client.rotate_key(&ns_name)?;
        self.get_public_key(name).map(|v| v.public_key)
    }

    fn sign_message(&mut self, name: &str, message: &HashValue) -> Result<Ed25519Signature, Error> {
        let name = self.crypto_name(name);
        Ok(self.client.sign_ed25519(&name, message.as_ref(), None)?)
    }

    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        let name = self.crypto_name(name);
        let vers = self.key_version(&name, &version)?;
        Ok(self
            .client
            .sign_ed25519(&name, message.as_ref(), Some(vers))?)
    }
}

enum VaultEngine {
    KVSecrets,
    Transit,
}

impl VaultEngine {
    fn to_policy_path(&self) -> &str {
        match self {
            VaultEngine::KVSecrets => "secret/data",
            VaultEngine::Transit => "transit/keys",
        }
    }

    fn ns_seperator(&self) -> &str {
        match self {
            VaultEngine::KVSecrets => "/",
            VaultEngine::Transit => "__",
        }
    }
}
