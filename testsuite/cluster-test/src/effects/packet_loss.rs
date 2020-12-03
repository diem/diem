// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

/// PacketLoss introduces a given percentage of PacketLoss for a given instance
use crate::{effects::Effect, instance::Instance};
use anyhow::Result;

use async_trait::async_trait;
use diem_logger::info;
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
impl Effect for PacketLoss {
    async fn activate(&mut self) -> Result<()> {
        info!("PacketLoss {:.*}% for {}", 2, self.percent, self.instance);
        let cmd = format!(
            "tc qdisc add dev eth0 root netem loss {:.*}%",
            2, self.percent
        );
        self.instance.util_cmd(cmd, "ac-packet-loss").await
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("PacketLoss {:.*}% for {}", 2, self.percent, self.instance);
        let cmd = "tc qdisc delete dev eth0 root; true".to_string();
        self.instance.util_cmd(cmd, "de-packet-loss").await
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
