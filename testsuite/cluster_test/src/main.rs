use clap::{App, Arg, ArgGroup, ArgMatches};
use cluster_test::{
    aws::Aws,
    cluster::Cluster,
    experiments::{Experiment, RebootRandomValidator},
    health::{AwsLogTail, HealthCheckRunner},
};
use std::{
    collections::HashSet,
    env,
    sync::mpsc::{self, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use termion::{color, style};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(1);

pub fn main() {
    let matches = arg_matches();
    let mut runner = ClusterTestRunner::setup(&matches);

    if matches.is_present(ARG_RUN) {
        runner.run_experiments_in_loop();
    } else if matches.is_present(ARG_RUN_ONCE) {
        runner.run_single_experiment();
    } else if matches.is_present(ARG_TAIL_LOGS) {
        runner.tail_logs();
    } else if matches.is_present(ARG_HEALTH_CHECK) {
        runner.run_health_check();
    }
}

struct ClusterTestRunner {
    logs: AwsLogTail,
    cluster: Cluster,
    health_check_runner: HealthCheckRunner,
}

impl ClusterTestRunner {
    /// Discovers cluster, setup log, etc
    pub fn setup(matches: &ArgMatches) -> Self {
        let workplace = matches
            .value_of(ARG_WORKPLACE)
            .expect("workplace should be set");
        let aws = Aws::new(workplace.into());
        let cluster = Cluster::discover(&aws).expect("Failed to discover cluster");
        println!("Discovered {} peers", cluster.instances().len());
        let log_tail_started = Instant::now();
        let logs =
            AwsLogTail::spawn_new(aws.clone(), &cluster).expect("Failed to start aws log tail");
        let log_tail_startup_time = Instant::now() - log_tail_started;
        println!(
            "Aws log thread started in {} ms",
            log_tail_startup_time.as_millis()
        );
        let health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        Self {
            logs,
            cluster,
            health_check_runner,
        }
    }

    /// Run experiments every EXPERIMENT_INTERVAL seconds until fails
    pub fn run_experiments_in_loop(&mut self) {
        let experiment_interval_sec = match env::var("EXPERIMENT_INTERVAL") {
            Ok(s) => s.parse().expect("EXPERIMENT_INTERVAL env is not a number"),
            Err(..) => 15,
        };
        let experiment_interval = Duration::from_secs(experiment_interval_sec);
        loop {
            self.run_single_experiment();
            thread::sleep(experiment_interval);
        }
    }

    pub fn run_single_experiment(&mut self) {
        let events = self.logs.recv_all();
        if !self.health_check_runner.run(&events).is_empty() {
            panic!("Some validators are unhealthy before experiment started");
        }

        let experiment = RebootRandomValidator::new(&self.cluster);
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
            let events = self.logs.recv_all_until_deadline(deadline);
            let failed_validators = self.health_check_runner.run(&events);
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
            let events = self.logs.recv_all_until_deadline(deadline);
            let failed_validators = self.health_check_runner.run(&events);
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

    fn run_health_check(&mut self) {
        loop {
            let deadline = Instant::now() + Duration::from_secs(1);
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = self.logs.recv_all_until_deadline(deadline);
            self.health_check_runner.run(&events);
        }
    }

    fn tail_logs(self) {
        for log in self.logs.event_receiver {
            println!("{:?}", log);
        }
    }
}

const ARG_WORKPLACE: &str = "workplace";
// Actions:
const ARG_TAIL_LOGS: &str = "tail-logs";
const ARG_HEALTH_CHECK: &str = "health-check";
const ARG_RUN: &str = "run";
const ARG_RUN_ONCE: &str = "run-once";

fn arg_matches() -> ArgMatches<'static> {
    let run = Arg::with_name(ARG_RUN).long("--run");
    let run_once = Arg::with_name(ARG_RUN_ONCE).long("--run-once");
    let workplace = Arg::with_name(ARG_WORKPLACE)
        .long("--workplace")
        .short("-w")
        .takes_value(true)
        .required(true);
    let tail_logs = Arg::with_name(ARG_TAIL_LOGS).long("--tail-logs");
    let health_check = Arg::with_name(ARG_HEALTH_CHECK).long("--health-check");
    // This grouping requires one and only one action (tail logs, run test, etc)
    let action_group = ArgGroup::with_name("action")
        .args(&[ARG_TAIL_LOGS, ARG_RUN, ARG_RUN_ONCE, ARG_HEALTH_CHECK])
        .required(true);

    App::new("cluster_test")
        .author("Libra Association <opensource@libra.org>")
        .group(action_group)
        .args(&[workplace, run, run_once, tail_logs, health_check])
        .get_matches()
}
