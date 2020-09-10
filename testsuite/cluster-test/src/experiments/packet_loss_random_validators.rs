// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

/// This module provides an experiment which introduces packet loss for
/// a given number of instances in the cluster. It undoes the packet loss
/// in the cluster after the given duration
use crate::{
    cluster::Cluster,
    effects::{self, packet_loss::PacketLoss},
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};

use async_trait::async_trait;
use std::{fmt, time::Duration};
use structopt::StructOpt;

pub struct PacketLossRandomValidators {
    instances: Vec<Instance>,
    percent: f32,
    duration: Duration,
}
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct PacketLossRandomValidatorsParams {
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of instances in which packet loss should be introduced"
    )]
    percent_instances: f32,
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of packet loss for each instance"
    )]
    packet_loss_percent: f32,
    #[structopt(
        long,
        default_value = "60",
        help = "Duration in secs for which packet loss happens"
    )]
    duration_secs: u64,
}

impl ExperimentParam for PacketLossRandomValidatorsParams {
    type E = PacketLossRandomValidators;
    fn build(self, cluster: &Cluster) -> Self::E {
        let total_instances = cluster.validator_instances().len();
        let packet_loss_num_instances: usize = std::cmp::min(
            ((self.percent_instances / 100.0) * total_instances as f32).ceil() as usize,
            total_instances,
        );
        let (test_cluster, _) = cluster.split_n_validators_random(packet_loss_num_instances);
        Self::E {
            instances: test_cluster.into_validator_instances(),
            percent: self.packet_loss_percent,
            duration: Duration::from_secs(self.duration_secs),
        }
    }
}

#[async_trait]
impl Experiment for PacketLossRandomValidators {
    async fn run(&mut self, _context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut effects: Vec<_> = self
            .instances
            .clone()
            .into_iter()
            .map(|instance| PacketLoss::new(instance, self.percent))
            .collect();
        effects::activate_all(&mut effects).await?;
        time::delay_for(self.duration).await;
        effects::deactivate_all(&mut effects).await?;
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for PacketLossRandomValidators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet Loss {:.*}% [", 2, self.percent)?;
        for instance in self.instances.iter() {
            write!(f, "{}, ", instance)?;
        }
        write!(f, "]")
    }
}
