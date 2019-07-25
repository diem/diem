use crate::{
    cluster::Cluster,
    effects::{Effect, Reboot},
    experiments::Experiment,
    instance::Instance,
};
use failure;
use std::{collections::HashSet, fmt, thread, time::Duration};

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
    fn affected_validators(&self) -> HashSet<String> {
        let mut r = HashSet::new();
        r.insert(self.instance.short_hash().clone());
        r
    }

    fn run(&self) -> failure::Result<()> {
        let reboot = Reboot::new(self.instance.clone());
        reboot.apply()?;
        while !reboot.is_complete() {
            thread::sleep(Duration::from_secs(5));
        }
        Ok(())
    }
}

impl fmt::Display for RebootRandomValidator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reboot {}", self.instance)
    }
}
