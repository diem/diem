use clap::{App, Arg, ArgGroup, ArgMatches};
use cluster_test::{
    aws::Aws,
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidator},
    health::{AwsLogTail, HealthCheckRunner},
};
use std::{
    collections::HashSet,
    sync::mpsc::{self, TryRecvError},
    thread,
    time::{Duration, Instant},
};

pub fn main() {
    let matches = arg_matches();

    let workplace = matches
        .value_of(ARG_WORKPLACE)
        .expect("workplace should be set");
    let aws = Aws::new(workplace.into());
    let logs = AwsLogTail::spawn_new(aws.clone()).expect("Failed to start aws log tail");
    let cluster = Cluster::discover().expect("Failed to discover cluster");
    println!("Waiting little bit before starting...");
    thread::sleep(Duration::from_secs(10));
    println!("Done");

    if matches.is_present(ARG_RUN) {
        let mut health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        let events = logs.recv_all();
        if !health_check_runner.run(&events).is_empty() {
            panic!("Some validators are unhealthy before experiment started");
        }

        let experiment = RebootRandomValidator::new(&cluster);
        println!("Starting experiment {}", experiment);
        let mut affected_validators = experiment.affected_validators();
        let (exp_result_sender, exp_result_recv) = mpsc::channel();
        thread::spawn(move || {
            let result = experiment.run();
            exp_result_sender
                .send(result)
                .expect("Failed to send experiment result");
        });
        loop {
            let deadline = Instant::now() + Duration::from_secs(1);
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = logs.recv_all_until_deadline(deadline);
            let failed_validators = health_check_runner.run(&events);
            for failed in failed_validators {
                if !affected_validators.contains(&failed) {
                    panic!(
                        "Validator {} failed, not expected for this experiment",
                        failed
                    );
                }
            }
            match exp_result_recv.try_recv() {
                Ok(result) => {
                    result.expect("Failed to run experiment");
                    break;
                }
                Err(TryRecvError::Empty) => {
                    // Experiment in progress, continue monitoring health
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Experiment thread exited without returning result");
                }
            }
        }

        println!();
        println!("**Experiment finished, waiting until all affected validators recover**");
        println!();

        loop {
            let deadline = Instant::now() + Duration::from_secs(1);
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = logs.recv_all_until_deadline(deadline);
            let failed_validators = health_check_runner.run(&events);
            let mut still_affected_validator = HashSet::new();
            for failed in failed_validators {
                if !affected_validators.contains(&failed) {
                    panic!(
                        "Validator {} failed, not expected for this experiment",
                        failed
                    );
                }
                still_affected_validator.insert(failed);
            }
            if still_affected_validator.is_empty() {
                break;
            }
            affected_validators = still_affected_validator;
        }

        println!("Experiment completed");
    }
    if matches.is_present(ARG_TAIL_LOGS) {
        for log in logs.event_receiver {
            println!("{:?}", log);
        }
    } else if matches.is_present(HEALTH_CHECK) {
        let mut health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        // Initial sleep to collect events
        // Otherwise liveness check will fail because it did not yet receive events from validators
        loop {
            let deadline = Instant::now() + Duration::from_secs(1);
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = logs.recv_all_until_deadline(deadline);
            health_check_runner.run(&events);
        }
    }
}

const ARG_TAIL_LOGS: &str = "tail-logs";
const HEALTH_CHECK: &str = "health-check";
const ARG_RUN: &str = "run";
const ARG_WORKPLACE: &str = "workplace";

fn arg_matches() -> ArgMatches<'static> {
    let run = Arg::with_name(ARG_RUN).long("--run");
    let workplace = Arg::with_name(ARG_WORKPLACE)
        .long("--workplace")
        .short("-w")
        .takes_value(true)
        .required(true);
    let tail_logs = Arg::with_name(ARG_TAIL_LOGS).long("--tail-logs");
    let health_check = Arg::with_name(HEALTH_CHECK)
        .long("--health-check")
        .conflicts_with(ARG_TAIL_LOGS);
    // This grouping means at least one action (tail logs, or run test, or both) is required
    let action_group = ArgGroup::with_name("action")
        .multiple(true)
        .args(&[ARG_TAIL_LOGS, ARG_RUN, HEALTH_CHECK])
        .required(true);

    App::new("cluster_test")
        .author("Libra Association <opensource@libra.org>")
        .group(action_group)
        .args(&[workplace, run, tail_logs, health_check])
        .get_matches()
}
