// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod utils;

use crate::handlers::utils::{
    handle_rejection, reply_with_async_channel_writer, reply_with_lcs_bytes,
    send_size_prefixed_lcs_bytes, unwrap_or_500,
};
use anyhow::Result;
use libra_crypto::hash::HashValue;
use libra_types::transaction::Version;
use libradb::backup::BackupHandler;
use warp::{filters::BoxedFilter, reply::Reply, Filter};

fn get_latest_state_root(backup_handler: &BackupHandler) -> Result<Box<dyn Reply>> {
    reply_with_lcs_bytes(&backup_handler.get_latest_state_root()?)
}

fn get_state_range_proof(
    backup_handler: &BackupHandler,
    version: Version,
    end_key: HashValue,
) -> Result<Box<dyn Reply>> {
    reply_with_lcs_bytes(&backup_handler.get_account_state_range_proof(end_key, version)?)
}

fn get_state_snapshot(backup_handler: &BackupHandler, version: Version) -> Result<Box<dyn Reply>> {
    reply_with_async_channel_writer(backup_handler, |bh, sender| {
        send_size_prefixed_lcs_bytes(bh.get_account_iter(version), sender)
    })
}

fn get_state_root_proof(
    backup_handler: &BackupHandler,
    version: Version,
) -> Result<Box<dyn Reply>> {
    reply_with_lcs_bytes(&backup_handler.get_state_root_proof(version)?)
}

pub(crate) fn get_routes(backup_handler: BackupHandler) -> BoxedFilter<(impl Reply,)> {
    // GET latest_state_root
    let bh = backup_handler.clone();
    let latest_state_root = warp::path::end()
        .map(move || get_latest_state_root(&bh))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_range_proof/<version>/<end_key>
    let bh = backup_handler.clone();
    let state_range_proof = warp::path!(Version / HashValue)
        .map(move |version, end_key| get_state_range_proof(&bh, version, end_key))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_snapshot/<version>
    let bh = backup_handler.clone();
    let state_snapshot = warp::path!(Version)
        .map(move |version| get_state_snapshot(&bh, version))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_root_proof/<version>
    let bh = backup_handler;
    let state_root_proof = warp::path!(Version)
        .map(move |version| get_state_root_proof(&bh, version))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // Route by endpoint name.
    let routes = warp::any()
        .and(warp::path("latest_state_root").and(latest_state_root))
        .or(warp::path("state_range_proof").and(state_range_proof))
        .or(warp::path("state_snapshot").and(state_snapshot))
        .or(warp::path("state_root_proof").and(state_root_proof));

    // Serve all routes for GET only.
    warp::get().and(routes).boxed()
}
