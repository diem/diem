// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{fmt, time::Duration};

use structopt::StructOpt;

use failure;
use futures::future::{BoxFuture, FutureExt};

use crate::experiments::Context;
/// This module provides an experiment which introduces packet loss for
/// a given number of instances in the cluster. It undoes the packet loss
/// in the cluster after the given duration
use crate::{
    cluster::Cluster,
    effects::{Action, PacketLoss, RemoveNetworkEffects},
    experiments::Experiment,
    instance::Instance,
};

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

impl PacketLossRandomValidators {
    pub fn new(params: PacketLossRandomValidatorsParams, cluster: &Cluster) -> Self {
        let total_instances = cluster.instances().len();
        let packet_loss_num_instances: usize = std::cmp::min(
            ((params.percent_instances / 100.0) * total_instances as f32).ceil() as usize,
            total_instances,
        );
        let (test_cluster, _) = cluster.split_n_random(packet_loss_num_instances);
        Self {
            instances: test_cluster.into_instances(),
            percent: params.packet_loss_percent,
            duration: Duration::from_secs(params.duration_secs),
        }
    }
}

impl Experiment for PacketLossRandomValidators {
    fn run(&mut self, _context: &mut Context) -> BoxFuture<failure::Result<Option<String>>> {
        async move {
            let mut instances = vec![];
            for instance in self.instances.iter() {
                let packet_loss = PacketLoss::new(instance.clone(), self.percent);
                packet_loss.apply().await?;
                instances.push(packet_loss)
            }
            time::delay_for(self.duration).await;
            for instance in self.instances.iter() {
                let remove_network_effects = RemoveNetworkEffects::new(instance.clone());
                remove_network_effects.apply().await?;
            }
            Ok(None)
        }
            .boxed()
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
