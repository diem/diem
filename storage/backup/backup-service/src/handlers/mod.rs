// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod utils;

use crate::handlers::utils::{
    handle_rejection, reply_with_async_channel_writer, reply_with_bcs_bytes,
    send_size_prefixed_bcs_bytes, unwrap_or_500, LATENCY_HISTOGRAM,
};
use diem_crypto::hash::HashValue;
use diem_types::transaction::Version;
use diemdb::backup::backup_handler::BackupHandler;
use warp::{filters::BoxedFilter, reply::Reply, Filter};

static DB_STATE: &str = "db_state";
static STATE_RANGE_PROOF: &str = "state_range_proof";
static STATE_SNAPSHOT: &str = "state_snapshot";
static STATE_ROOT_PROOF: &str = "state_root_proof";
static EPOCH_ENDING_LEDGER_INFOS: &str = "epoch_ending_ledger_infos";
static TRANSACTIONS: &str = "transactions";
static TRANSACTION_RANGE_PROOF: &str = "transaction_range_proof";

pub(crate) fn get_routes(backup_handler: BackupHandler) -> BoxedFilter<(impl Reply,)> {
    // GET db_state
    let bh = backup_handler.clone();
    let db_state = warp::path::end()
        .map(move || reply_with_bcs_bytes(DB_STATE, &bh.get_db_state()?))
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_range_proof/<version>/<end_key>
    let bh = backup_handler.clone();
    let state_range_proof = warp::path!(Version / HashValue)
        .map(move |version, end_key| {
            reply_with_bcs_bytes(
                STATE_RANGE_PROOF,
                &bh.get_account_state_range_proof(end_key, version)?,
            )
        })
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET state_snapshot/<version>
    let bh = backup_handler.clone();
    let state_snapshot = warp::path!(Version)
        .map(move |version| {
            reply_with_async_channel_writer(&bh, STATE_SNAPSHOT, |bh, sender| {
                send_size_prefixed_bcs_bytes(bh.get_account_iter(version), sender)
            })
        })
        .recover(handle_rejection);

    // GET state_root_proof/<version>
    let bh = backup_handler.clone();
    let state_root_proof = warp::path!(Version)
        .map(move |version| {
            reply_with_bcs_bytes(STATE_ROOT_PROOF, &bh.get_state_root_proof(version)?)
        })
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // GET epoch_ending_ledger_infos/<start_epoch>/<end_epoch>/
    let bh = backup_handler.clone();
    let epoch_ending_ledger_infos = warp::path!(u64 / u64)
        .map(move |start_epoch, end_epoch| {
            // use async move block to group `bh` and the iterator into the same lifetime, since the
            // latter references the former.
            reply_with_async_channel_writer(
                &bh,
                EPOCH_ENDING_LEDGER_INFOS,
                |bh, sender| async move {
                    send_size_prefixed_bcs_bytes(
                        bh.get_epoch_ending_ledger_info_iter(start_epoch, end_epoch),
                        sender,
                    )
                    .await
                },
            )
        })
        .recover(handle_rejection);

    // GET transactions/<start_version>/<num_transactions>
    let bh = backup_handler.clone();
    let transactions = warp::path!(Version / usize)
        .map(move |start_version, num_transactions| {
            // use async move block to group `bh` and the iterator into the same lifetime, since the
            // latter references the former.
            reply_with_async_channel_writer(&bh, TRANSACTIONS, |bh, sender| async move {
                send_size_prefixed_bcs_bytes(
                    bh.get_transaction_iter(start_version, num_transactions),
                    sender,
                )
                .await
            })
        })
        .recover(handle_rejection);

    // GET transaction_range_proof/<first_version>/<last_version>
    let bh = backup_handler;
    let transaction_range_proof = warp::path!(Version / Version)
        .map(move |first_version, last_version| {
            reply_with_bcs_bytes(
                TRANSACTION_RANGE_PROOF,
                &bh.get_transaction_range_proof(first_version, last_version)?,
            )
        })
        .map(unwrap_or_500)
        .recover(handle_rejection);

    // Route by endpoint name.
    let routes = warp::any()
        .and(warp::path(DB_STATE).and(db_state))
        .or(warp::path(STATE_RANGE_PROOF).and(state_range_proof))
        .or(warp::path(STATE_SNAPSHOT).and(state_snapshot))
        .or(warp::path(STATE_ROOT_PROOF).and(state_root_proof))
        .or(warp::path(EPOCH_ENDING_LEDGER_INFOS).and(epoch_ending_ledger_infos))
        .or(warp::path(TRANSACTIONS).and(transactions))
        .or(warp::path(TRANSACTION_RANGE_PROOF).and(transaction_range_proof));

    // Serve all routes for GET only.
    warp::get()
        .and(routes)
        .with(warp::log::custom(|info| {
            let endpoint = info.path().split('/').nth(1).unwrap_or("-");
            LATENCY_HISTOGRAM
                .with_label_values(&[endpoint, info.status().as_str()])
                .observe(info.elapsed().as_secs_f64())
        }))
        .boxed()
}
