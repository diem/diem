use clap::{App, Arg, ArgGroup, ArgMatches};
use failure::{
    self,
    prelude::{bail, format_err},
};
use libra_cluster_test::{
    aws::Aws,
    cluster::Cluster,
    deployment::{DeploymentManager, SOURCE_TAG, TESTED_TAG},
    effects::{Action, Effect, Reboot, StopContainer},
    experiments::{Experiment, RebootRandomValidators},
    health::{DebugPortLogThread, HealthCheckRunner, LogTail},
    log_prune::LogPruner,
    slack::SlackClient,
    suite::ExperimentSuite,
    tx_emitter::TxEmitter,
};
use slog::{o, Drain};
use slog_scope::info;
use std::{
    collections::HashSet,
    env, mem,
    sync::mpsc::{self, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use termion::{color, style};
use threadpool;

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

pub fn main() {
    setup_log();
    let matches = arg_matches();

    if matches.is_present(ARG_PRUNE) {
        let util = ClusterUtil::setup(&matches);
        util.prune_logs();
        return;
    } else if matches.is_present(ARG_EMIT_TX) {
        let util = ClusterUtil::setup(&matches);
        util.emit_tx();
        return;
    }

    let mut runner = ClusterTestRunner::setup(&matches);

    if matches.is_present(ARG_RUN) {
        runner.run_suite_in_loop();
    } else if matches.is_present(ARG_RUN_ONCE) {
        let experiment = RebootRandomValidators::new(3, &runner.cluster);
        runner.run_single_experiment(Box::new(experiment)).unwrap();
    } else if matches.is_present(ARG_TAIL_LOGS) {
        runner.tail_logs();
    } else if matches.is_present(ARG_HEALTH_CHECK) {
        runner.run_health_check();
    } else if matches.is_present(ARG_WIPE_ALL_DB) {
        runner.stop();
        runner.wipe_all_db(true);
        runner.start();
    } else if matches.is_present(ARG_REBOOT) {
        runner.reboot();
    } else if matches.is_present(ARG_RESTART) {
        runner.restart();
    } else if matches.is_present(ARG_STOP) {
        runner.stop();
    } else if matches.is_present(ARG_START) {
        runner.start();
    }
}

fn setup_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    let logger_guard = slog_scope::set_global_logger(logger);
    std::mem::forget(logger_guard);
}

struct ClusterUtil {
    cluster: Cluster,
    aws: Aws,
}

struct ClusterTestRunner {
    logs: LogTail,
    cluster: Cluster,
    health_check_runner: HealthCheckRunner,
    deployment_manager: DeploymentManager,
    experiment_interval: Duration,
    slack: Option<SlackClient>,
    thread_pool: threadpool::ThreadPool,
}

impl ClusterUtil {
    pub fn setup(matches: &ArgMatches) -> Self {
        let workplace = matches
            .value_of(ARG_WORKPLACE)
            .expect("workplace should be set");
        let aws = Aws::new(workplace.into());
        let peers = matches.values_of_lossy(ARG_PEERS);
        let cluster = Cluster::discover(&aws).expect("Failed to discover cluster");
        let cluster = match peers {
            None => cluster,
            Some(peers) => cluster.sub_cluster(peers),
        };
        info!("Discovered {} peers", cluster.instances().len());
        Self { cluster, aws }
    }

    pub fn prune_logs(&self) {
        let log_prune = LogPruner::new(self.aws.clone());
        log_prune.prune_logs();
    }

    pub fn emit_tx(self) {
        let emitter = TxEmitter::new(&self.cluster);
        emitter.run();
    }
}

impl ClusterTestRunner {
    /// Discovers cluster, setup log, etc
    pub fn setup(matches: &ArgMatches) -> Self {
        let util = ClusterUtil::setup(matches);
        let cluster = util.cluster;
        let aws = util.aws;
        let log_tail_started = Instant::now();
        let logs = DebugPortLogThread::spawn_new(&cluster);
        let log_tail_startup_time = Instant::now() - log_tail_started;
        info!(
            "Log tail thread started in {} ms",
            log_tail_startup_time.as_millis()
        );
        let health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        let experiment_interval_sec = match env::var("EXPERIMENT_INTERVAL") {
            Ok(s) => s.parse().expect("EXPERIMENT_INTERVAL env is not a number"),
            Err(..) => 15,
        };
        let experiment_interval = Duration::from_secs(experiment_interval_sec);
        let deployment_manager = DeploymentManager::new(aws.clone(), cluster.clone());
        let slack = SlackClient::try_new_from_environment();
        let thread_pool = threadpool::Builder::new()
            .num_threads(10)
            .thread_name("ssh-pool".to_string())
            .build();
        Self {
            logs,
            cluster,
            health_check_runner,
            deployment_manager,
            experiment_interval,
            slack,
            thread_pool,
        }
    }

    pub fn run_suite_in_loop(&mut self) {
        let mut hash_to_tag = None;
        loop {
            if let Some(hash) = self.deployment_manager.latest_hash_changed() {
                info!(
                    "New version of `{}` tag is available: `{}`",
                    SOURCE_TAG, hash
                );
                match self.redeploy(hash.clone()) {
                    Err(e) => {
                        self.report_failure(format!("Failed to deploy `{}`: {}", hash, e));
                        return;
                    }
                    Ok(true) => {
                        self.slack_message(format!(
                            "Deployed new version `{}`, running test suite",
                            hash
                        ));
                        hash_to_tag = Some(hash);
                    }
                    Ok(false) => {}
                }
            }
            let suite = ExperimentSuite::new_pre_release(&self.cluster);
            if let Err(e) = self.run_suite(suite) {
                self.report_failure(format!("{}", e));
                return;
            }
            if let Some(hash_to_tag) = hash_to_tag.take() {
                info!("Test suite succeed first time for `{}`", hash_to_tag);
                if let Err(e) = self
                    .deployment_manager
                    .tag_tested_image(hash_to_tag.clone())
                {
                    self.report_failure(format!("Failed to tag tested image: {}", e));
                    return;
                }
                self.slack_message(format!(
                    "Test suite passed. Tagged `{}` as `{}`",
                    hash_to_tag, TESTED_TAG
                ));
            }
            thread::sleep(self.experiment_interval);
        }
    }

    fn report_failure(&self, msg: String) {
        self.slack_message(msg);
    }

    fn redeploy(&mut self, hash: String) -> failure::Result<bool> {
        if env::var("ALLOW_DEPLOY") != Ok("yes".to_string()) {
            info!("Deploying is disabled. Run with ALLOW_DEPLOY=yes to enable deploy");
            return Ok(false);
        }
        self.stop();
        if env::var("WIPE_ON_DEPLOY") != Ok("no".to_string()) {
            info!("Wiping validators");
            self.wipe_all_db(false);
        } else {
            info!("WIPE_ON_DEPLOY is set to no, keeping database");
        }
        self.deployment_manager.redeploy(hash)?;
        thread::sleep(Duration::from_secs(60));
        self.logs.recv_all();
        self.health_check_runner.clear();
        self.start();
        info!("Waiting until all validators healthy after deployment");
        self.wait_until_all_healthy()?;
        Ok(true)
    }

    fn run_suite(&mut self, suite: ExperimentSuite) -> failure::Result<()> {
        info!("Starting suite");
        let suite_started = Instant::now();
        for experiment in suite.experiments {
            let experiment_name = format!("{}", experiment);
            self.run_single_experiment(experiment).map_err(move |e| {
                format_err!("Experiment `{}` failed: `{}`", experiment_name, e)
            })?;
            thread::sleep(self.experiment_interval);
        }
        info!(
            "Suite completed in {:?}",
            Instant::now().duration_since(suite_started)
        );
        Ok(())
    }

    pub fn run_single_experiment(
        &mut self,
        experiment: Box<dyn Experiment>,
    ) -> failure::Result<()> {
        let events = self.logs.recv_all();
        if !self.health_check_runner.run(&events).is_empty() {
            bail!("Some validators are unhealthy before experiment started");
        }

        info!(
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

        // We expect experiments completes and cluster go into healthy state within timeout
        let experiment_deadline = Instant::now() + Duration::from_secs(10 * 60);

        loop {
            if Instant::now() > experiment_deadline {
                bail!("Experiment did not complete in time");
            }
            let deadline = Instant::now() + HEALTH_POLL_INTERVAL;
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = self.logs.recv_all_until_deadline(deadline);
            let failed_validators = self.health_check_runner.run(&events);
            for failed in failed_validators {
                if !affected_validators.contains(&failed) {
                    bail!(
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

        info!(
            "{}Experiment finished, waiting until all affected validators recover{}",
            style::Bold,
            style::Reset
        );

        for validator in affected_validators.iter() {
            self.health_check_runner.invalidate(validator);
        }

        loop {
            if Instant::now() > experiment_deadline {
                bail!("Cluster did not become healthy in time");
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
                    bail!(
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

        info!("Experiment completed");
        Ok(())
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

    fn wait_until_all_healthy(&mut self) -> failure::Result<()> {
        let wait_deadline = Instant::now() + Duration::from_secs(10 * 60);
        for instance in self.cluster.instances() {
            self.health_check_runner.invalidate(instance.short_hash());
        }
        loop {
            let now = Instant::now();
            if now > wait_deadline {
                bail!("Validators did not become healthy after deployment");
            }
            let deadline = now + HEALTH_POLL_INTERVAL;
            let events = self.logs.recv_all_until_deadline(deadline);
            if self.health_check_runner.run(&events).is_empty() {
                break;
            }
        }
        Ok(())
    }

    fn tail_logs(self) {
        for log in self.logs.event_receiver {
            info!("{:?}", log);
        }
    }

    fn slack_message(&self, msg: String) {
        info!("{}", msg);
        if let Some(ref slack) = self.slack {
            if let Err(e) = slack.send_message(&msg) {
                info!("Failed to send slack message: {}", e);
            }
        }
    }

    fn wipe_all_db(&self, safety_wait: bool) {
        info!("Going to wipe db on all validators in cluster!");
        if safety_wait {
            info!("Waiting 10 seconds before proceed");
            thread::sleep(Duration::from_secs(10));
            info!("Starting...");
        }
        let jobs = self
            .cluster
            .instances()
            .iter()
            .map(|instance| {
                let instance = instance.clone();
                move || {
                    if let Err(e) =
                        instance.run_cmd_tee_err(vec!["sudo", "rm", "-rf", "/data/libra/"])
                    {
                        info!("Failed to wipe {}: {:?}", instance, e);
                    }
                }
            })
            .collect();
        self.execute_jobs(jobs);
        info!("Done");
    }

    fn reboot(self) {
        let mut reboots = vec![];
        for instance in self.cluster.instances() {
            info!("Rebooting {}", instance);
            let reboot = Reboot::new(instance.clone());
            if let Err(err) = reboot.apply() {
                info!("Failed to reboot {}: {:?}", instance, err);
            } else {
                reboots.push(reboot);
            }
        }
        info!("Waiting to complete");
        while reboots.iter().any(|r| !r.is_complete()) {
            thread::sleep(Duration::from_secs(5));
        }
        info!("Completed");
    }

    fn restart(&self) {
        self.stop();
        self.start();
        info!("Completed");
    }

    pub fn stop(&self) {
        self.activate_all(&self.make_stop_effects())
    }

    pub fn start(&self) {
        self.deactivate_all(&self.make_stop_effects())
    }

    fn make_stop_effects(&self) -> Vec<StopContainer> {
        self.cluster
            .instances()
            .clone()
            .into_iter()
            .map(StopContainer::new)
            .collect()
    }

    fn activate_all<T: Effect>(&self, effects: &[T]) {
        let jobs = effects
            .iter()
            .map(|effect| {
                move || {
                    if let Err(e) = effect.activate() {
                        info!("Failed to activate {}: {:?}", effect, e);
                    }
                }
            })
            .collect();
        self.execute_jobs(jobs);
    }

    fn deactivate_all<T: Effect>(&self, effects: &[T]) {
        let jobs = effects
            .iter()
            .map(|effect| {
                move || {
                    if let Err(e) = effect.deactivate() {
                        info!("Failed to deactivate {}: {:?}", effect, e);
                    }
                }
            })
            .collect();
        self.execute_jobs(jobs);
    }

    /// Executes jobs, wait for them to complete and return results
    /// Note: Results in vector do not match order of input jobs
    fn execute_jobs<'a, R, J>(&self, jobs: Vec<J>) -> Vec<R>
    where
        R: Send + 'a,
        J: FnOnce() -> R + Send + 'a,
    {
        let (sender, recv) = mpsc::channel();
        let size = jobs.len();
        for job in jobs {
            let sender = sender.clone();
            let closure = move || {
                let r = job();
                sender
                    .send(r)
                    .expect("main execute_jobs thread terminated before worker");
            };
            let closure: Box<dyn FnOnce() + Send + 'a> = Box::new(closure);
            // Using mem::transmute to cast from 'a to 'static lifetime
            // This is safe because we ensure lifetime of current stack frame
            // is longer then lifetime of closure
            // Even if one of worker threads panics, we still going to wait in recv loop below
            // until every single thread completes
            let closure: Box<dyn FnOnce() + Send + 'static> = unsafe { mem::transmute(closure) };
            self.thread_pool.execute(closure);
        }
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let r = recv.recv().expect("One of job threads had panic");
            result.push(r);
        }
        result
    }
}

const ARG_WORKPLACE: &str = "workplace";
const ARG_PEERS: &str = "peers";
// Actions:
const ARG_TAIL_LOGS: &str = "tail-logs";
const ARG_HEALTH_CHECK: &str = "health-check";
const ARG_RUN: &str = "run";
const ARG_RUN_ONCE: &str = "run-once";
const ARG_WIPE_ALL_DB: &str = "wipe-all-db";
const ARG_REBOOT: &str = "reboot";
const ARG_STOP: &str = "stop";
const ARG_START: &str = "start";
const ARG_RESTART: &str = "restart";
const ARG_EMIT_TX: &str = "emit-tx";
const ARG_PRUNE: &str = "prune-logs";

fn arg_matches() -> ArgMatches<'static> {
    // Parameters
    let workplace = Arg::with_name(ARG_WORKPLACE)
        .long("--workplace")
        .short("-w")
        .takes_value(true)
        .required(true);
    let peers = Arg::with_name(ARG_PEERS)
        .long("--peers")
        .short("-p")
        .takes_value(true)
        .use_delimiter(true)
        .conflicts_with(ARG_PRUNE);
    // Actions
    let wipe_all_db = Arg::with_name(ARG_WIPE_ALL_DB).long("--wipe-all-db");
    let run = Arg::with_name(ARG_RUN).long("--run");
    let run_once = Arg::with_name(ARG_RUN_ONCE).long("--run-once");
    let tail_logs = Arg::with_name(ARG_TAIL_LOGS).long("--tail-logs");
    let health_check = Arg::with_name(ARG_HEALTH_CHECK).long("--health-check");
    let prune_logs = Arg::with_name(ARG_PRUNE).long("--prune-logs");
    let reboot = Arg::with_name(ARG_REBOOT).long("--reboot");
    let restart = Arg::with_name(ARG_RESTART).long("--restart");
    let stop = Arg::with_name(ARG_STOP).long("--stop");
    let start = Arg::with_name(ARG_START).long("--start");
    let emit_tx = Arg::with_name(ARG_EMIT_TX).long("--emit-tx");
    // This grouping requires one and only one action (tail logs, run test, etc)
    let action_group = ArgGroup::with_name("action")
        .args(&[
            ARG_TAIL_LOGS,
            ARG_RUN,
            ARG_RUN_ONCE,
            ARG_HEALTH_CHECK,
            ARG_WIPE_ALL_DB,
            ARG_REBOOT,
            ARG_RESTART,
            ARG_STOP,
            ARG_START,
            ARG_EMIT_TX,
            ARG_PRUNE,
        ])
        .required(true);
    App::new("cluster_test")
        .author("Libra Association <opensource@libra.org>")
        .group(action_group)
        .args(&[
            // parameters
            workplace,
            peers,
            // actions
            run,
            run_once,
            tail_logs,
            health_check,
            wipe_all_db,
            reboot,
            restart,
            stop,
            start,
            prune_logs,
            emit_tx,
        ])
        .get_matches()
}
