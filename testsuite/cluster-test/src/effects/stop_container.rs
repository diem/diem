// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Effect, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use std::fmt;

pub struct StopContainer {
    instance: Instance,
}

impl StopContainer {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl Effect for StopContainer {
    async fn activate(&self) -> Result<()> {
        self.instance
            .run_cmd(vec!["sudo /usr/sbin/service docker stop"])
            .await
    }

    async fn deactivate(&self) -> Result<()> {
        self.instance
            .run_cmd(vec![
                "sudo /usr/sbin/service docker start && sudo /usr/sbin/service ecs start",
            ])
            .await
    }
}

impl fmt::Display for StopContainer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stop container {}", self.instance)
    }
}
