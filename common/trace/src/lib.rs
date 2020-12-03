// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use diem_logger::json_log::JsonLogEntry;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub mod counters;
pub mod trace;

pub mod prelude {
    pub use crate::{
        end_trace, node_sampling_data, send_logs, trace_code_block, trace_edge, trace_event,
    };
}

pub use trace::{diem_trace_set, is_selected, set_diem_trace};

const TRACE_EVENT: &str = "trace_event";
const TRACE_EDGE: &str = "trace_edge";
const PAGING_SIZE: usize = 10000;

#[derive(Deserialize)]
struct Response {
    #[serde(rename = "_scroll_id")]
    scroll_id: String,
    hits: Hits,
}

#[derive(Deserialize)]
struct Hits {
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Hit {
    #[serde(rename = "_source")]
    source: Source,
}

#[derive(Deserialize)]
struct Source {
    data: serde_json::Value,
    #[serde(rename = "kubernetes.pod_name")]
    pod_name: String,
}

pub struct DiemTraceClient {
    client: Client,
    addr: String,
}

impl DiemTraceClient {
    /// Create DiemTraceClient from a valid socket address.
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        let client = Client::new();
        let addr = format!("http://{}:{}", address.as_ref(), port);

        Self { client, addr }
    }

    pub async fn get_diem_trace(
        &self,
        start_time: DateTime<Utc>,
        duration: Duration,
    ) -> Result<Vec<(String, JsonLogEntry)>> {
        let start = start_time.format("%FT%T%.3fZ").to_string();
        let end = (start_time + duration).format("%FT%T%.3fZ").to_string();
        let response = self
            .client
            .get(&format!("{}/_search?scroll=1m", self.addr))
            .json(&json!(
            {
                "size": PAGING_SIZE,
                "query": {
                    "bool": {
                        "must": [
                            { "term": { "name":   "diem_trace"}}
                        ],
                        "filter": [
                            { "range": { "@timestamp": { "gte": start, "lte": end }}}
                        ]
                    }
                }
            }
            ))
            .send()
            .await?;

        let response: Response = response.json().await?;

        let mut id = response.scroll_id.clone();
        let mut v: Vec<(String, JsonLogEntry)> = Vec::new();
        parse_response(&mut v, response)?;

        loop {
            let response = self
                .client
                .get(&format!("{}/_search/scroll", self.addr))
                .json(&json!(
                {
                    "scroll" : "1m",
                    "scroll_id" : id
                }
                ))
                .send()
                .await?;
            let response: Response = response.json().await?;
            id = response.scroll_id.clone();
            let hits = response.hits.hits.len();
            parse_response(&mut v, response)?;
            if hits < PAGING_SIZE {
                break;
            }
        }
        Ok(v)
    }
}

fn parse_response(v: &mut Vec<(String, JsonLogEntry)>, response: Response) -> Result<()> {
    for item in response.hits.hits {
        let peer = item.source.pod_name;
        let item = item.source.data;
        if let Some(trace_event) = item.get(TRACE_EVENT) {
            v.push((peer, serde_json::from_value(trace_event.clone())?));
        } else if let Some(trace_edge) = item.get(TRACE_EDGE) {
            v.push((peer, serde_json::from_value(trace_edge.clone())?));
        }
    }
    Ok(())
}
