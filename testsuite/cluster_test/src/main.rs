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
use termion::{color, style};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

pub fn main() {
    let matches = arg_matches();

    let workplace = matches
        .value_of(ARG_WORKPLACE)
        .expect("workplace should be set");
    let aws = Aws::new(workplace.into());
    let logs = AwsLogTail::spawn_new(aws.clone()).expect("Failed to start aws log tail");
    println!("Aws log thread started");
    let cluster = Cluster::discover(&aws).expect("Failed to discover cluster");
    println!("Discovered {} peers", cluster.instances().len());

    if matches.is_present(ARG_RUN) {
        let mut health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        let events = logs.recv_all();
        if !health_check_runner.run(&events).is_empty() {
            panic!("Some validators are unhealthy before experiment started");
        }

        let experiment = RebootRandomValidator::new(&cluster);
        println!(
            "{}Starting experiment {}{}{}{}",
            style::Bold,
            color::Fg(color::Blue),
            experiment,
            color::Fg(color::Reset),
            style::Reset
        );
        let affected_validators = experiment.affected_validators();
        let (exp_result_sender, exp_result_recv) = mpsc::channel();
        thread::spawn(move || {
            let result = experiment.run();
            exp_result_sender
                .send(result)
                .expect("Failed to send experiment result");
        });

        // We expect experiments completes and cluster go into healthy state within 2 minutes
        let experiment_deadline = Instant::now() + Duration::from_secs(2 * 60);

        loop {
            if Instant::now() > experiment_deadline {
                panic!("Experiment did not complete in time");
            }
            let deadline = Instant::now() + HEALTH_POLL_INTERVAL;
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

        println!(
            "{}Experiment finished, waiting until all affected validators recover{}",
            style::Bold,
            style::Reset
        );

        loop {
            if Instant::now() > experiment_deadline {
                panic!("Cluster did not become healthy in time");
            }
            let deadline = Instant::now() + HEALTH_POLL_INTERVAL;
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
        }

        println!("Experiment completed");
    }
    if matches.is_present(ARG_TAIL_LOGS) {
        for log in logs.event_receiver {
            println!("{:?}", log);
        }
    } else if matches.is_present(HEALTH_CHECK) {
        let mut health_check_runner = HealthCheckRunner::new_all(cluster.clone());
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
