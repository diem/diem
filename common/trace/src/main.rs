// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, NaiveDateTime, Utc};
use debug_interface::{
    libra_trace::{random_node, trace_node},
    trace::LibraTraceClient,
};
use libra_logger::{info, Logger};
use serde_json::Value;
use std::env;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Libra Trace")]
struct Args {
    #[structopt(long, help = "Hostname of elastic search backend")]
    host: String,

    #[structopt(long, help = "Port of elastic search backend", default_value = "9200")]
    port: u16,

    #[structopt(
        long,
        help = "Start time to retrieve libra traces, format as yyyy-MM-ddTHH:mm:ss in UTC"
    )]
    start: String,

    #[structopt(
        long,
        help = "Time to retrieve libra traces for in seconds",
        default_value = "5"
    )]
    duration: i64,
}

#[tokio::main]
pub async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    Logger::new().is_async(true).init();

    let args = Args::from_args();

    let start = DateTime::<Utc>::from_utc(
        NaiveDateTime::parse_from_str(&args.start, "%FT%T").expect("Failed to parse start time."),
        Utc,
    );
    let duration = chrono::Duration::seconds(args.duration);
    let libra_trace_client = LibraTraceClient::new(args.host, args.port);
    let trace = match libra_trace_client.get_libra_trace(start, duration).await {
        Ok(trace) => Some(trace),
        Err(err) => {
            info!("Failed to capture traces from elastic search {}", err);
            None
        }
    };

    if let Some(trace) = trace {
        info!("Traced {} events", trace.len());
        let mut events = vec![];
        for (node, mut event) in trace {
            // This could be done more elegantly, but for now this will do
            event
                .json
                .as_object_mut()
                .unwrap()
                .insert("peer".to_string(), Value::String(node));
            events.push(event);
        }
        events.sort_by_key(|k| k.timestamp);
        let node =
            random_node(&events[..], "json-rpc::submit", "txn::").expect("No trace node found");
        info!("Tracing {}", node);
        trace_node(&events[..], &node);
    }
}
