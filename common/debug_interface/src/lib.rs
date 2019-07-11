// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::{
    node_debug_interface::{DumpJemallocHeapProfileRequest, GetNodeDetailsRequest},
    node_debug_interface_grpc::NodeDebugInterfaceClient,
};
use failure::prelude::*;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::{collections::HashMap, sync::Arc};

// Generated
pub mod proto;

pub mod node_debug_helpers;
pub mod node_debug_service;

/// Implement default utility client for NodeDebugInterface
pub struct NodeDebugClient {
    client: NodeDebugInterfaceClient,
}

impl NodeDebugClient {
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        Self::from_socket_addr_str(&format!("{}:{}", address.as_ref(), port))
    }

    /// Create NodeDebugInterfaceClient from a valid socket address.
    pub fn from_socket_addr_str<A: AsRef<str>>(socket_addr: A) -> Self {
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-debug-").build());
        let ch = ChannelBuilder::new(env).connect(&socket_addr.as_ref());
        let client = NodeDebugInterfaceClient::new(ch);

        Self { client }
    }

    pub fn get_node_metric<S: AsRef<str>>(&self, metric: S) -> Result<Option<i64>> {
        let metrics = self.get_node_metrics()?;
        Ok(metrics.get(metric.as_ref()).cloned())
    }

    pub fn get_node_metrics(&self) -> Result<HashMap<String, i64>> {
        let response = self
            .client
            .get_node_details(&GetNodeDetailsRequest::new())
            .context("Unable to query Node metrics")?;

        response
            .stats
            .into_iter()
            .map(|(k, v)| match v.parse::<i64>() {
                Ok(v) => Ok((k, v)),
                Err(_) => Err(format_err!(
                    "Failed to parse stat value to i64 {}: {}",
                    &k,
                    &v
                )),
            })
            .collect()
    }

    pub fn dump_heap_profile(&self) -> Result<i32> {
        let response = self
            .client
            .dump_jemalloc_heap_profile(&DumpJemallocHeapProfileRequest::new())
            .context("Unable to request heap dump")?;

        Ok(response.status_code)
    }
}
