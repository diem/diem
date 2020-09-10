// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use libra_logger::{info, json_log, Filter, Logger};
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};
use warp::Filter as _;

#[derive(Debug)]
pub struct NodeDebugService {
    runtime: Runtime,
}

impl NodeDebugService {
    pub fn new(address: SocketAddr, logger: Option<Arc<Logger>>) -> Self {
        let runtime = Builder::new()
            .thread_name("nodedebug-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[rpc] failed to create runtime");

        // GET /metrics
        let metrics =
            warp::path("metrics").map(|| warp::reply::json(&libra_metrics::get_all_metrics()));

        // GET /events
        let events = warp::path("events").map(|| warp::reply::json(&json_log::pop_last_entries()));

        // Post /log
        let log = warp::post()
            .and(warp::path("log"))
            // 16kb should be long enough for a filter
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::bytes())
            .map(move |bytes: bytes::Bytes| {
                if let (Some(logger), Ok(filter)) = (&logger, ::std::str::from_utf8(&bytes)) {
                    info!(filter = filter, "Updating logging filter");
                    logger.set_filter(Filter::builder().parse(filter).build());
                }

                warp::reply::reply()
            });

        let routes = log.or(warp::get().and(metrics.or(events)));

        let server = runtime.enter(move || warp::serve(routes).bind(address));
        runtime.handle().spawn(server);

        Self { runtime }
    }
}
