// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use proxy::Proxy;
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
    #[error("Http error, status code: {0}, status text: {1}, body: {2}")]
    HttpError(u16, String, String),
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
        if let Some(e) = resp.synthetic_error() {
            // Local error
            Error::InternalError(e.to_string())
        } else {
            // Clear the buffer
            let status = resp.status();
            let status_text = resp.status_text().to_string();
            match resp.into_string() {
                Ok(body) => Error::HttpError(status, status_text, body),
                Err(e) => Error::InternalError(e.to_string()),
            }
        }
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
    branch: String,
    owner: String,
    repository: String,
    token: String,
}

impl Client {
    pub fn new(owner: String, repository: String, branch: String, token: String) -> Self {
        Self {
            branch,
            owner,
            repository,
            token,
        }
    }

    /// Delete a file from a GitHub repository
    pub fn delete_file(&self, path: &str) -> Result<(), Error> {
        // Occasionally GitHub sends us back delayed results and the file is already deleted.
        let hash = match self.get_sha(path) {
            Ok(hash) => hash,
            Err(Error::NotFound(_)) => return Ok(()),
            Err(e) => return Err(e),
        };

        let resp = self
            .upgrade_request(ureq::delete(&self.post_url(path)))
            .send_json(
                json!({ "branch": self.branch.to_string(), "message": "diem-secure", "sha": hash }),
            );

        match resp.status() {
            200 => Ok(()),
            _ => Err(resp.into()),
        }
    }

    /// Recursively delete all files, which as a by product will delete all folders
    pub fn delete_directory(&self, path: &str) -> Result<(), Error> {
        let files = self.get_directory(path.trim_end_matches('/'))?;
        for file in files {
            if file.ends_with('/') {
                self.delete_directory(&file[..file.len() - 1])?;
                continue;
            }
            self.delete_file(&file)?;
        }
        Ok(())
    }

    /// Retrieve a list of branches, this is effectively a status check on the repository
    pub fn get_branches(&self) -> Result<Vec<String>, Error> {
        let url = format!("{}/repos/{}/{}/branches", URL, self.owner, self.repository);
        let resp = self.upgrade_request(ureq::get(&url)).call();

        match resp.status() {
            200 => {
                let resp = resp.into_string()?;
                let branches: Vec<Branch> = serde_json::from_str(&resp)?;
                Ok(branches.into_iter().map(|b| b.name).collect())
            }
            _ => Err(resp.into()),
        }
    }

    /// Retrieve the name of contents within a given directory, note, there are no such thing as
    /// empty directories in git.
    pub fn get_directory(&self, path: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .get_internal(path)?
            .into_iter()
            .map(|entry| {
                if entry.content_type == "dir" {
                    entry.path + "/"
                } else {
                    entry.path
                }
            })
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
            // Apparently GitHub introduces newlines every 60 characters and at the end of content,
            // this strips those characters out.
            Ok(content.lines().collect::<Vec<_>>().join(""))
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
            Ok(hash) => {
                json!({ "branch": self.branch.to_string(), "content": content, "message": format!("[diem-management] {}", path), "sha": hash })
            }
            Err(Error::NotFound(_)) => {
                json!({ "branch": self.branch.to_string(), "content": content, "message": format!("[diem-management] {}", path) })
            }
            Err(e) => return Err(e),
        };

        let resp = self
            .upgrade_request(ureq::put(&self.post_url(path)))
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

        let proxy = Proxy::new();
        let host = request.get_host().expect("unable to get the host");
        let proxy_url = proxy.https(&host);
        if let Some(proxy_url) = proxy_url {
            request.set_proxy(ureq::Proxy::new(proxy_url).expect("Unable to parse proxy_url"));
        }
        request
    }

    /// Get can read files or directories, this makes it easier to use
    fn get_internal(&self, path: &str) -> Result<Vec<GetResponse>, Error> {
        let resp = self.upgrade_request(ureq::get(&self.get_url(path))).call();
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
            404 => Err(Error::NotFound(path.into())),
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

    fn post_url(&self, path: &str) -> String {
        format!(
            "{}/repos/{}/{}/contents/{}",
            URL, self.owner, self.repository, path
        )
    }

    fn get_url(&self, path: &str) -> String {
        format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            URL, self.owner, self.repository, path, self.branch
        )
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Branch {
    name: String,
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
    // This framework will not create branches, it must already exist!
    const BRANCH: &str = "BRANCH";
    const TOKEN: &str = "TOKEN";

    #[ignore]
    #[test]
    fn test_files() {
        let path = "data.txt";
        let value1 = "hello";
        let value1_encoded = base64::encode(&value1);
        let value2 = "world";
        let value2_encoded = base64::encode(&value2);

        let github = Client::new(OWNER.into(), REPOSITORY.into(), BRANCH.into(), TOKEN.into());

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

        // Verify delete_file
        github.delete_file(path).unwrap();
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

        let github = Client::new(OWNER.into(), REPOSITORY.into(), BRANCH.into(), TOKEN.into());

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
        github.delete_file(path1).unwrap();

        // Verify one is good and the other is gone
        github.get_directory(path1_root).unwrap_err();

        let b64_value = base64::decode(github.get_file(path2).unwrap()).unwrap();
        let value = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, value2);

        // Cleanup the rest
        github.delete_file(path2).unwrap();
        github.get_directory(path2_root).unwrap_err();
    }

    #[ignore]
    #[test]
    fn test_recursive_delete_file() {
        let root = "root_dir";
        let file0 = "root_dir/another_dir/another_dir/ok.txt";
        let file1 = "root_dir/another_dir/ok.txt";
        let value = "hello";
        let value_encoded = base64::encode(&value);

        let github = Client::new(OWNER.into(), REPOSITORY.into(), BRANCH.into(), TOKEN.into());

        github.put(file0, &value_encoded).unwrap();
        github.put(file1, &value_encoded).unwrap();

        let b64_value = base64::decode(github.get_file(file0).unwrap()).unwrap();
        let rv = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, rv);

        let b64_value = base64::decode(github.get_file(file1).unwrap()).unwrap();
        let rv = std::str::from_utf8(&b64_value).unwrap();
        assert_eq!(value, rv);

        github.delete_directory(root).unwrap();
        github.get_file(file0).unwrap_err();
        github.get_file(file1).unwrap_err();
    }

    #[ignore]
    #[test]
    fn test_branches() {
        let github = Client::new(OWNER.into(), REPOSITORY.into(), BRANCH.into(), TOKEN.into());
        let branches = github.get_branches().unwrap();
        assert!(!branches.is_empty());
        assert!(branches.iter().any(|b| b == BRANCH));
    }
}
