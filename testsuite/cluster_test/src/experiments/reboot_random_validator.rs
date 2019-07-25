use crate::{
    cluster::Cluster,
    effects::{Effect, Reboot},
    experiments::Experiment,
    instance::Instance,
};
use failure;
use std::{thread, time::Duration};

pub struct RebootRandomValidator {
    instance: Instance,
}

impl RebootRandomValidator {
    pub fn new(cluster: &Cluster) -> Self {
        Self {
            instance: cluster.random_instance(),
        }
    }
}

impl Experiment for RebootRandomValidator {
    fn run(&self) -> failure::Result<()> {
        let reboot = Reboot::new(self.instance.clone());
        reboot.apply()?;
        while !reboot.is_complete() {
            thread::sleep(Duration::from_secs(5));
        }
        Ok(())
    }
}
