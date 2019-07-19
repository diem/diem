use cluster_test::{
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidator},
};

pub fn main() {
    let cluster = Cluster::discover().expect("Failed to discover cluster");
    let experiment = RebootRandomValidator::new(&cluster);
    experiment.run().expect("Failed to run experiment");
    println!("OK");
}
