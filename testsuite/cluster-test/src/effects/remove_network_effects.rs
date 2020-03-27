// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

/// RemoveNetworkEffect deletes all network effects introduced on an instance
use crate::{effects::Action, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use libra_logger::debug;
use std::fmt;

pub struct RemoveNetworkEffects {
    instance: Instance,
}

impl RemoveNetworkEffects {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl Action for RemoveNetworkEffects {
    async fn apply(&self) -> Result<()> {
        debug!("RemoveNetworkEffects for {}", self.instance);
        self.instance
            .run_cmd(vec!["sudo tc qdisc delete dev eth0 root; true".to_string()])
            .await
    }
}

impl fmt::Display for RemoveNetworkEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RemoveNetworkEffects for {}", self.instance)
    }
}
