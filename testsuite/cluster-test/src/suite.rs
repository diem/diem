use crate::{
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidators},
};
use std::cmp::min;

pub struct ExperimentSuite {
    pub experiments: Vec<Box<dyn Experiment>>,
}

impl ExperimentSuite {
    pub fn new_pre_release(cluster: &Cluster) -> Self {
        let mut experiments = vec![];
        let count = min(3, cluster.instances().len() / 3);
        // Reboot different sets of 3 validators *100 times
        for _ in 0..20 {
            let b: Box<dyn Experiment> = Box::new(RebootRandomValidators::new(count, cluster));
            experiments.push(b);
        }
        Self { experiments }
    }
}
