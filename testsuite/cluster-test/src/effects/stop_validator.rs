// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

/// StopValidator introduces a rebooting for a given instance
use crate::{effects::Effect, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use diem_logger::debug;
use std::fmt;

pub struct StopValidator {
    instance: Instance,
}

impl StopValidator {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl Effect for StopValidator {
    async fn activate(&mut self) -> Result<()> {
        debug!("Stopping validator {}", self.instance);
        self.instance.stop().await
    }

    async fn deactivate(&mut self) -> Result<()> {
        debug!("Starting validator {}", self.instance);
        self.instance.start().await
    }
}

impl fmt::Display for StopValidator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stop validator {}", self.instance)
    }
}
