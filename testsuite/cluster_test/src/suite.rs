use crate::{
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidators},
};

pub struct ExperimentSuite {
    pub experiments: Vec<Box<dyn Experiment>>,
}

impl ExperimentSuite {
    pub fn new_pre_release(cluster: &Cluster) -> Self {
        let mut experiments = vec![];
        // Reboot different sets of 3 validators *100 times
        for _ in 0..100 {
            let b: Box<dyn Experiment> = Box::new(RebootRandomValidators::new(3, cluster));
            experiments.push(b);
        }
        Self { experiments }
    }
}
