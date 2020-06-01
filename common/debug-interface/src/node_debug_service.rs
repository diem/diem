// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use crate::json_log;
use std::net::SocketAddr;
use tokio::runtime::{Builder, Runtime};
use warp::Filter;

#[derive(Debug)]
pub struct NodeDebugService {
    runtime: Runtime,
}

impl NodeDebugService {
    pub fn new(address: SocketAddr) -> Self {
        let runtime = Builder::new()
            .thread_name("nodedebug-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[rpc] failed to create runtime");

        // GET /metrics
        let metrics =
            warp::path("metrics").map(|| warp::reply::json(&libra_metrics::get_all_metrics()));

        // GET /evnets
        let events = warp::path("events").map(|| warp::reply::json(&json_log::pop_last_entries()));

        let routes = warp::get().and(metrics.or(events));

        let server = runtime.enter(move || warp::serve(routes).bind(address));
        runtime.handle().spawn(server);

        Self { runtime }
    }
}
