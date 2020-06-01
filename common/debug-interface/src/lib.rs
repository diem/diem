// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::json_log::JsonLogEntry;
use anyhow::Result;
use reqwest::blocking;
use std::collections::HashMap;

pub mod json_log;
pub mod libra_trace;
pub mod node_debug_service;

pub mod prelude {
    pub use crate::{end_trace, event, trace_code_block, trace_edge, trace_event};
}

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
