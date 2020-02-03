// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};
use thiserror::Error;
use ureq;

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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<ureq::Response> for Error {
    fn from(response: ureq::Response) -> Self {
        Error::HttpError(response.status(), response.status_line().into())
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
}

impl Client {
    pub fn new(host: String, token: String) -> Self {
        Self { host, token }
    }

    /// Create a new policy in Vault, see the explanation for Policy for how the data is
    /// structured. Vault does not distingush a create and update. An update must first read the
    /// existing policy, amend the contents,  and then be applied via this API.
    pub fn set_policy(&self, policy_name: &str, policy: &Policy) -> Result<(), Error> {
        let response = ureq::post(&format!("{}/v1/sys/policy/{}", self.host, policy_name))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .send_json(policy.try_into()?);
        if response.ok() {
            Ok(())
        } else {
            Err(response.into())
        }
    }

    /// Retrieves the policy at the given policy name.
    pub fn read_policy(&self, policy_name: &str) -> Result<Policy, Error> {
        let response = ureq::get(&format!("{}/v1/sys/policy/{}", self.host, policy_name))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .call();
        match response.status() {
            200 => Ok(Policy::try_from(response.into_json()?)?),
            _ => Err(response.into()),
        }
    }

    /// Creates a new token or identity for accessing Vault. The token will have access to anything
    /// under the default policy and any perscribed policies.
    pub fn create_token(&self, policies: Vec<&str>) -> Result<String, Error> {
        let response = ureq::post(&format!("{}/v1/auth/token/create", self.host))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .send_json(json!({ "policies": policies }));
        if response.ok() {
            let response: CreateTokenResponse = serde_json::from_str(&response.into_string()?)?;
            Ok(response.auth.client_token)
        } else {
            Err(response.into())
        }
    }

    /// List all stored secrets
    pub fn list_secrets(&self, secret: &str) -> Result<Vec<String>, Error> {
        let response = ureq::request(
            "LIST",
            &format!("{}/v1/secret/metadata/{}", self.host, secret),
        )
        .set("X-Vault-Token", &self.token)
        .timeout_connect(10_000)
        .call();
        match response.status() {
            200 => {
                let response: ReadSecretListResponse =
                    serde_json::from_str(&response.into_string()?)?;
                Ok(response.data.keys)
            }
            // There are no secrets.
            404 => Ok(vec![]),
            _ => Err(response.into()),
        }
    }

    /// Delete a specific secret store
    pub fn delete_secret(&self, secret: &str) -> Result<(), Error> {
        let response = ureq::delete(&format!("{}/v1/secret/metadata/{}", self.host, secret))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .call();
        if response.ok() {
            Ok(())
        } else {
            Err(response.into())
        }
    }

    /// Read a key/value pair from a given secret store.
    pub fn read_secret(&self, secret: &str, key: &str) -> Result<String, Error> {
        let response = ureq::get(&format!("{}/v1/secret/data/{}", self.host, secret))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .call();
        match response.status() {
            200 => {
                let mut response: ReadSecretResponse =
                    serde_json::from_str(&response.into_string()?)?;
                let value = response
                    .data
                    .data
                    .remove(key)
                    .ok_or_else(|| Error::NotFound(secret.into(), key.into()))?;
                Ok(value)
            }
            404 => Err(Error::NotFound(secret.into(), key.into())),
            _ => Err(response.into()),
        }
    }

    /// Returns whether or not the vault is unsealed (can be read from / written to). This can be
    /// queried without authentication.
    pub fn unsealed(&self) -> Result<bool, Error> {
        let response = ureq::get(&format!("{}/v1/sys/seal-status", self.host))
            .timeout_connect(10_000)
            .call();
        match response.status() {
            200 => {
                let response: SealStatusResponse = serde_json::from_str(&response.into_string()?)?;
                Ok(!response.sealed)
            }
            _ => Err(response.into()),
        }
    }

    /// Create or update a key/value pair in a given secret store.
    pub fn write_secret(&self, secret: &str, key: &str, value: &str) -> Result<(), Error> {
        let response = ureq::put(&format!("{}/v1/secret/data/{}", self.host, secret))
            .set("X-Vault-Token", &self.token)
            .timeout_connect(10_000)
            .send_json(json!({ "data": { key: value } }));
        match response.status() {
            200 => Ok(()),
            _ => Err(response.into()),
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
    data: HashMap<String, String>,
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
                path: HashMap::new(),
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
    path: HashMap<String, PathPolicy>,
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
