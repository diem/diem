// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use async_trait::async_trait;
use futures::future::try_join_all;
use std::fmt::Display;
use std::time::Duration;

mod mempool;
mod state_sync;
mod tx_emitter;

#[async_trait]
pub trait Handler: Display + Send {
    async fn start(&mut self, duration: Duration) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
}

pub struct HandlerRunner {
    handlers: Vec<Box<dyn Handler>>,
}

impl HandlerRunner {
    pub fn new(handlers: Vec<Box<dyn Handler>>) -> Self {
        Self { handlers }
    }

    pub async fn start_all<T: Handler>(handler: &mut Vec<T>, duration: Duration) -> Result<()> {
        try_join_all(handler.iter_mut().map(|h| h.start(duration))).await?;
        Ok(())
    }

    pub async fn stop_all<T: Handler>(handler: &mut Vec<T>) -> Result<()> {
        try_join_all(handler.iter_mut().map(Handler::stop)).await?;
        Ok(())
    }
}
