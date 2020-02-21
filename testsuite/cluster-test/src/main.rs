// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process;
use std::{
    collections::HashSet,
    env, thread,
    time::{Duration, Instant},
};

use chrono::{Datelike, Timelike, Utc};
use rand::prelude::ThreadRng;
use rand::Rng;
use reqwest::Url;
use slog::{o, Drain};
use slog_scope::{info, warn};
use structopt::{clap::ArgGroup, StructOpt};
use termion::{color, style};

use anyhow::{bail, format_err, Result};
use cluster_test::effects::RemoveNetworkEffects;
use cluster_test::experiments::{get_experiment, Context};
use cluster_test::github::GitHub;
use cluster_test::health::PrintFailures;
use cluster_test::instance::Instance;
use cluster_test::prometheus::Prometheus;
use cluster_test::report::SuiteReport;
use cluster_test::tx_emitter::{EmitJobRequest, EmitThreadParams};
use cluster_test::util::unix_timestamp_now;
use cluster_test::{
    aws::Aws,
    cluster::Cluster,
    deployment::DeploymentManager,
    effects::{Action, Effect, Reboot, StopContainer},
    experiments::Experiment,
    health::{DebugPortLogThread, HealthCheckRunner, LogTail},
    slack::SlackClient,
    stats,
    suite::ExperimentSuite,
    tx_emitter::TxEmitter,
};
use futures::future::join_all;
use futures::future::FutureExt;
use futures::future::TryFutureExt;
use futures::select;
use tokio::runtime::{Builder, Runtime};
use tokio::time::{delay_for, delay_until, Instant as TokioInstant};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("action"))]
struct Args {
    #[structopt(short = "p", long, use_delimiter = true)]
    peers: Vec<String>,

    #[structopt(
        long,
        help = "If set, tries to connect to a libra-swarm instead of aws"
    )]
    swarm: bool,

    #[structopt(long, group = "action")]
    wipe_all_db: bool,
    #[structopt(long, group = "action")]
    discovery: bool,
    #[structopt(long, group = "action")]
    pssh: bool,
    #[structopt(long, group = "action")]
    run: Option<String>,
    #[structopt(long, group = "action")]
    tail_logs: bool,
    #[structopt(long, group = "action")]
    health_check: bool,
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
    #[structopt(long, group = "action")]
    stop_experiment: bool,
    #[structopt(long, group = "action")]
    perf_run: bool,
    #[structopt(long, group = "action")]
    cleanup: bool,
    #[structopt(long, group = "action")]
    run_ci_suite: bool,

    #[structopt(last = true)]
    last: Vec<String>,

    #[structopt(long)]
    deploy: Option<String>,
    #[structopt(long, multiple = true)]
    changelog: Option<Vec<String>>,

    // emit_tx options
    #[structopt(long, default_value = "10")]
    accounts_per_client: usize,
    #[structopt(long)]
    workers_per_ac: Option<usize>,
    #[structopt(long, default_value = "0")]
    wait_millis: u64,
    #[structopt(long)]
    burst: bool,
    #[structopt(long, default_value = "mint.key")]
    mint_file: String,
    #[structopt(
        long,
        help = "Time to run --emit-tx for in seconds",
        default_value = "60"
    )]
    duration: u64,

    //stop_experiment options
    #[structopt(long, default_value = "10")]
    max_stopped: usize,

    #[structopt(
        long,
        help = "Whether transactions should be submitted to validators or full nodes"
    )]
    pub emit_to_validator: Option<bool>,
}

