// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{format_err, Result};
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub commit: GitCommitInfo,
}

#[derive(Debug, Deserialize)]
pub struct GitCommitInfo {
    pub author: Author,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

pub struct GitHub {
    client: reqwest::Client,
}

impl GitHub {
    pub fn new() -> GitHub {
        let client = reqwest::Client::new();
        GitHub { client }
    }

    /// repo in format owner/repo_name
    /// sha can be long or short hash, or branch name
    /// Paging is not implemented yet
    pub fn get_commits(&self, repo: &str, sha: &str) -> Result<Vec<CommitInfo>> {
        let url = format!("https://api.github.com/repos/{}/commits?sha={}", repo, sha);
        let url: Url = url.parse().expect("Failed to parse github url");
        let request = self.client.get(url);
        let mut response = request
            .send()
            .map_err(|e| format_err!("Failed to query github: {:?}", e))?;
        let response: Vec<CommitInfo> = response
            .json()
            .map_err(|e| format_err!("Failed to parse github response: {:?}", e))?;
        Ok(response)
    }
}
