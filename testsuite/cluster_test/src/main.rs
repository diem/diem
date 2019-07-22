use clap::{App, Arg, ArgGroup, ArgMatches};
use cluster_test::{
    aws::Aws,
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidator},
    health::AwsLogTail,
};

pub fn main() {
    let matches = arg_matches();
    if matches.is_present(ARG_RUN) {
        let cluster = Cluster::discover().expect("Failed to discover cluster");
        let experiment = RebootRandomValidator::new(&cluster);
        experiment.run().expect("Failed to run experiment");
        println!("Test complete");
    }
    if matches.is_present(ARG_TAIL_LOGS) {
        let workplace = matches
            .value_of(ARG_WORKPLACE)
            .expect("workplace should be set");
        let aws = Aws::new(workplace.into());
        let logs = AwsLogTail::spawn_new(aws.clone()).expect("Failed to start aws log tail");
        for log in logs.event_receiver {
            println!("{:?}", log);
        }
    }
}

const ARG_TAIL_LOGS: &str = "tail-logs";
const ARG_RUN: &str = "run";
const ARG_WORKPLACE: &str = "workplace";

fn arg_matches() -> ArgMatches<'static> {
    let run = Arg::with_name(ARG_RUN).long("--run");
    let workplace = Arg::with_name(ARG_WORKPLACE)
        .long("--workplace")
        .short("-w")
        .takes_value(true);
    let tail_logs = Arg::with_name(ARG_TAIL_LOGS)
        .long("--tail-logs")
        .requires(ARG_WORKPLACE);
    // This grouping means at least one action (tail logs, or run test, or both) is required
    let action_group = ArgGroup::with_name("action")
        .multiple(true)
        .args(&[ARG_TAIL_LOGS, ARG_RUN])
        .required(true);

    App::new("cluster_test")
        .author("Libra Association <opensource@libra.org>")
        .group(action_group)
        .args(&[run, tail_logs, workplace])
        .get_matches()
}
