// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stream_rpc::transport::util::Transport;
use futures::Stream;
use tokio::sync::mpsc;

pub type StreamSender = mpsc::Sender<Result<String, anyhow::Error>>;

pub type BoxConnectionStream =
    Box<dyn Stream<Item = Result<Option<String>, anyhow::Error>> + Send + Unpin>;

pub type Task = tokio::task::JoinHandle<()>;

#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub transport: Transport,
    pub sdk_info: crate::util::SdkInfo,
    pub remote_addr: Option<String>,
}
