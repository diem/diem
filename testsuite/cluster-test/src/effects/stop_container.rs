// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Effect, instance::Instance};
use anyhow::Result;
use futures::future::{BoxFuture, FutureExt};
use std::fmt;

pub struct StopContainer {
    instance: Instance,
}

impl StopContainer {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

impl Effect for StopContainer {
    fn activate(&self) -> BoxFuture<Result<()>> {
        self.instance
            .run_cmd(vec!["sudo /usr/sbin/service docker stop"])
            .boxed()
    }

    fn deactivate(&self) -> BoxFuture<Result<()>> {
        self.instance
            .run_cmd(vec![
                "sudo /usr/sbin/service docker start && sudo /usr/sbin/service ecs start",
            ])
            .boxed()
    }
}

impl fmt::Display for StopContainer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stop container {}", self.instance)
    }
}