pub fn main() {
    setup_log();

    let args = Args::from_args();

    if args.swarm && !args.emit_tx {
        panic!("Can only use --emit-tx option in --swarm mode");
    }

    if args.emit_tx {
        let mut rt = Runtime::new().unwrap();
        let thread_params = EmitThreadParams {
            wait_millis: args.wait_millis,
            wait_committed: !args.burst,
        };
        let duration = Duration::from_secs(args.duration);
        if args.swarm {
            let util = BasicSwarmUtil::setup(&args);
            rt.block_on(util.emit_tx(
                args.accounts_per_client,
                args.workers_per_ac,
                thread_params,
                duration,
            ));
            return;
        } else {
            let util = ClusterUtil::setup(&args);
            rt.block_on(util.emit_tx(
                args.accounts_per_client,
                args.workers_per_ac,
                thread_params,
                duration,
            ));
            return;
        }
    } else if args.discovery {
        let util = ClusterUtil::setup(&args);
        util.discovery();
        return;
    } else if args.pssh {
        let util = ClusterUtil::setup(&args);
        util.pssh(args.last);
        return;
    }

    let mut runner = ClusterTestRunner::setup(&args);

    if let Some(ref hash_or_tag) = args.deploy {
        // Deploy deploy_hash before running whatever command
        let hash = runner
            .deployment_manager
            .resolve(hash_or_tag)
            .expect("Failed to resolve tag");
        exit_on_error(runner.redeploy(&hash));
    }

    let mut perf_msg = None;

    if args.tail_logs {
        runner.tail_logs();
        return;
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
    } else if args.perf_run {
        perf_msg = Some(runner.perf_run());
    } else if args.stop_experiment {
        runner.stop_experiment(args.max_stopped);
    } else if args.cleanup {
        runner.cleanup();
    } else if args.run_ci_suite {
        perf_msg = Some(exit_on_error(runner.run_ci_suite()));
    } else if let Some(experiment_name) = args.run {
        runner
            .cleanup_and_run(get_experiment(
                &experiment_name,
                &args.last,
                &runner.cluster,
            ))
            .unwrap();
        info!(
            "{}Experiment Result: {}{}",
            style::Bold,
            runner.report,
            style::Reset
        );
    } else if args.changelog.is_none() && args.deploy.is_none() {
        println!("No action specified");
        process::exit(1);
    }

    if let Some(mut changelog) = args.changelog {
        if changelog.len() != 2 {
            println!("Use: changelog <from> <to>");
            process::exit(1);
        }
        let to_commit = changelog.remove(1);
        let from_commit = Some(changelog.remove(0));
        if let Some(perf_msg) = perf_msg {
            runner.send_changelog_message(&perf_msg, &from_commit, &to_commit);
        } else {
            println!("{}", runner.get_changelog(from_commit.as_ref(), &to_commit));
        }
    } else if let Some(perf_msg) = perf_msg {
        println!("{}", perf_msg);
    }
}

fn exit_on_error<T>(r: Result<T>) -> T {
    match r {
        Ok(r) => r,
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    }
}

fn setup_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = std::sync::Mutex::new(drain).fuse();
    let logger = slog::Logger::root(drain, o!());
    let logger_guard = slog_scope::set_global_logger(logger);
    std::mem::forget(logger_guard);
}

struct BasicSwarmUtil {
    cluster: Cluster,
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
    runtime: Runtime,
    slack: SlackClient,
    slack_changelog_url: Option<Url>,
    tx_emitter: TxEmitter,
    prometheus: Prometheus,
    github: GitHub,
    report: SuiteReport,
    global_emit_job_request: EmitJobRequest,
    emit_to_validator: bool,
}

fn parse_host_port(s: &str) -> Result<(String, u32)> {
    let v = s.split(':').collect::<Vec<&str>>();
    if v.len() != 2 {
        return Err(format_err!("Failed to parse {:?} in host:port format", s));
    }
    let host = v[0].to_string();
    let port = v[1].parse::<u32>()?;
    Ok((host, port))
}

impl BasicSwarmUtil {
    pub fn setup(args: &Args) -> Self {
        if args.peers.is_empty() {
            panic!("Peers not set in args");
        }
        let parsed_peers: Vec<_> = args
            .peers
            .iter()
            .map(|peer| parse_host_port(peer).unwrap())
            .collect();
        Self {
            cluster: Cluster::from_host_port(parsed_peers, &args.mint_file),
        }
    }

    pub async fn emit_tx(
        self,
        accounts_per_client: usize,
        workers_per_ac: Option<usize>,
        thread_params: EmitThreadParams,
        duration: Duration,
    ) {
        let mut emitter = TxEmitter::new(&self.cluster);
        let job = emitter
            .start_job(EmitJobRequest {
                instances: self.cluster.validator_instances().to_vec(),
                accounts_per_client,
                workers_per_ac,
                thread_params,
            })
            .await
            .expect("Failed to start emit job");
        thread::sleep(duration);
        emitter.stop_job(job);
    }
}

