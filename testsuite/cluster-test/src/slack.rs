// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{bail, format_err, Result};
use reqwest::{self, Url};
use serde_json::{self, json};

pub struct SlackClient {
    client: reqwest::blocking::Client,
}

impl SlackClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::new();
        Self { client }
    }

    pub fn send_message(&self, url: &Url, msg: &str) -> Result<()> {
        let msg = json!({ "text": msg });
        let msg = serde_json::to_string(&msg)
            .map_err(|e| format_err!("Failed to serialize message for slack: {:?}", e))?;
        let request = self.client.post(url.clone()).body(msg);
        let response = request
            .send()
            .map_err(|e| format_err!("Failed to send slack message: {:?}", e))?;
        if !response.status().is_success() {
            bail!("Slack service returned error code: {}", response.status())
        }
        Ok(())
    }
}

impl Default for SlackClient {
    fn default() -> Self {
        Self::new()
    }
}
