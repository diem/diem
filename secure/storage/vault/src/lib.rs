// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    sync::Arc,
};
use thiserror::Error;

/// Request timeout for vault operations
const TIMEOUT: u64 = 10_000;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Http error: {1}")]
    HttpError(u16, String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Missing field {0}")]
    MissingField(String),
    #[error("404: Not Found: {0}/{1}")]
    NotFound(String, String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<base64::DecodeError> for Error {
    fn from(error: base64::DecodeError) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<libra_crypto::traits::CryptoMaterialError> for Error {
    fn from(error: libra_crypto::traits::CryptoMaterialError) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<ureq::Response> for Error {
    fn from(resp: ureq::Response) -> Self {
        Error::HttpError(resp.status(), resp.status_line().into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

/// Client provides a client around the restful interface to a Vault servce. Learn more
/// here: https://www.vaultproject.io/api-docs/
///
/// A brief overview of Vault:
///
/// * Vault stores data in various paths, in the case of a WebAPI, different URLs. So, for example,
/// both a secret and a policy are hosted at distinct paths. Policies are then used to define which
/// actors can access those paths and with what actions.
/// * Vault uses a KV store separated into various containers or secrets. In the concept of a file
/// system, a secret might represent a folder, where keys would be files, and the contents the
/// values. Policies are only applied at the folder level.
/// * Data is accessed in Vault via tokens. Policies can only be granted during creation of a
/// token, but policies can be amended afterward. So you cannot add new policies to a token, but
/// you can increase the tokens abilities by modifying the underlying policies.
pub struct Client {
    host: String,
    token: String,
    tls_config: Option<Arc<rustls::ClientConfig>>,
}

impl Client {
    pub fn new(host: String, token: String, ca_certificate: Option<String>) -> Self {
        let tls_config = if let Some(certificate) = ca_certificate {
            let mut tls_config = rustls::ClientConfig::new();
            // First try the certificate as a DER encoded cert, then as a PEM, and then panic.
            let cert = rustls::Certificate(certificate.as_bytes().to_vec());
            if tls_config.root_store.add(&cert).is_err() {
                let certs = rustls::internal::pemfile::certs(&mut certificate.as_bytes()).unwrap();
                tls_config.root_store.add(&certs[0]).unwrap();
            }
            Some(Arc::new(tls_config))
        } else {
            None
        };
        Self {
            host,
            token,
            tls_config,
        }
    }

    pub fn delete_policy(&self, policy_name: &str) -> Result<(), Error> {
        let request = ureq::delete(&format!("{}/v1/sys/policy/{}", self.host, policy_name));
        let resp = self.upgrade_request(request).call();
        if resp.ok() {
            Ok(())
        } else {
            Err(resp.into())
        }
    }

    pub fn list_policies(&self) -> Result<Vec<String>, Error> {
        let request = ureq::get(&format!("{}/v1/sys/policy", self.host));
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => {
                let policies: ListPoliciesResponse = serde_json::from_str(&resp.into_string()?)?;
                Ok(policies.policies)
            }
            // There are no policies.
            404 => Ok(vec![]),
            _ => Err(resp.into()),
        }
    }

    /// Retrieves the policy at the given policy name.
    pub fn read_policy(&self, policy_name: &str) -> Result<Policy, Error> {
        let request = ureq::get(&format!("{}/v1/sys/policy/{}", self.host, policy_name));
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => Ok(Policy::try_from(resp.into_json()?)?),
            _ => Err(resp.into()),
        }
    }

    /// Create a new policy in Vault, see the explanation for Policy for how the data is
    /// structured. Vault does not distingush a create and update. An update must first read the
    /// existing policy, amend the contents,  and then be applied via this API.
    pub fn set_policy(&self, policy_name: &str, policy: &Policy) -> Result<(), Error> {
        let request = ureq::post(&format!("{}/v1/sys/policy/{}", self.host, policy_name));
        let resp = self.upgrade_request(request).send_json(policy.try_into()?);
        if resp.ok() {
            Ok(())
        } else {
            Err(resp.into())
        }
    }

    /// Creates a new token or identity for accessing Vault. The token will have access to anything
    /// under the default policy and any prescribed policies.
    pub fn create_token(&self, policies: Vec<&str>) -> Result<String, Error> {
        let request = ureq::post(&format!("{}/v1/auth/token/create", self.host));
        let resp = self
            .upgrade_request(request)
            .send_json(json!({ "policies": policies }));
        if resp.ok() {
            let resp: CreateTokenResponse = serde_json::from_str(&resp.into_string()?)?;
            Ok(resp.auth.client_token)
        } else {
            Err(resp.into())
        }
    }

    /// List all stored secrets
    pub fn list_secrets(&self, secret: &str) -> Result<Vec<String>, Error> {
        let request = ureq::request(
            "LIST",
            &format!("{}/v1/secret/metadata/{}", self.host, secret),
        );
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => {
                let resp: ReadSecretListResponse = serde_json::from_str(&resp.into_string()?)?;
                Ok(resp.data.keys)
            }
            // There are no secrets.
            404 => Ok(vec![]),
            _ => Err(resp.into()),
        }
    }

    /// Delete a specific secret store
    pub fn delete_secret(&self, secret: &str) -> Result<(), Error> {
        let request = ureq::delete(&format!("{}/v1/secret/metadata/{}", self.host, secret));
        let resp = self.upgrade_request(request).call();
        if resp.ok() {
            Ok(())
        } else {
            Err(resp.into())
        }
    }

    /// Read a key/value pair from a given secret store.
    pub fn read_secret(&self, secret: &str, key: &str) -> Result<ReadResponse<String>, Error> {
        let request = ureq::get(&format!("{}/v1/secret/data/{}", self.host, secret));
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => {
                let mut resp: ReadSecretResponse = serde_json::from_str(&resp.into_string()?)?;
                let data = &mut resp.data;
                let value = data
                    .data
                    .remove(key)
                    .ok_or_else(|| Error::NotFound(secret.into(), key.into()))?;
                let created_time = data.metadata.created_time.clone();
                let version = data.metadata.version;
                Ok(ReadResponse::new(created_time, value, version))
            }
            404 => Err(Error::NotFound(secret.into(), key.into())),
            _ => Err(resp.into()),
        }
    }

    pub fn create_ed25519_key(&self, name: &str, exportable: bool) -> Result<(), Error> {
        let request = ureq::post(&format!("{}/v1/transit/keys/{}", self.host, name));
        let resp = self
            .upgrade_request(request)
            .send_json(json!({ "type": "ed25519", "exportable": exportable }));
        match resp.status() {
            200 => Ok(()),
            204 => Ok(()),
            404 => Err(Error::NotFound("transit/".into(), name.into())),
            _ => Err(resp.into()),
        }
    }

    pub fn delete_key(&self, name: &str) -> Result<(), Error> {
        let request = ureq::post(&format!("{}/v1/transit/keys/{}/config", self.host, name));
        let resp = self
            .upgrade_request(request)
            .send_json(json!({ "deletion_allowed": true }));

        if !resp.ok() {
            return Err(resp.into());
        }

        let request = ureq::delete(&format!("{}/v1/transit/keys/{}", self.host, name));
        let resp = self.upgrade_request(request).call();
        if resp.ok() {
            Ok(())
        } else {
            Err(resp.into())
        }
    }

    pub fn export_ed25519_key(
        &self,
        name: &str,
        version: Option<u32>,
    ) -> Result<Ed25519PrivateKey, Error> {
        let request = ureq::get(&format!(
            "{}/v1/transit/export/signing-key/{}",
            self.host, name
        ));
        let resp = self.upgrade_request(request).call();
        if resp.ok() {
            let export_key: ExportKeyResponse = serde_json::from_str(&resp.into_string()?)?;
            if let Some(version) = version {
                let key = export_key.data.keys.iter().find(|(k, _v)| **k == version);
                let (_, key) = key.ok_or_else(|| Error::NotFound("transit".into(), name.into()))?;
                // Composite key [private|public]
                Ok(Ed25519PrivateKey::try_from(&base64::decode(key)?[..32])?)
            } else if let Some(key) = export_key.data.keys.values().last() {
                // Composite key [private|public]
                Ok(Ed25519PrivateKey::try_from(&base64::decode(key)?[..32])?)
            } else {
                Err(Error::NotFound("transit".into(), name.into()))
            }
        } else {
            Err(resp.into())
        }
    }

    pub fn list_keys(&self) -> Result<Vec<String>, Error> {
        let request = ureq::request("LIST", &format!("{}/v1/transit/keys", self.host));
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => {
                let list_keys: ListKeysResponse = serde_json::from_str(&resp.into_string()?)?;
                Ok(list_keys.data.keys)
            }
            404 => Err(Error::NotFound("transit/".into(), "keys".into())),
            _ => Err(resp.into()),
        }
    }

    pub fn read_ed25519_key(
        &self,
        name: &str,
    ) -> Result<Vec<ReadResponse<Ed25519PublicKey>>, Error> {
        let request = ureq::get(&format!("{}/v1/transit/keys/{}", self.host, name));
        let resp = self.upgrade_request(request).call();
        match resp.status() {
            200 => {
                let read_key: ReadKeyResponse = serde_json::from_str(&resp.into_string()?)?;
                let mut read_resp = Vec::new();
                for (version, value) in read_key.data.keys {
                    read_resp.push(ReadResponse::new(
                        value.creation_time,
                        Ed25519PublicKey::try_from(base64::decode(&value.public_key)?.as_slice())?,
                        version,
                    ));
                }
                Ok(read_resp)
            }
            404 => Err(Error::NotFound("transit/".into(), name.into())),
            _ => Err(resp.into()),
        }
    }

    pub fn rotate_key(&self, name: &str) -> Result<(), Error> {
        let request = ureq::post(&format!("{}/v1/transit/keys/{}/rotate", self.host, name));
        let resp = self.upgrade_request(request).call();
        if resp.ok() {
            Ok(())
        } else {
            Err(resp.into())
        }
    }

    pub fn sign_ed25519(
        &self,
        name: &str,
        data: &[u8],
        version: Option<u32>,
    ) -> Result<Ed25519Signature, Error> {
        let data = if let Some(version) = version {
            json!({ "input": base64::encode(&data), "key_version": version })
        } else {
            json!({ "input": base64::encode(&data) })
        };

        let request = ureq::post(&format!("{}/v1/transit/sign/{}", self.host, name));
        let resp = self.upgrade_request(request).send_json(data);
        if resp.ok() {
            let signature: SignatureResponse = serde_json::from_str(&resp.into_string()?)?;
            let signature = &signature.data.signature;
            let signature_pieces: Vec<_> = signature.split(':').collect();
            let signature = signature_pieces
                .get(2)
                .ok_or_else(|| Error::SerializationError(signature.into()))?;
            Ok(Ed25519Signature::try_from(
                base64::decode(&signature)?.as_slice(),
            )?)
        } else {
            Err(resp.into())
        }
    }

    /// Create or update a key/value pair in a given secret store.
    pub fn write_secret(&self, secret: &str, key: &str, value: &str) -> Result<(), Error> {
        let request = ureq::put(&format!("{}/v1/secret/data/{}", self.host, secret));
        let resp = self
            .upgrade_request(request)
            .send_json(json!({ "data": { key: value } }));
        match resp.status() {
            200 => Ok(()),
            _ => Err(resp.into()),
        }
    }

    /// Returns whether or not the vault is unsealed (can be read from / written to). This can be
    /// queried without authentication.
    pub fn unsealed(&self) -> Result<bool, Error> {
        let request = ureq::get(&format!("{}/v1/sys/seal-status", self.host));
        let resp = self.upgrade_request_without_token(request).call();
        println!("{:?}", resp.synthetic_error());
        match resp.status() {
            200 => {
                let resp: SealStatusResponse = serde_json::from_str(&resp.into_string()?)?;
                Ok(!resp.sealed)
            }
            _ => Err(resp.into()),
        }
    }

    fn upgrade_request(&self, request: ureq::Request) -> ureq::Request {
        let mut request = self.upgrade_request_without_token(request);
        request.set("X-Vault-Token", &self.token);
        request
    }

    fn upgrade_request_without_token(&self, mut request: ureq::Request) -> ureq::Request {
        request.timeout_connect(TIMEOUT);
        if let Some(tls_config) = self.tls_config.as_ref() {
            request.set_tls_config(tls_config.clone());
        }
        request
    }
}

/// Provides a simple wrapper for all read APIs.
#[derive(Debug)]
pub struct ReadResponse<T> {
    pub creation_time: String,
    pub value: T,
    pub version: u32,
}

impl<T> ReadResponse<T> {
    pub fn new(creation_time: String, value: T, version: u32) -> Self {
        Self {
            creation_time,
            value,
            version,
        }
    }
}

/// Below is a sample output of a CreateTokenResponse. Only the fields leveraged by this framework
/// are decoded.
/// {
///   "request_id": "f00341c1-fad5-f6e6-13fd-235617f858a1",
///   "lease_id": "",
///   "renewable": false,
///   "lease_duration": 0,
///   "data": null,
///   "wrap_info": null,
///   "warnings": [
///     "Policy \"stage\" does not exist",
///     "Policy \"web\" does not exist"
///   ],
///   "auth": {
///     "client_token": "s.wOrq9dO9kzOcuvB06CMviJhZ",
///     "accessor": "B6oixijqmeR4bsLOJH88Ska9",
///     "policies": ["default", "stage", "web"],
///     "token_policies": ["default", "stage", "web"],
///     "metadata": {
///       "user": "armon"
///     },
///     "lease_duration": 3600,
///     "renewable": true,
///     "entity_id": "",
///     "token_type": "service",
///     "orphan": false
///   }
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CreateTokenResponse {
    auth: CreateTokenAuth,
}

/// See CreateTokenResponse
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CreateTokenAuth {
    client_token: String,
}

/// Below is a sample output of ExportKeyResponse
/// {
///   "data": {
///    "name": "foo",
///    "keys": {
///      "1": "eyXYGHbTmugUJn6EtYD/yVEoF6pCxm4R/cMEutUm3MY=",
///      "2": "Euzymqx6iXjS3/NuGKDCiM2Ev6wdhnU+rBiKnJ7YpHE="
///    }
///  }
///}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ExportKeyResponse {
    data: ExportKey,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ExportKey {
    name: String,
    keys: BTreeMap<u32, String>,
}

/// Below is a sample output of ListPoliciesResponse
/// {
///   "policies": ["root", "deploy"]
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ListPoliciesResponse {
    policies: Vec<String>,
}

/// Below is a sample output of ListKeysResponse.
///
/// {
///   "data": {
///     "keys": ["foo", "bar"]
///   },
///   "lease_duration": 0,
///   "lease_id": "",
///   "renewable": false
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ListKeysResponse {
    data: ListKeys,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ListKeys {
    keys: Vec<String>,
}

/// Below is a sample output of ReadKeyResponse
/// {
///   "data": {
///     "type": "aes256-gcm96",
///     "deletion_allowed": false,
///     "derived": false,
///     "exportable": false,
///     "allow_plaintext_backup": false,
///     "keys": {
///       "1": 1442851412
///     },
///     "min_decryption_version": 1,
///     "min_encryption_version": 0,
///     "name": "foo",
///     "supports_encryption": true,
///     "supports_decryption": true,
///     "supports_derivation": true,
///     "supports_signing": false
///   }
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadKeyResponse {
    data: ReadKeys,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadKeys {
    keys: BTreeMap<u32, ReadKey>,
    name: String,
    #[serde(rename = "type")]
    key_type: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadKey {
    creation_time: String,
    public_key: String,
}

/// Below is a sample output of ReadSecretListResponse. All fields are decoded and used.
/// Note: in the case that a secret contains a subpath, that will be returned. Vault does
/// not automatically recurse.
/// {
///   "data": {
///     "keys": ["foo", "foo/"]
///   }
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretListResponse {
    data: ReadSecretListData,
}

/// See ReadSecretListResponse
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretListData {
    keys: Vec<String>,
}

/// Below is a sample output of ReadSecretResponse. Note, this returns all keys within a secret.
/// Only fields leveraged by this framework are decoded.
/// {
///   "data": {
///     "data": {
///       "foo": "bar"
///     },
///     "metadata": {
///       "created_time": "2018-03-22T02:24:06.945319214Z",
///       "deletion_time": "",
///       "destroyed": false,
///       "version": 1
///     }
///   }
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretResponse {
    data: ReadSecretData,
}

/// See ReadPolicyResponse
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretData {
    data: BTreeMap<String, String>,
    metadata: ReadSecretMetadata,
}

/// See ReadPolicyResponse
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretMetadata {
    created_time: String,
    version: u32,
}

/// This data structure is used to represent both policies read from Vault and written to Vault.
/// Thus the same Policy read, can then be written back after amending. Vault stores the rules or
/// per path policies in an encoded json blob, so that effectively means json within json, hence
/// the unusual semantics below.
/// {
///   rules: json!{
///     path: {
///       'auth/*': { capabilities: ['create', 'read', 'update', 'delete', 'list', 'sudo'] },
///       'sys/auth/*': { capabilities: ['create', 'read', 'update', 'delete', 'sudo'] },
///     }
///   }
/// }
/// Note: Vault claims rules is deprecated and policy should be used instead, but that doesn't seem
/// to work and makes the reading asymmetrical from the writing.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Policy {
    #[serde(skip)]
    internal_rules: PolicyPaths,
    rules: String,
}

impl Policy {
    pub fn new() -> Self {
        Self {
            internal_rules: PolicyPaths {
                path: BTreeMap::new(),
            },
            rules: "".to_string(),
        }
    }

    pub fn add_policy(&mut self, path: &str, capabilities: Vec<Capability>) {
        let path_policy = PathPolicy { capabilities };
        self.internal_rules
            .path
            .insert(path.to_string(), path_policy);
    }
}

impl TryFrom<serde_json::Value> for Policy {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let mut policy: Self = serde_json::from_value(value)?;
        policy.internal_rules = serde_json::from_str(&policy.rules)?;
        Ok(policy)
    }
}

impl TryFrom<&Policy> for serde_json::Value {
    type Error = serde_json::Error;

    fn try_from(policy: &Policy) -> Result<Self, Self::Error> {
        Ok(json!({"rules": serde_json::to_string(&policy.internal_rules)?}))
    }
}

/// Represents the policy for a given path.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PolicyPaths {
    path: BTreeMap<String, PathPolicy>,
}

/// Represents the set of capabilities used within a policy.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PathPolicy {
    capabilities: Vec<Capability>,
}

/// The various set of capabilities available to a policy within Vault.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Create,
    Delete,
    Deny,
    List,
    Read,
    Sudo,
    Update,
}

/// Below is an example of SignatureResponse.
/// {
///   "data": {
///     "signature": "vault:v1:MEUCIQCyb869d7KWuA0hBM9b5NJrmWzMW3/pT+0XYCM9VmGR+QIgWWF6ufi4OS2xo1eS2V5IeJQfsi59qeMWtgX0LipxEHI="
///   }
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SignatureResponse {
    data: Signature,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Signature {
    signature: String,
}

/// Below is an example of SealStatusResponse. Only the fields leveraged by this framework are
/// decoded.
/// {
///   "type": "shamir",
///   "sealed": false,
///   "t": 3,
///   "n": 5,
///   "progress": 0,
///   "version": "0.9.0",
///   "cluster_name": "vault-cluster-d6ec3c7f",
///   "cluster_id": "3e8b3fec-3749-e056-ba41-b62a63b997e8",
///   "nonce": "ef05d55d-4d2c-c594-a5e8-55bc88604c24"
/// }
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SealStatusResponse {
    sealed: bool,
}
