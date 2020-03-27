// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

/// PacketLoss introduces a given percentage of PacketLoss for a given instance
use crate::{effects::Action, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use libra_logger::info;
use std::fmt;

pub struct PacketLoss {
    instance: Instance,
    percent: f32,
}

impl PacketLoss {
    pub fn new(instance: Instance, percent: f32) -> Self {
        Self { instance, percent }
    }
}

#[async_trait]
impl Action for PacketLoss {
    async fn apply(&self) -> Result<()> {
        info!("PacketLoss {:.*}% for {}", 2, self.percent, self.instance);
        let cmd = format!(
            "sudo tc qdisc delete dev eth0 root; sudo tc qdisc add dev eth0 root netem loss {:.*}%",
            2, self.percent
        );
        self.instance.run_cmd(vec![cmd]).await
    }
}

impl fmt::Display for PacketLoss {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PacketLoss {:.*}% for {}",
            2, self.percent, self.instance
        )
    }
}
