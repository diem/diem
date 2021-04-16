// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stream_rpc::transport::{sse::get_sse_routes, websocket::get_websocket_routes};
use diem_config::config::StreamConfig;
use std::sync::Arc;
use storage_interface::DbReader;
use warp::{filters::BoxedFilter, Filter, Reply};

pub fn get_stream_routes(
    config: &StreamConfig,
    content_length_limit: u64,
    diem_db: Arc<dyn DbReader>,
) -> BoxedFilter<(impl Reply,)> {
    let (wss_routes, _cm) = get_websocket_routes(config, content_length_limit, diem_db.clone());
    let (sse_routes, _cm) = get_sse_routes(config, content_length_limit, diem_db);

    // If streaming rpc isn't enabled, return a 404
    // We do this here because we can't build routes conditionally as if/else types won't match
    let is_enabled = config.enabled;
    warp::any()
        .and_then(move || {
            futures::future::ready(if is_enabled {
                Ok(())
            } else {
                Err(warp::reject::not_found())
            })
        })
        .untuple_one()
        .and(wss_routes.or(sse_routes))
        .boxed()
}
