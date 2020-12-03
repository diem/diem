// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_logger::json_log::JsonLogEntry;
use reqwest::blocking;
use std::collections::HashMap;

pub mod node_debug_service;

/// Implement default utility client for NodeDebugInterface
pub struct NodeDebugClient {
    client: blocking::Client,
    addr: String,
}

impl NodeDebugClient {
    /// Create NodeDebugInterfaceClient from a valid socket address.
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        let client = blocking::Client::new();
        let addr = format!("http://{}:{}", address.as_ref(), port);

        Self { client, addr }
    }

    pub fn get_node_metric<S: AsRef<str>>(&mut self, metric: S) -> Result<Option<i64>> {
        let metrics = self.get_node_metrics()?;
        Ok(metrics.get(metric.as_ref()).cloned())
    }

    pub fn get_node_metrics(&mut self) -> Result<HashMap<String, i64>> {
        let response = self.client.get(&format!("{}/metrics", self.addr)).send()?;

        response
            .json::<HashMap<String, String>>()?
            .into_iter()
            .map(|(k, v)| match v.parse::<i64>() {
                Ok(v) => Ok((k, v)),
                Err(_) => Err(anyhow::format_err!(
                    "Failed to parse stat value to i64 {}: {}",
                    &k,
                    &v
                )),
            })
            .collect()
    }

    pub fn get_events(&mut self) -> Result<Vec<JsonLogEntry>> {
        let response = self.client.get(&format!("{}/events", self.addr)).send()?;

        Ok(response.json()?)
    }
}

/// Implement default utility client for AsyncNodeDebugInterface
pub struct AsyncNodeDebugClient {
    client: reqwest::Client,
    addr: String,
}

impl AsyncNodeDebugClient {
    /// Create AsyncNodeDebugInterface from a valid socket address.
    pub fn new<A: AsRef<str>>(client: reqwest::Client, address: A, port: u16) -> Self {
        let addr = format!("http://{}:{}", address.as_ref(), port);

        Self { client, addr }
    }

    pub async fn get_node_metric<S: AsRef<str>>(&mut self, metric: S) -> Result<Option<i64>> {
        let metrics = self.get_node_metrics().await?;
        Ok(metrics.get(metric.as_ref()).cloned())
    }

    pub async fn get_node_metrics(&mut self) -> Result<HashMap<String, i64>> {
        let response = self
            .client
            .get(&format!("{}/metrics", self.addr))
            .send()
            .await?;

        response
            .json::<HashMap<String, String>>()
            .await?
            .into_iter()
            .map(|(k, v)| match v.parse::<i64>() {
                Ok(v) => Ok((k, v)),
                Err(_) => Err(anyhow::format_err!(
                    "Failed to parse stat value to i64 {}: {}",
                    &k,
                    &v
                )),
            })
            .collect()
    }

    pub async fn get_events(&mut self) -> Result<Vec<JsonLogEntry>> {
        let response = self
            .client
            .get(&format!("{}/events", self.addr))
            .send()
            .await?;

        Ok(response.json().await?)
    }
}
