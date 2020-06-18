// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytes::Bytes;
use hyper::Body;
use libra_logger::prelude::*;
use libradb::backup::BackupHandler;
use serde::Serialize;
use std::{convert::Infallible, future::Future};
use warp::{reply::Response, Rejection, Reply};

pub(super) fn reply_with_lcs_bytes<R: Serialize>(record: &R) -> Result<Box<dyn Reply>> {
    let bytes = lcs::to_bytes(record)?;
    Ok(Box::new(bytes))
}

pub(super) fn reply_with_async_channel_writer<G, F>(
    backup_handler: &BackupHandler,
    get_channel_writer: G,
) -> Result<Box<dyn Reply>>
where
    G: FnOnce(BackupHandler, hyper::body::Sender) -> F,
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, body) = Body::channel();
    let bh = backup_handler.clone();
    tokio::spawn(get_channel_writer(bh, sender));

    Ok(Box::new(Response::new(body)))
}

pub(super) async fn send_size_prefixed_lcs_bytes<I, R>(
    iter_res: Result<I>,
    mut sender: hyper::body::Sender,
) where
    I: Iterator<Item = Result<R>>,
    R: Serialize,
{
    send_size_prefixed_lcs_bytes_impl(iter_res, &mut sender)
        .await
        .unwrap_or_else(|e| {
            warn!("Failed writing to output http body: {:?}", e);
            sender.abort()
        });
}

async fn send_size_prefixed_lcs_bytes_impl<I, R>(
    iter_res: Result<I>,
    sender: &mut hyper::body::Sender,
) -> Result<()>
where
    I: Iterator<Item = Result<R>>,
    R: Serialize,
{
    for record_res in iter_res? {
        let record = record_res?;
        let record_bytes = lcs::to_bytes(&record)?;
        let size_bytes = (record_bytes.len() as u32).to_be_bytes();
        sender.send_data(Bytes::from(size_bytes.to_vec())).await?;
        sender.send_data(Bytes::from(record_bytes)).await?;
    }
    Ok(())
}

/// Return 500 on any error raised by the request handler.
pub(super) fn unwrap_or_500(result: Result<Box<dyn Reply>>) -> Box<dyn Reply> {
    match result {
        Ok(resp) => resp,
        Err(e) => {
            warn!("Request handler exception: {:#}", e);
            Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Return 400 on any rejections (parameter parsing errors).
pub(super) async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    warn!("bad request: {:?}", err);
    Ok(warp::http::StatusCode::BAD_REQUEST)
}
