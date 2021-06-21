// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stream_rpc::transport::util::Transport;
use diem_json_rpc_types::stream::errors::StreamError;
use futures::Stream;
use tokio::sync::mpsc;

pub type StreamSender = mpsc::Sender<Result<String, StreamError>>;

pub type BoxConnectionStream =
    Box<dyn Stream<Item = Result<Option<String>, StreamError>> + Send + Unpin>;

#[derive(Clone, Debug)]
pub struct ConnectionContext {
    pub transport: Transport,
    pub sdk_info: crate::util::SdkInfo,
    pub remote_addr: Option<String>,
}
