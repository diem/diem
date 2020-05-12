// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use libra_crypto::hash::HashValue;
use libra_logger::prelude::*;
use libra_types::transaction::Version;
use libradb::backup::BackupHandler;
use std::convert::Infallible;
use warp::{filters::BoxedFilter, reply::Reply, Filter, Rejection};

fn get_state_range_proof(
    backup_handler: &BackupHandler,
    version: Version,
    end_key: HashValue,
) -> Result<Box<dyn Reply>> {
    let bytes = lcs::to_bytes(&backup_handler.get_account_state_range_proof(end_key, version)?)?;
    Ok(Box::new(bytes))
}

/// Return 500 on any error raised by the request handler.
fn get_state_snapshot(
    _backup_handler: &BackupHandler,
    _version: Version,
) -> Result<Box<dyn Reply>> {
    bail!("unimplemented.")
}

/// Return 500 on any error raised by the request handler.
fn unwrap_or_500(result: Result<Box<dyn Reply>>) -> Box<dyn Reply> {
    match result {
        Ok(resp) => resp,
        Err(e) => {
            warn!("Request handler exception: {:#}", e);
            Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Return 400 on any rejections (parameter parsing errors).
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    warn!("bad request: {:?}", err);
    Ok(warp::http::StatusCode::BAD_REQUEST)
}

pub(crate) fn get_routes(backup_handler: BackupHandler) -> BoxedFilter<(impl Reply,)> {
    // GET state_range_proof/<version>/<end_key>
    let bh = backup_handler.clone();
    let state_range_proof = warp::path!(Version / HashValue)
        .map(move |version, end_key| get_state_range_proof(&bh, version, end_key))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_snapshot/<version>
    let bh = backup_handler;
    let state_snapshot = warp::path!(Version)
        .map(move |version| get_state_snapshot(&bh, version))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // Route by endpoint name.
    let routes = warp::any()
        .and(warp::path("state_range_proof").and(state_range_proof))
        .or(warp::path("state_snapshot").and(state_snapshot));

    // Serve all routes for GET only.
    warp::get().and(routes).boxed()
}