impl ClusterUtil {
    pub fn setup(args: &Args) -> Self {
        let aws = Aws::new();
        let cluster = Cluster::discover(&aws).expect("Failed to discover cluster");
        let cluster = if args.peers.is_empty() {
            cluster
        } else {
            cluster.validator_sub_cluster(args.peers.clone())
        };
        let prometheus = Prometheus::new(
            cluster
                .prometheus_ip()
                .expect("Failed to discover prometheus ip in aws"),
            aws.workspace(),
        );
        info!(
            "Discovered {} validators and {} fns in {} workspace",
            cluster.validator_instances().len(),
            cluster.fullnode_instances().len(),
            aws.workspace()
        );
        Self {
            cluster,
            aws,
            prometheus,
        }
    }

    pub fn discovery(&self) {
        for instance in self.cluster.all_instances() {
            println!("{} {}", instance.peer_name(), instance.ip());
        }
    }

    pub fn pssh(&self, cmd: Vec<String>) {
        let mut runtime = Runtime::new().unwrap();
        let futures = self.cluster.all_instances().map(|x| {
            x.run_cmd_tee_err(&cmd).map(move |r| {
                if let Err(e) = r {
                    warn!("Failed on {}: {}", x, e)
                }
            })
        });
        runtime.block_on(join_all(futures));
    }

    pub async fn emit_tx(
        self,
        accounts_per_client: usize,
        workers_per_ac: Option<usize>,
        thread_params: EmitThreadParams,
        duration: Duration,
    ) {
        let mut emitter = TxEmitter::new(&self.cluster);
        emitter
            .start_job(EmitJobRequest {
                instances: self.cluster.validator_instances().to_vec(),
                accounts_per_client,
                workers_per_ac,
                thread_params,
            })
            .await
            .expect("Failed to start emit job");
        self.run_stat_loop(duration);
    }

    fn run_stat_loop(&self, duration: Duration) {
        let deadline = Instant::now() + duration;
        let window = Duration::from_secs(30);
        thread::sleep(Duration::from_secs(30)); // warm up
        loop {
            if Instant::now() > deadline {
                return;
            }
            thread::sleep(Duration::from_secs(10));
            let now = unix_timestamp_now();
            match stats::txn_stats(&self.prometheus, now - window, now) {
                Ok((avg_tps, avg_latency)) => {
                    info!("Tps: {:.0}, latency: {:.0} ms", avg_tps, avg_latency)
                }
                Err(err) => info!("Stat error: {:?}", err),
            }
        }
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
        let workspace = aws.workspace().clone();
        let deployment_manager = DeploymentManager::new(aws, cluster.clone());
        let slack = SlackClient::new();
        let slack_changelog_url = env::var("SLACK_CHANGELOG_URL")
            .map(|u| u.parse().expect("Failed to parse SLACK_CHANGELOG_URL"))
            .ok();
        let tx_emitter = TxEmitter::new(&cluster);
        let prometheus = Prometheus::new(
            cluster
                .prometheus_ip()
                .expect("Failed to discover prometheus ip in aws"),
            &workspace,
        );
        let github = GitHub::new();
        let report = SuiteReport::new();
        let runtime = Builder::new()
            .threaded_scheduler()
            .core_threads(num_cpus::get())
            .thread_name("ct-tokio")
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        let global_emit_job_request = EmitJobRequest {
            instances: vec![],
            accounts_per_client: args.accounts_per_client,
            workers_per_ac: args.workers_per_ac,
            thread_params: EmitThreadParams {
                wait_millis: args.wait_millis,
                wait_committed: !args.burst,
            },
        };
        let emit_to_validator =
            if cluster.fullnode_instances().len() < cluster.validator_instances().len() {
                true
            } else {
                args.emit_to_validator.unwrap_or(false)
            };
        Self {
            logs,
            cluster,
            health_check_runner,
            deployment_manager,
            experiment_interval,
            slack,
            runtime,
            slack_changelog_url,
            tx_emitter,
            prometheus,
            github,
            report,
            global_emit_job_request,
            emit_to_validator,
        }
    }

