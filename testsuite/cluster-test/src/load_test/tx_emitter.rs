// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;

use crate::load_test::Handler;
use crate::tx_emitter::EmitJob;
use crate::{experiments::Context, tx_emitter::EmitJobRequest};
use async_trait::async_trait;
use std::fmt;
use std::time::Duration;

pub struct TxEmitter<'a> {
    context: Context<'a>,
    job: Option<EmitJob>,
}

impl TxEmitter<'_> {
    pub fn new(context: Context<'static>) -> Self {
        Self { context, job: None }
    }
}

#[async_trait]
impl Handler for TxEmitter<'_> {
    async fn start(&mut self, _duration: Duration) -> Result<()> {
        self.job = Some(
            self.context
                .tx_emitter
                .start_job(EmitJobRequest::for_instances(
                    self.context.cluster.fullnode_instances().to_vec(),
                    self.context.global_emit_job_request,
                    0,
                ))
                .await?,
        );

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let job = self.job.take().expect("");
        self.context.tx_emitter.stop_job(job).await;
        Ok(())
    }
}

impl fmt::Display for TxEmitter<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
