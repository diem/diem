// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use diem_config::config::NodeConfig;
use diem_logger::{info, json_log, Filter, Logger};
use diem_metrics::json_metrics::get_git_rev;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};
use warp::Filter as _;

#[derive(Debug)]
pub struct NodeDebugService {
    runtime: Runtime,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
struct NodeInfo {
    #[serde(default)]
    node_config: NodeConfig,
    #[serde(default)]
    git_revision: String,
}

impl NodeDebugService {
    pub fn new(address: SocketAddr, logger: Option<Arc<Logger>>, node_config: &NodeConfig) -> Self {
        let runtime = Builder::new_multi_thread()
            .thread_name("nodedebug")
            .enable_all()
            .build()
            .expect("[rpc] failed to create runtime");

        // GET /metrics
        let metrics =
            warp::path("metrics").map(|| warp::reply::json(&diem_metrics::get_all_metrics()));

        // GET /events
        let events = warp::path("events").map(|| warp::reply::json(&json_log::pop_last_entries()));

        // Post /log/filter
        let local_filter = {
            let logger = logger.clone();

            warp::path("filter")
                // 16kb should be long enough for a filter
                .and(warp::body::content_length_limit(1024 * 16))
                .and(warp::body::bytes())
                .map(move |bytes: bytes::Bytes| {
                    if let (Some(logger), Ok(filter)) = (&logger, ::std::str::from_utf8(&bytes)) {
                        info!(filter = filter, "Updating local logging filter");
                        logger.set_filter(Filter::builder().parse(filter).build());
                    }

                    warp::reply::reply()
                })
        };

        // Post /log/remote-filter
        let remote_filter = warp::path("remote-filter")
            // 16kb should be long enough for a filter
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::bytes())
            .map(move |bytes: bytes::Bytes| {
                if let (Some(logger), Ok(filter)) = (&logger, ::std::str::from_utf8(&bytes)) {
                    info!(filter = filter, "Updating remote logging filter");
                    logger.set_remote_filter(Filter::builder().parse(filter).build());
                }

                warp::reply::reply()
            });

        // Post /log
        let log = warp::post()
            .and(warp::path("log"))
            .and(local_filter.or(remote_filter));

        // Get /node-info (git revision the node was built at and the node config being used)
        let node_info = NodeInfo {
            git_revision: get_git_rev(),
            node_config: node_config.clone(),
        };
        let node_info_route = warp::path("node-info").map(move || warp::reply::json(&node_info));

        let routes = log.or(warp::get().and(metrics.or(events).or(node_info_route)));

        runtime
            .handle()
            .spawn(async move { warp::serve(routes).bind(address).await });

        Self { runtime }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
}
