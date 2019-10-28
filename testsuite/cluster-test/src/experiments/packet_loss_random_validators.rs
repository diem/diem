/// This module provides an experiment which introduces packet loss for
/// a given number of instances in the cluster. It undoes the packet loss
/// in the cluster after the given duration
use crate::{
    cluster::Cluster,
    effects::{Action, PacketLoss, RemoveNetworkEffects},
    experiments::Experiment,
    instance::Instance,
};
use failure;
use rand::Rng;
use std::{collections::HashSet, fmt, thread, time::Duration};

pub struct PacketLossRandomValidators {
    instances: Vec<Instance>,
    percent: f32,
    duration: Duration,
}

impl PacketLossRandomValidators {
    pub fn new(count: usize, percent: f32, duration: Duration, cluster: &Cluster) -> Self {
        if count > cluster.instances().len() {
            panic!(
                "count {} should be <= number of instances {}",
                count,
                cluster.instances().len()
            );
        }
        let mut instances = Vec::with_capacity(count);
        let mut all_instances = cluster.instances().clone();
        let mut rnd = rand::thread_rng();
        for _i in 0..count {
            let instance = all_instances.remove(rnd.gen_range(0, all_instances.len()));
            instances.push(instance);
        }

        Self {
            instances,
            percent,
            duration,
        }
    }
}

impl Experiment for PacketLossRandomValidators {
    fn affected_validators(&self) -> HashSet<String> {
        let mut r = HashSet::new();
        for instance in self.instances.iter() {
            r.insert(instance.short_hash().clone());
        }
        r
    }

    fn run(&self) -> failure::Result<()> {
        let mut instances = vec![];
        for instance in self.instances.iter() {
            let packet_loss = PacketLoss::new(instance.clone(), self.percent);
            packet_loss.apply()?;
            instances.push(packet_loss)
        }
        thread::sleep(self.duration);
        for instance in self.instances.iter() {
            let remove_network_effects = RemoveNetworkEffects::new(instance.clone());
            remove_network_effects.apply()?;
        }
        Ok(())
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
