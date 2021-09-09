// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    FutureExt, SinkExt, StreamExt,
};
use std::hint;

pub enum Instruction {
    Ok,
    Clear,
}

pub struct ResponseWithInstruction<T> {
    pub resp: T,
    pub instruction: Instruction,
}

impl<T> From<T> for ResponseWithInstruction<T> {
    fn from(val: T) -> Self {
        Self {
            resp: val,
            instruction: Instruction::Ok,
        }
    }
}

#[async_trait]
pub trait StatelessPipeline: Send + Sync {
    type Request;
    type Response;
    async fn process(&self, req: Self::Request) -> ResponseWithInstruction<Self::Response>;
}

pub struct PipelinePhase<T: StatelessPipeline> {
    rx: UnboundedReceiver<T::Request>,
    tx: UnboundedSender<T::Response>,
    processor: Box<T>,
}

impl<T: StatelessPipeline> PipelinePhase<T> {
    pub fn new(
        rx: UnboundedReceiver<T::Request>,
        tx: UnboundedSender<T::Response>,
        processor: Box<T>,
    ) -> Self {
        Self { rx, tx, processor }
    }

    pub fn exhaust_requests_non_blocking(&mut self) {
        while self.rx.next().now_or_never().is_some() {
            hint::spin_loop()
        }
    }

    pub async fn start(mut self) {
        // main loop
        while let Some(req) = self.rx.next().await {
            let ResponseWithInstruction { resp, instruction } = self.processor.process(req).await;
            match instruction {
                Instruction::Ok => {}
                Instruction::Clear => self.exhaust_requests_non_blocking(),
            }
            if self.tx.send(resp).await.is_err() {
                break;
            }
        }
    }
}