    pub fn run_ci_suite(&mut self) -> Result<String> {
        let suite = ExperimentSuite::new_pre_release(&self.cluster);
        self.run_suite(suite)?;
        let perf_msg = format!("Performance report:\n```\n{}\n```", self.report);
        Ok(perf_msg)
    }

    pub fn send_changelog_message(
        &self,
        perf_msg: &str,
        from_commit: &Option<String>,
        to_commit: &str,
    ) {
        info!(
            "Generating changelog from {:?} to {}",
            from_commit, to_commit
        );
        let changelog = self.get_changelog(from_commit.as_ref(), &to_commit);
        self.slack_changelog_message(format!("{}\n\n{}", changelog, perf_msg));
    }

    fn get_changelog(&self, prev_commit: Option<&String>, upstream_commit: &str) -> String {
        let commits = self.github.get_commits("libra/libra", &upstream_commit);
        match commits {
            Err(e) => {
                info!("Failed to get github commits: {:?}", e);
                format!("*Revision upstream_{}*", upstream_commit)
            }
            Ok(commits) => {
                let mut msg = format!("*Revision {}*", upstream_commit);
                for commit in commits {
                    if let Some(prev_commit) = prev_commit {
                        if commit.sha.starts_with(prev_commit) {
                            break;
                        }
                    }
                    let commit_lines: Vec<_> = commit.commit.message.split('\n').collect();
                    let commit_head = commit_lines[0];
                    let short_sha = &commit.sha[..6];
                    let email_parts: Vec<_> = commit.commit.author.email.split('@').collect();
                    let author = email_parts[0];
                    let line = format!("\n>\u{2022} {} _{}_ {}", short_sha, author, commit_head);
                    msg.push_str(&line);
                }
                msg
            }
        }
    }

    fn redeploy(&mut self, hash: &str) -> Result<()> {
        info!("Cleaning up before deploy");
        self.cleanup();
        info!("Stopping validators");
        self.stop();
        if env::var("WIPE_ON_DEPLOY") != Ok("no".to_string()) {
            info!("Wiping validators");
            self.wipe_all_db(false);
        } else {
            info!("WIPE_ON_DEPLOY is set to no, keeping database");
        }
        self.deployment_manager.redeploy(hash.to_string())?;
        thread::sleep(Duration::from_secs(60));
        self.logs.recv_all();
        self.health_check_runner.clear();
        self.tx_emitter.clear();
        self.start();
        info!("Waiting until all validators healthy after deployment");
        self.wait_until_all_healthy()?;
        Ok(())
    }

    fn run_suite(&mut self, suite: ExperimentSuite) -> Result<()> {
        info!("Starting suite");
        let suite_started = Instant::now();
        for experiment in suite.experiments {
            let experiment_name = format!("{}", experiment);
            self.run_single_experiment(experiment, None)
                .map_err(move |e| {
                    format_err!("Experiment `{}` failed: `{}`", experiment_name, e)
                })?;
            thread::sleep(self.experiment_interval);
        }
        info!(
            "Suite completed in {:?}",
            Instant::now().duration_since(suite_started)
        );
        self.print_report();
        Ok(())
    }

    pub fn print_report(&self) {
        let json_report =
            serde_json::to_string_pretty(&self.report).expect("Failed to serialize report to json");
        println!("====json-report-begin===");
        println!("{}", json_report);
        println!("====json-report-end===");
    }

    pub fn perf_run(&mut self) -> String {
        let suite = ExperimentSuite::new_perf_suite(&self.cluster);
        self.run_suite(suite).unwrap();
        self.report.to_string()
    }

    pub fn cleanup_and_run(&mut self, experiment: Box<dyn Experiment>) -> Result<()> {
        self.cleanup();
        self.run_single_experiment(experiment, Some(self.global_emit_job_request.clone()))?;
        self.print_report();
        Ok(())
    }

