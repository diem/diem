// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// Request timeout for github operations
const ACCEPT_HEADER: &str = "Accept";
const ACCEPT_VALUE: &str = "Accept: application/vnd.github.v3+json";
const TIMEOUT: u64 = 10_000;
const URL: &str = "https://api.github.com";

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Http error: {1}")]
    HttpError(u16, String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Missing field {0}")]
    MissingField(String),
    #[error("404: Not Found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
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

/// Client provides a client around the restful interface to GitHub API version 3. Learn more
/// here: https://developer.github.com/v3
///
/// This is not intended for securely storing private data, though perhaps it could with a private
/// repository. The tooling is intended to be used to exchange data in an authenticated fashion
/// across multiple peers.
pub struct Client {
    repository: String,
    owner: String,
    token: String,
}

impl Client {
    pub fn new(owner: String, repository: String, token: String) -> Self {
        Self {
            repository,
            owner,
            token,
        }
    }

    /// Delete a file from a GitHub repository
    pub fn delete(&self, path: &str) -> Result<(), Error> {
        let hash = self.get_sha(path)?;
        let resp = self
            .upgrade_request(ureq::delete(&self.url(path)))
            .send_json(json!({ "message": "libra-secure", "sha": hash }));

        match resp.status() {
            200 => Ok(()),
            _ => Err(resp.into()),
        }
    }

    /// Retrieve the name of contents within a given directory, note, there are no such thing as
    /// empty directories in git.
    pub fn get_directory(&self, path: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .get_internal(path)?
            .into_iter()
            .map(|entry| entry.path)
            .collect())
    }

    /// Retrieve the contents of a file.
    pub fn get_file(&self, path: &str) -> Result<String, Error> {
        let value = self.get_internal(path)?;
        if value.len() == 1 && value[0].path == path {
            let content = value[0]
                .content
                .as_ref()
                .ok_or_else(|| Error::InternalError("No content found".into()))?;
            Ok(content.trim().into())
        } else {
            Err(Error::InternalError(format!(
                "get mismatch, found {} entries",
                value.len()
            )))
        }
    }

    /// Create or update a file.
    pub fn put(&self, path: &str, content: &str) -> Result<(), Error> {
        let json = match self.get_sha(path) {
            Ok(hash) => json!({ "content": content, "message": "libra-secure", "sha": hash }),
            Err(Error::NotFound(_)) => json!({ "content": content, "message": "libra-secure" }),
            Err(e) => return Err(e),
        };

        let resp = self
            .upgrade_request(ureq::put(&self.url(path)))
            .send_json(json);

        match resp.status() {
            200 => Ok(()),
            201 => Ok(()),
            _ => Err(resp.into()),
        }
    }

    /// Simple wrapper around requests to add default parameters to the request
    fn upgrade_request(&self, mut request: ureq::Request) -> ureq::Request {
        request
            .set("Authorization", &format!("token {}", self.token))
            .set(ACCEPT_HEADER, ACCEPT_VALUE)
            .timeout_connect(TIMEOUT);
        request
    }

    /// Get can read files or directories, this makes it easier to use
    fn get_internal(&self, path: &str) -> Result<Vec<GetResponse>, Error> {
        let resp = self.upgrade_request(ureq::get(&self.url(path))).call();
        match resp.status() {
            200 => {
                let resp = resp.into_string()?;
                let get_resp: Result<GetResponse, Error> =
                    serde_json::from_str(&resp).map_err(|e| e.into());

                if let Ok(get_resp) = get_resp {
                    return Ok(vec![get_resp]);
                }

                let get_resp: Result<Vec<GetResponse>, Error> =
                    serde_json::from_str(&resp).map_err(|e| e.into());
                if let Ok(get_resp) = get_resp {
                    return Ok(get_resp);
                }

                Err(Error::SerializationError(resp))
            }
            404 => Err(Error::NotFound(self.url(path))),
            _ => Err(resp.into()),
        }
    }

    /// Returns the sha hash of a file according to GitHub
    fn get_sha(&self, path: &str) -> Result<String, Error> {
        let value = self.get_internal(path)?;
        if let Some(value) = value.iter().find(|entry| entry.path == path) {
            Ok(value.sha.clone())
        } else {
            // This is a directory
            Err(Error::InternalError(format!(
                "get mismatch, found {} entries",
                value.len()
            )))
        }
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/repos/{}/{}/contents/{}",
            URL, self.owner, self.repository, path
        )
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct GetResponse {
    #[serde(rename = "type")]
    content_type: String,
    path: String,
    sha: String,
    content: Option<String>,
}

// This test suite depends on the developer providing an active repository, the account where that
// repository is hosted, and a token that can modify that repository. Due to the nature of these
// tests, they cannot be provided in this framework and the tests are labeled as ignored
//
// GitHub updates can misbehave due to changes in file structure thus dependent operations can
// fail, for example, an update of a file reads its hash and then updates. If another operation
// happens after the hash retrieval but before the update, the hash will change resulting in a
// failed update. Therefore these tests must be run in series via:
// `cargo xtest -- --ignored --test-threads=1`
#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "OWNER";
    const REPOSITORY: &str = "REPOSITORY";
    const TOKEN: &str = "TOKEN";

    #[ignore]
    #[test]
    fn test_files() {
        let path = "data.txt";
        let value1 = "hello";
        let value1_encoded = base64::encode(&value1);
        let value2 = "world";
        let value2_encoded = base64::encode(&value2);

        let github = Client::new(OWNER.into(), REPOSITORY.into(), TOKEN.into());

        // Try a create
        github.get_file(path).unwrap_err();
        github.put(path, &value1_encoded).unwrap();
        let b64_value = base64::decode(github.get_file(path).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value1);

        // Try an update
        github.put(path, &value2_encoded).unwrap();
        let b64_value = base64::decode(github.get_file(path).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value2);

        // Verify delete
        github.delete(path).unwrap();
        github.get_file(path).unwrap_err();
    }

    #[ignore]
    #[test]
    fn test_directories() {
        let path1_root = "dir";
        let path1 = "dir/data1.txt";
        let value1 = "hello";
        let value1_encoded = base64::encode(&value1);

        let path2_root = "dir1";
        let path2 = "dir1/data1.txt";
        let value2 = "world";
        let value2_encoded = base64::encode(&value2);

        let github = Client::new(OWNER.into(), REPOSITORY.into(), TOKEN.into());

        // Initialize two directories with a file each
        github.put(path1, &value1_encoded).unwrap();
        github.put(path2, &value2_encoded).unwrap();

        // Verify the contents
        let b64_value = base64::decode(github.get_file(path1).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value1);

        let b64_value = base64::decode(github.get_file(path2).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value2);

        // Delete one of the directories
        github.delete(path1).unwrap();

        // Verify one is good and the other is gone
        github.get_directory(path1_root).unwrap_err();

        let b64_value = base64::decode(github.get_file(path2).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value2);

        // Cleanup the rest
        github.delete(path2).unwrap();
        github.get_directory(path2_root).unwrap_err();
    }
}
