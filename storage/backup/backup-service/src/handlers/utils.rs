// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytes::Bytes;
use diem_logger::prelude::*;
use diem_metrics::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use diemdb::backup::backup_handler::BackupHandler;
use hyper::Body;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{convert::Infallible, future::Future};
use warp::{reply::Response, Rejection, Reply};

pub(super) static LATENCY_HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_backup_service_latency_s",
        "Backup service endpoint latency.",
        &["endpoint", "status"]
    )
    .unwrap()
});

pub(super) static THROUGHPUT_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_backup_service_sent_bytes",
        "Backup service throughput in bytes.",
        &["endpoint"]
    )
    .unwrap()
});

pub(super) fn reply_with_bcs_bytes<R: Serialize>(
    endpoint: &str,
    record: &R,
) -> Result<Box<dyn Reply>> {
    let bytes = bcs::to_bytes(record)?;
    THROUGHPUT_COUNTER
        .with_label_values(&[endpoint])
        .inc_by(bytes.len() as u64);
    Ok(Box::new(bytes))
}

pub(super) struct BytesSender {
    endpoint: &'static str,
    inner: hyper::body::Sender,
}

impl BytesSender {
    fn new(endpoint: &'static str, inner: hyper::body::Sender) -> Self {
        Self { endpoint, inner }
    }

    async fn send_data(&mut self, chunk: Bytes) -> Result<()> {
        let n_bytes = chunk.len();
        self.inner.send_data(chunk).await?;
        THROUGHPUT_COUNTER
            .with_label_values(&[self.endpoint])
            .inc_by(n_bytes as u64);
        Ok(())
    }

    fn abort(self) {
        self.inner.abort()
    }
}

pub(super) fn reply_with_async_channel_writer<G, F>(
    backup_handler: &BackupHandler,
    endpoint: &'static str,
    get_channel_writer: G,
) -> Box<dyn Reply>
where
    G: FnOnce(BackupHandler, BytesSender) -> F,
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, body) = Body::channel();
    let sender = BytesSender::new(endpoint, sender);
    let bh = backup_handler.clone();
    tokio::spawn(get_channel_writer(bh, sender));

    Box::new(Response::new(body))
}

pub(super) async fn send_size_prefixed_bcs_bytes<I, R>(iter_res: Result<I>, mut sender: BytesSender)
where
    I: Iterator<Item = Result<R>>,
    R: Serialize,
{
    send_size_prefixed_bcs_bytes_impl(iter_res, &mut sender)
        .await
        .unwrap_or_else(|e| {
            warn!("Failed writing to output http body: {:?}", e);
            sender.abort()
        });
}

async fn send_size_prefixed_bcs_bytes_impl<I, R>(
    iter_res: Result<I>,
    sender: &mut BytesSender,
) -> Result<()>
where
    I: Iterator<Item = Result<R>>,
    R: Serialize,
{
    for record_res in iter_res? {
        let record = record_res?;
        let record_bytes = bcs::to_bytes(&record)?;
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
