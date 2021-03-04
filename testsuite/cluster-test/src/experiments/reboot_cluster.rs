// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{fmt, time::Duration};

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use async_trait::async_trait;
use diem_logger::info;
use futures::future::try_join_all;
use std::{
    collections::HashSet,
    fmt::{Error, Formatter},
};
use structopt::StructOpt;
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct RebootClusterParams {}

pub struct RebootCluster {
    instances: Vec<Instance>,
}

impl ExperimentParam for RebootClusterParams {
    type E = RebootCluster;
    fn build(self, cluster: &Cluster) -> Self::E {
        Self::E {
            instances: <&[instance::Instance]>::clone(&cluster.validator_instances()).to_vec(),
        }
    }
}

#[async_trait]
impl Experiment for RebootCluster {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.instances)
    }

    async fn run(&mut self, _context: &mut Context<'_>) -> anyhow::Result<()> {
        let futures: Vec<_> = self.instances.iter().map(Instance::stop).collect();
        try_join_all(futures).await?;
        for inst in &self.instances {
            info!("Starting node {}", inst.peer_name());
            inst.start().await?;
            time::sleep(Duration::from_secs(10)).await;
        }
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for RebootCluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Reboot cluster")
    }
}
