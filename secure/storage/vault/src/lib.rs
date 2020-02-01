// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
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

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

/// Client provides a client around the restful interface to a Vault servce. Learn more
/// here: https://www.vaultproject.io/api-docs/
pub struct Client {
    host: String,
    token: String,
}

impl Client {
    pub fn new(host: String, token: String) -> Self {
        Self { host, token }
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

    /// Returns whether or not the vault is unsealed (can be read from / written to)
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

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CreateTokenResponse {
    auth: CreateTokenAuth,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CreateTokenAuth {
    client_token: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretListResponse {
    data: ReadSecretListData,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretListData {
    keys: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretResponse {
    data: ReadSecretData,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ReadSecretData {
    data: HashMap<String, String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SealStatusResponse {
    sealed: bool,
}
