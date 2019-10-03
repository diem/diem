use cluster_test::prometheus::Prometheus;
use cluster_test::tx_emitter::{EmitJobRequest, EmitThreadParams};
use cluster_test::util::unix_timestamp_now;
use cluster_test::{
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
use failure::{
    self,
    prelude::{bail, format_err},
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
use structopt::{clap::ArgGroup, StructOpt};
use termion::{color, style};
use threadpool;

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("action").required(true))]
struct Args {
    #[structopt(short = "w", long)]
    workplace: String,
    #[structopt(short = "p", long, use_delimiter = true, conflicts_with = "prune-logs")]
    peers: Vec<String>,

    #[structopt(long, group = "action")]
    wipe_all_db: bool,
    #[structopt(long, group = "action")]
    run: bool,
    #[structopt(long, group = "action")]
    run_once: bool,
    #[structopt(long, group = "action")]
    tail_logs: bool,
    #[structopt(long, group = "action")]
    health_check: bool,
    #[structopt(long, group = "action")]
    prune_logs: bool,
    #[structopt(long, group = "action")]
    reboot: bool,
    #[structopt(long, group = "action")]
    restart: bool,
    #[structopt(long, group = "action")]
    stop: bool,
    #[structopt(long, group = "action")]
    start: bool,
    #[structopt(long, group = "action")]
    emit_tx: bool,

    // emit_tx options
    #[structopt(long, default_value = "10")]
    accounts_per_client: usize,
    #[structopt(long, default_value = "50")]
    wait_millis: u64,
    #[structopt(long)]
    burst: bool,
}

pub fn main() {
    setup_log();

    let args = Args::from_args();

    if args.prune_logs {
        let util = ClusterUtil::setup(&args);
        util.prune_logs();
        return;
    } else if args.emit_tx {
        let util = ClusterUtil::setup(&args);
        let thread_params = EmitThreadParams {
            wait_millis: args.wait_millis,
            wait_committed: !args.burst,
        };
        util.emit_tx(args.accounts_per_client, thread_params);
        return;
    }

    let mut runner = ClusterTestRunner::setup(&args);

    if args.run {
        runner.run_suite_in_loop();
    } else if args.run_once {
        let experiment = RebootRandomValidators::new(3, &runner.cluster);
        runner.run_single_experiment(Box::new(experiment)).unwrap();
    } else if args.tail_logs {
        runner.tail_logs();
    } else if args.health_check {
        runner.run_health_check();
    } else if args.wipe_all_db {
        runner.stop();
        runner.wipe_all_db(true);
        runner.start();
    } else if args.reboot {
        runner.reboot();
    } else if args.restart {
        runner.restart();
    } else if args.stop {
        runner.stop();
    } else if args.start {
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
    prometheus: Prometheus,
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
    pub fn setup(args: &Args) -> Self {
        let aws = Aws::new(args.workplace.clone());
        let cluster = Cluster::discover(&aws).expect("Failed to discover cluster");
        let cluster = if args.peers.is_empty() {
            cluster
        } else {
            cluster.sub_cluster(args.peers.clone())
        };
        let prometheus = Prometheus::new(cluster.prometheus_ip());
        info!("Discovered {} peers", cluster.instances().len());
        Self {
            cluster,
            aws,
            prometheus,
        }
    }

    pub fn prune_logs(&self) {
        let log_prune = LogPruner::new(self.aws.clone());
        log_prune.prune_logs();
    }

    pub fn emit_tx(self, accounts_per_client: usize, thread_params: EmitThreadParams) {
        let mut emitter = TxEmitter::new(&self.cluster);
        let _job = emitter.start_job(EmitJobRequest {
            instances: self.cluster.instances().to_vec(),
            accounts_per_client,
            thread_params,
        });
        self.run_stat_loop();
    }

    fn run_stat_loop(&self) {
        thread::sleep(Duration::from_secs(30)); // warm up
        loop {
            thread::sleep(Duration::from_secs(10));
            if let Err(err) = self.print_stat() {
                info!("Stat error: {:?}", err);
            }
        }
    }

    fn print_stat(&self) -> failure::Result<()> {
        let step = 10;
        let end = unix_timestamp_now();
        let start = end - Duration::from_secs(30); // avg over last 30 sec
        let tps = self.prometheus.query_range(
            "irate(consensus_gauge{op='last_committed_version'}[1m])".to_string(),
            &start,
            &end,
            step,
        )?;
        let avg_tps = tps.avg().ok_or_else(|| format_err!("No tps data"))?;
        let latency = self.prometheus.query_range(
            "irate(mempool_duration_sum{op='e2e.latency'}[1m])/irate(mempool_duration_count{op='e2e.latency'}[1m])"
                .to_string(),
            &start,
            &end,
            step,
        )?;
        let avg_latency = latency
            .avg()
            .ok_or_else(|| format_err!("No latency data"))?;
        info!(
            "Tps: {:.0}, latency: {:.0} ms",
            avg_tps,
            avg_latency * 1000.
        );
        Ok(())
    }
}

impl ClusterTestRunner {
    /// Discovers cluster, setup log, etc
    pub fn setup(args: &Args) -> Self {
        let util = ClusterUtil::setup(args);
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