    pub fn run_single_experiment(
        &mut self,
        mut experiment: Box<dyn Experiment>,
        mut global_emit_job_request: Option<EmitJobRequest>,
    ) -> Result<()> {
        let events = self.logs.recv_all();
        if let Err(s) =
            self.health_check_runner
                .run(&events, &HashSet::new(), PrintFailures::UnexpectedOnly)
        {
            bail!(
                "Some validators are unhealthy before experiment started : {}",
                s
            );
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
        let deadline = experiment.deadline();
        let experiment_deadline = Instant::now() + deadline;
        let context = Context::new(
            &mut self.tx_emitter,
            &self.prometheus,
            &self.cluster,
            &mut self.report,
            &mut global_emit_job_request,
            self.emit_to_validator,
        );
        {
            let logs = &mut self.logs;
            let health_check_runner = &mut self.health_check_runner;
            let affected_validators = &affected_validators;
            self.runtime.block_on(async move {
                let mut context = context;
                let mut deadline_future =
                    delay_until(TokioInstant::from_std(experiment_deadline)).fuse();
                let mut run_future = experiment.run(&mut context).fuse();
                loop {
                    select! {
                        delay = deadline_future => {
                            bail!("Experiment deadline reached");
                        }
                        result = run_future => {
                            return result.map_err(|e|format_err!("Failed to run experiment: {}", e));
                        }
                        delay = delay_for(HEALTH_POLL_INTERVAL).fuse() => {
                            let events = logs.recv_all();
                            if let Err(s) = health_check_runner.run(
                                &events,
                                &affected_validators,
                                PrintFailures::UnexpectedOnly,
                            ) {
                                bail!("Validators which were not under experiment failed : {}", s);
                            }
                        }
                    }
                }
            })?;
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

            let unhealthy_validators;
            match self.health_check_runner.run(
                &events,
                &affected_validators,
                PrintFailures::UnexpectedOnly,
            ) {
                Err(s) => bail!("Validators which were not under experiment failed : {}", s),
                Ok(r) => unhealthy_validators = r,
            }
            if unhealthy_validators.is_empty() {
                break;
            }
        }

        info!("Experiment completed");
        Ok(())
    }

    pub fn stop_experiment(&mut self, max_stopped: usize) {
        let mut instances = self.cluster.validator_instances().to_vec();
        let mut rng = ThreadRng::default();
        let mut stop_effects = vec![];
        let mut stopped_instance_ids = vec![];
        let mut results = vec![];
        let window = Duration::from_secs(60);
        loop {
            let job = self
                .runtime
                .block_on(self.tx_emitter.start_job(EmitJobRequest::for_instances(
                    instances.clone(),
                    &Some(self.global_emit_job_request.clone()),
                )))
                .expect("Failed to start emit job");
            thread::sleep(Duration::from_secs(30) + window);
            let now = unix_timestamp_now();
            match stats::txn_stats(&self.prometheus, now - window, now) {
                Ok((avg_tps, avg_latency)) => {
                    results.push((stop_effects.len(), avg_tps, avg_latency));
                    info!("Tps: {:.0}, latency: {:.0} ms", avg_tps, avg_latency);
                }
                Err(err) => info!("Stat error: {:?}", err),
            }
            self.tx_emitter.stop_job(job);
            if stop_effects.len() > max_stopped {
                break;
            }
            let stop_validator = rng.gen_range(0, instances.len());
            let stop_validator = instances.remove(stop_validator);
            stopped_instance_ids.push(stop_validator.peer_name().clone());
            let stop_effect = StopContainer::new(stop_validator);
            info!(
                "Stopped {} validators: {}",
                stopped_instance_ids.len(),
                stopped_instance_ids.join(",")
            );
            self.runtime
                .block_on(stop_effect.activate())
                .expect("Failed to stop container");
            stop_effects.push(stop_effect);
            thread::sleep(Duration::from_secs(30));
        }
        println!("Results in csv format:");
        println!("DOWN\tTPS\tLAT");
        for (stopped, tps, lat) in results {
            println!("{}\t{:.0}\t{:.0}", stopped, tps, lat * 1000.);
        }
        let futures = stop_effects.iter().map(|stop_effect| {
            stop_effect
                .deactivate()
                .map_err(move |e| info!("Failed to deactivate {}: {:?}", stop_effect, e))
        });
        self.runtime.block_on(join_all(futures));
    }

    fn run_health_check(&mut self) {
        loop {
            let deadline = Instant::now() + Duration::from_secs(1);
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = self.logs.recv_all_until_deadline(deadline);
            let _ignore =
                self.health_check_runner
                    .run(&events, &HashSet::new(), PrintFailures::All);
        }
    }

    fn wait_until_all_healthy(&mut self) -> Result<()> {
        let wait_deadline = Instant::now() + Duration::from_secs(20 * 60);
        for instance in self.cluster.validator_instances() {
            self.health_check_runner.invalidate(instance.peer_name());
        }
        loop {
            let now = Instant::now();
            if now > wait_deadline {
                bail!("Validators did not become healthy after deployment");
            }
            let deadline = now + HEALTH_POLL_INTERVAL;
            let events = self.logs.recv_all_until_deadline(deadline);
            if let Ok(failed_instances) =
                self.health_check_runner
                    .run(&events, &HashSet::new(), PrintFailures::None)
            {
                if failed_instances.is_empty() {
                    break;
                }
            }
        }
        Ok(())
    }

    fn tail_logs(self) {
        for log in self.logs.event_receiver {
            info!("{:?}", log);
        }
    }

    fn slack_changelog_message(&self, msg: String) {
        info!("{}", msg);
        if let Some(ref changelog_url) = self.slack_changelog_url {
            if let Err(e) = self.slack.send_message(changelog_url, &msg) {
                info!("Failed to send slack message: {}", e);
            }
        }
    }

    fn wipe_all_db(&mut self, safety_wait: bool) {
        info!("Going to wipe db on all validators in cluster!");
        if safety_wait {
            info!("Waiting 10 seconds before proceed");
            thread::sleep(Duration::from_secs(10));
            info!("Starting...");
        }
        let now = Utc::now();
        let suffix = format!(
            ".{:04}{:02}{:02}-{:02}{:02}{:02}.gz",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        let suffix = &suffix;
        let log_file = "/data/libra/libra.log";
        info!("Will use suffix {} for log rotation", suffix);
        let jobs = self
            .cluster
            .all_instances()
            .map(|instance| Self::wipe_instance(log_file, &suffix, instance));
        self.runtime.block_on(join_all(jobs));
        info!("Done");
    }

    async fn wipe_instance(log_file: &str, suffix: &str, instance: &Instance) {
        instance
            .run_cmd_tee_err(vec!["sudo", "rm", "-rf", "/data/libra/common"])
            .await
            .map_err(|e| info!("Failed to wipe {}: {:?}", instance, e))
            .ok();
        instance
            .run_cmd_tee_err(vec![format!(
                "! test -f {f} || (sudo timeout 45 gzip -S {s} {f} || (echo gzip failed; sudo rm -f {f}))",
                f = log_file,
                s = suffix
            )]).await
            .map_err(|e| info!("Failed to gzip log file {}: {:?}", instance, e))
            .ok();
    }

    fn reboot(&mut self) {
        let futures = self.cluster.all_instances().map(|instance| {
            async move {
                let reboot = Reboot::new(instance.clone());
                reboot
                    .apply()
                    .await
                    .map_err(|e| info!("Failed to reboot {}: {:?}", instance, e))
            }
        });
        self.runtime.block_on(join_all(futures));
        info!("Completed");
    }

    fn restart(&mut self) {
        self.stop();
        self.start();
        info!("Completed");
    }

    fn cleanup(&mut self) {
        let futures = self.cluster.all_instances().map(|instance| {
            async move {
                RemoveNetworkEffects::new(instance.clone())
                    .apply()
                    .await
                    .map_err(|e| {
                        info!(
                            "Failed to remove network effects for {}. Error: {}",
                            instance, e
                        );
                    })
            }
        });
        self.runtime.block_on(join_all(futures));
    }

    pub fn stop(&mut self) {
        let effects = self.make_stop_effects();
        let futures = effects.iter().map(|e| e.activate());
        self.runtime.block_on(join_all(futures));
    }

    pub fn start(&mut self) {
        let effects = self.make_stop_effects();
        let futures = effects.iter().map(|e| e.deactivate());
        self.runtime.block_on(join_all(futures));
    }

    fn make_stop_effects(&self) -> Vec<StopContainer> {
        self.cluster
            .all_instances()
            .map(|x| StopContainer::new(x.clone()))
            .collect()
    }
}
