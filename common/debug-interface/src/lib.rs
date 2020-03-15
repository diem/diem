// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::proto::{
    node_debug_interface_client::NodeDebugInterfaceClient, GetEventsRequest, GetEventsResponse,
    GetNodeDetailsRequest,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::runtime::{Builder, Runtime};

// Generated
pub mod proto;

pub mod json_log;
pub mod libra_trace;
pub mod node_debug_service;

pub mod prelude {
    pub use crate::{end_trace, event, trace_code_block, trace_edge, trace_event};
}

/// Implement default utility client for NodeDebugInterface
pub struct NodeDebugClient {
    // Currently the runtime but be ordered before the tonic client to ensure that the runtime is
    // dropped last when this struct is dropped.
    // See https://github.com/tokio-rs/tokio/issues/1948 for more info.
    rt: Runtime,
    addr: String,
    client: Option<NodeDebugInterfaceClient<tonic::transport::Channel>>,
}

impl NodeDebugClient {
    /// Create NodeDebugInterfaceClient from a valid socket address.
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        let rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        let addr = format!("http://{}:{}", address.as_ref(), port);

        Self {
            client: None,
            addr,
            rt,
        }
    }

    fn client(
        &mut self,
    ) -> Result<(
        &mut Runtime,
        &mut NodeDebugInterfaceClient<tonic::transport::Channel>,
    )> {
        if self.client.is_none() {
            self.client = Some(
                self.rt
                    .block_on(NodeDebugInterfaceClient::connect(self.addr.clone()))?,
            );
        }

        // client is guaranteed to be populated by the time we reach here
        Ok((&mut self.rt, self.client.as_mut().unwrap()))
    }

    pub fn get_node_metric<S: AsRef<str>>(&mut self, metric: S) -> Result<Option<i64>> {
        let metrics = self.get_node_metrics()?;
        Ok(metrics.get(metric.as_ref()).cloned())
    }

    pub fn get_node_metrics(&mut self) -> Result<HashMap<String, i64>> {
        let (rt, client) = self.client()?;
        let response = rt
            .block_on(client.get_node_details(GetNodeDetailsRequest::default()))
            .context("Unable to query Node metrics")?;

        response
            .into_inner()
            .stats
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

    pub fn get_events(&mut self) -> Result<GetEventsResponse> {
        let (rt, client) = self.client()?;
        let response = rt
            .block_on(client.get_events(GetEventsRequest::default()))
            .context("Unable to query Node events")?;
        Ok(response.into_inner())
    }
}

/// Implement default utility client for AsyncNodeDebugInterface 
pub struct AsyncNodeDebugClient {
    addr: String,
    client: Option<NodeDebugInterfaceClient<tonic::transport::Channel>>,
  }
  
  impl AsyncNodeDebugClient {
    /// Create AsyncNodeDebugInterface from a valid socket address.
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        let addr = format!("http://{}:{}", address.as_ref(), port);
  
        Self {
            client: None,
            addr,
        }
    }
  
    async fn client(
        &mut self,
    ) -> Result<
        &mut NodeDebugInterfaceClient<tonic::transport::Channel>,
    > {
        if self.client.is_none() {
            self.client = Some(
              NodeDebugInterfaceClient::connect(self.addr.clone()).await?
            );
        }
  
        // client is guaranteed to be populated by the time we reach here
        Ok(self.client.as_mut().unwrap())
    }
  
    pub async fn get_node_metric<S: AsRef<str>>(&mut self, metric: S) -> Result<Option<i64>> {
        let metrics = self.get_node_metrics().await?;
        Ok(metrics.get(metric.as_ref()).cloned())
    }
  
    pub async fn get_node_metrics(&mut self) -> Result<HashMap<String, i64>> {
        let client = self.client().await?;
        let response = client.get_node_details(GetNodeDetailsRequest::default()).await
            .context("Unable to query Node metrics")?;
  
        response
            .into_inner()
            .stats
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
  
    pub async fn get_events(&mut self) -> Result<GetEventsResponse> {
        let client = self.client().await?;
        let response = client.get_events(GetEventsRequest::default()).await
            .context("Unable to query Node events")?;
        Ok(response.into_inner())
    }
  }
  