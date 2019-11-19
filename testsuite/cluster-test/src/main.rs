// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use chrono::{Datelike, Timelike, Utc};
use cluster_test::effects::RemoveNetworkEffects;
use cluster_test::experiments::{MultiRegionSimulation, PacketLossRandomValidators};
use cluster_test::github::GitHub;
use cluster_test::health::PrintFailures;
use cluster_test::instance::Instance;
use cluster_test::prometheus::Prometheus;
use cluster_test::thread_pool_executor::ThreadPoolExecutor;
use cluster_test::tx_emitter::{EmitJobRequest, EmitThreadParams};
use cluster_test::util::unix_timestamp_now;
use cluster_test::{
    aws::Aws,
    cluster::Cluster,
    deployment::{DeploymentManager, SOURCE_TAG},
    effects::{Action, Effect, Reboot, StopContainer},
    experiments::{Experiment, RebootRandomValidators},
    health::{DebugPortLogThread, HealthCheckRunner, LogTail},
    slack::SlackClient,
    suite::ExperimentSuite,
    tx_emitter::TxEmitter,
};
use failure::{
    self,
    prelude::{bail, format_err},
};
use rand::prelude::ThreadRng;
use rand::Rng;
use reqwest::Url;
use slog::{o, Drain};
use slog_scope::{info, warn};
use std::process;
use std::{
    collections::HashSet,
    env,
    sync::mpsc::{self, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use structopt::{clap::ArgGroup, StructOpt};
use termion::{color, style};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("action").required(true))]
struct Args {
    #[structopt(short = "w", long, conflicts_with = "swarm")]
    workplace: Option<String>,
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
    #[structopt(last = true, requires = "pssh")]
    last: Vec<String>,
    #[structopt(long, group = "action")]
    run: bool,
    #[structopt(long, group = "action")]
    run_once: bool,
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
    packet_loss_experiment: bool,
    #[structopt(long, group = "action")]
    perf_run: bool,
    #[structopt(long, group = "action")]
    cleanup: bool,
    #[structopt(long, group = "action")]
    multi_region_simulation: bool,
    #[structopt(long, group = "action")]
    changelog: Option<String>,
    #[structopt(long, group = "action")]
    run_ci_suite: bool,

    #[structopt(long)]
    deploy: Option<String>,

    // emit_tx options
    #[structopt(long, default_value = "10")]
    accounts_per_client: usize,
    #[structopt(long, default_value = "50")]
    wait_millis: u64,
    #[structopt(long)]
    burst: bool,
    #[structopt(long, default_value = "mint.key")]
    mint_file: String,

    //stop_experiment options
    #[structopt(long, default_value = "10")]
    max_stopped: usize,

    // multi_region_simulation: options
    #[structopt(
        long,
        default_value = "10",
        help = "Space separated list of various split sizes"
    )]
    multi_region_splits: Vec<usize>,
    #[structopt(
        long,
        default_value = "50",
        help = "Space separated list of various delays in ms between the two regions"
    )]
    multi_region_delays_ms: Vec<u64>,
    #[structopt(
        long,
        default_value = "300",
        help = "Duration in secs (per config) for which multi region experiment happens"
    )]
    multi_region_exp_duration_secs: u64,

    //packet_loss_experiment options
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of instances in which packet loss should be introduced"
    )]
    packet_loss_percent_instances: f32,
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of packet loss for each instance"
    )]
    packet_loss_percent: f32,
    #[structopt(
        long,
        default_value = "60",
        help = "Duration in secs for which packet loss happens"
    )]
    packet_loss_duration_secs: u64,
}

pub fn main() {
    setup_log();

    let args = Args::from_args();

    if args.swarm && !args.emit_tx {
        panic!("Can only use --emit-tx option in --swarm mode");
    }

    if args.emit_tx {
        let thread_params = EmitThreadParams {
            wait_millis: args.wait_millis,
            wait_committed: !args.burst,
        };
        if args.swarm {
            let util = BasicSwarmUtil::setup(&args);
            util.emit_tx(args.accounts_per_client, thread_params);
            return;
        } else {
            let util = ClusterUtil::setup(&args);
            util.emit_tx(args.accounts_per_client, thread_params);
            return;
        }
    } else if args.stop_experiment {
        let util = ClusterUtil::setup(&args);
        util.stop_experiment(args.max_stopped);
        return;
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

    if let Some(ref deploy_hash) = args.deploy {
        // Deploy deploy_hash before running whatever command
        exit_on_error(runner.redeploy(deploy_hash));
    }

    if args.run {
        runner.run_suite_in_loop();
    } else if args.run_once {
        let experiment = RebootRandomValidators::new(3, &runner.cluster);
        runner.cleanup_and_run(Box::new(experiment)).unwrap();
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
    } else if args.packet_loss_experiment {
        let total_instances = runner.cluster.instances().len();
        let packet_loss_num_instances: usize = std::cmp::min(
            ((args.packet_loss_percent_instances / 100.0) * total_instances as f32).ceil() as usize,
            total_instances,
        );
        let experiment = PacketLossRandomValidators::new(
            packet_loss_num_instances,
            args.packet_loss_percent,
            Duration::from_secs(args.packet_loss_duration_secs),
            &runner.cluster,
        );
        runner.cleanup_and_run(Box::new(experiment)).unwrap();
    } else if args.multi_region_simulation {
        let experiment = MultiRegionSimulation::new(
            args.multi_region_splits.clone(),
            args.multi_region_delays_ms.clone(),
            Duration::from_secs(args.multi_region_exp_duration_secs),
            runner.cluster.clone(),
            runner.thread_pool_executor.clone(),
            runner.prometheus.clone(),
        );
        runner.cleanup_and_run(Box::new(experiment)).unwrap();
    } else if args.perf_run {
        runner.perf_run();
    } else if args.cleanup {
        runner.cleanup();
    } else if let Some(commit) = args.changelog {
        let prev_commit = runner
            .deployment_manager
            .get_tested_upstream_commit()
            .map_err(|e| warn!("Failed to get prev_commit: {:?}", e))
            .ok();
        println!("Prev commit: {:?}", prev_commit);
        println!("{}", runner.get_changelog(prev_commit.as_ref(), &commit));
    } else if args.run_ci_suite {
        exit_on_error(runner.run_ci_suite(args.deploy));
    } else {
        // Arg parser should prevent this from happening since action group is required
        unreachable!()
    }
}

fn exit_on_error<T>(r: failure::Result<T>) -> T {
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
    thread_pool_executor: ThreadPoolExecutor,
    slack: SlackClient,
    slack_log_url: Option<Url>,
    slack_changelog_url: Option<Url>,
    tx_emitter: TxEmitter,
    prometheus: Prometheus,
    github: GitHub,
}

fn parse_host_port(s: &str) -> failure::Result<(String, u32)> {
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

    pub fn emit_tx(self, accounts_per_client: usize, thread_params: EmitThreadParams) {
        let mut emitter = TxEmitter::new(&self.cluster);
        emitter
            .start_job(EmitJobRequest {
                instances: self.cluster.instances().to_vec(),
                accounts_per_client,
                thread_params,
            })
            .expect("Failed to start emit job");
        thread::park();
    }
}

impl ClusterUtil {
    pub fn setup(args: &Args) -> Self {
        let aws = Aws::new(
            args.workplace
                .as_ref()
                .expect("--workplace not set")
                .clone(),
        );
        let cluster = Cluster::discover(&aws, &args.mint_file).expect("Failed to discover cluster");
        let cluster = if args.peers.is_empty() {
            cluster
        } else {
            cluster.sub_cluster(args.peers.clone())
        };
        let prometheus = Prometheus::new(
            cluster
                .prometheus_ip()
                .expect("Failed to discover prometheus ip in aws"),
        );
        info!("Discovered {} peers", cluster.instances().len());
        Self {
            cluster,
            aws,
            prometheus,
        }
    }

    pub fn discovery(&self) {
        for instance in self.cluster.instances() {
            println!("{} {}", instance.short_hash(), instance.ip());
        }
    }

    pub fn pssh(&self, cmd: Vec<String>) {
        let executor = ThreadPoolExecutor::new("pssh".to_string());
        let jobs = self
            .cluster
            .instances()
            .iter()
            .map(|x| {
                let cmd = &cmd;
                move || {
                    if let Err(e) = x.run_cmd_tee_err(cmd) {
                        warn!("Failed on {}: {}", x, e)
                    }
                }
            })
            .collect();
        executor.execute_jobs(jobs);
    }

    pub fn emit_tx(self, accounts_per_client: usize, thread_params: EmitThreadParams) {
        let mut emitter = TxEmitter::new(&self.cluster);
        emitter
            .start_job(EmitJobRequest {
                instances: self.cluster.instances().to_vec(),
                accounts_per_client,
                thread_params,
            })
            .expect("Failed to start emit job");
        self.run_stat_loop();
    }

    pub fn stop_experiment(self, max_stopped: usize) {
        let mut emitter = TxEmitter::new(&self.cluster);
        let mut instances = self.cluster.instances().to_vec();
        let mut rng = ThreadRng::default();
        let mut stop_effects = vec![];
        let mut stopped_instance_ids = vec![];
        let mut results = vec![];
        let window = Duration::from_secs(60);
        loop {
            let job = emitter
                .start_job(EmitJobRequest {
                    instances: instances.clone(),
                    accounts_per_client: 10,
                    thread_params: EmitThreadParams::default(),
                })
                .expect("Failed to start emit job");
            thread::sleep(Duration::from_secs(30) + window);
            match print_stat(&self.prometheus, window) {
                Err(e) => info!("Failed to get stats: {:?}", e),
                Ok((tps, lat)) => results.push((stop_effects.len(), tps, lat)),
            }
            emitter.stop_job(job);
            if stop_effects.len() > max_stopped {
                break;
            }
            let stop_validator = rng.gen_range(0, instances.len());
            let stop_validator = instances.remove(stop_validator);
            stopped_instance_ids.push(stop_validator.short_hash().clone());
            let stop_effect = StopContainer::new(stop_validator);
            info!(
                "Stopped {} validators: {}",
                stopped_instance_ids.len(),
                stopped_instance_ids.join(",")
            );
            stop_effect.activate().expect("Failed to stop container");
            stop_effects.push(stop_effect);
            thread::sleep(Duration::from_secs(30));
        }
        println!("Results in csv format:");
        println!("DOWN\tTPS\tLAT");
        for (stopped, tps, lat) in results {
            println!("{}\t{:.0}\t{:.0}", stopped, tps, lat * 1000.);
        }
        for stop_effect in stop_effects {
            if let Err(e) = stop_effect.deactivate() {
                info!("Failed to deactivate {}: {:?}", stop_effect, e);
            }
        }
    }

    fn run_stat_loop(&self) {
        let window = Duration::from_secs(30);
        thread::sleep(Duration::from_secs(30)); // warm up
        loop {
            thread::sleep(Duration::from_secs(10));
            if let Err(err) = print_stat(&self.prometheus, window) {
                info!("Stat error: {:?}", err);
            }
        }
    }
}

fn print_stat(prometheus: &Prometheus, window: Duration) -> failure::Result<(f64, f64)> {
    let step = 10;
    let end = unix_timestamp_now();
    let start = end - window;
    let avg_tps = prometheus
        .query_range_avg(
            "irate(libra_consensus_last_committed_version[1m])".to_string(),
            &start,
            &end,
            step,
        )
        .map_err(|e| format_err!("No tps data: {}", e))?;
    let avg_latency = prometheus.query_range_avg(
        "irate(mempool_duration_sum{op='e2e.latency'}[1m])/irate(mempool_duration_count{op='e2e.latency'}[1m])"
            .to_string(),
        &start,
        &end,
        step,
    ).map_err(|_| format_err!("No latency data"))?;
    info!(
        "Tps: {:.0}, latency: {:.0} ms",
        avg_tps,
        avg_latency * 1000.
    );
    Ok((avg_tps, avg_latency))
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
        let slack = SlackClient::new();
        let slack_log_url = env::var("SLACK_LOG_URL")
            .map(|u| u.parse().expect("Failed to parse SLACK_LOG_URL"))
            .ok();
        let slack_changelog_url = env::var("SLACK_CHANGELOG_URL")
            .map(|u| u.parse().expect("Failed to parse SLACK_CHANGELOG_URL"))
            .ok();
        let thread_pool_executor = ThreadPoolExecutor::new("ssh-pool".into());
        let tx_emitter = TxEmitter::new(&cluster);
        let prometheus = Prometheus::new(
            cluster
                .prometheus_ip()
                .expect("Failed to discover prometheus ip in aws"),
        );
        let github = GitHub::new();
        Self {
            logs,
            cluster,
            health_check_runner,
            deployment_manager,
            experiment_interval,
            slack,
            thread_pool_executor,
            slack_log_url,
            slack_changelog_url,
            tx_emitter,
            prometheus,
            github,
        }
    }

    pub fn run_ci_suite(&mut self, hash_to_tag: Option<String>) -> failure::Result<()> {
        let suite = ExperimentSuite::new_pre_release(&self.cluster);
        self.run_suite(suite)?;
        info!("Starting measure_performance");
        let report = self.measure_performance()?;
        let perf_msg = format!(
            "Performance report:\n```\n{}\n```",
            report.to_slack_message()
        );
        if let Some(hash_to_tag) = hash_to_tag {
            info!("Test suite succeed first time for `{}`", hash_to_tag);
            let prev_commit = self
                .deployment_manager
                .get_tested_upstream_commit()
                .map_err(|e| warn!("Failed to get prev_commit: {}", e))
                .ok();
            let upstream_commit = match self
                .deployment_manager
                .tag_tested_image(hash_to_tag.clone())
            {
                Err(e) => {
                    return Err(format_err!("Failed to tag tested image: {}", e));
                }
                Ok(upstream_commit) => upstream_commit,
            };
            info!(
                "prev_commit: {:?}, upstream_commit: {}",
                prev_commit, upstream_commit
            );
            let changelog = self.get_changelog(prev_commit.as_ref(), &upstream_commit);
            self.slack_changelog_message(format!("{}\n\n{}", changelog, perf_msg));
        } else {
            println!("{}", perf_msg);
        }
        Ok(())
    }

    pub fn run_suite_in_loop(&mut self) {
        self.cleanup();
        loop {
            let hash_to_tag;
            if let Some(hash) = self.deployment_manager.latest_hash_changed() {
                let upstream_tag = self
                    .deployment_manager
                    .get_upstream_tag(&hash)
                    .unwrap_or_else(|e| {
                        warn!("Failed to get upstream tag for {}: {}", hash, e);
                        "<unknown tag>".to_string()
                    });
                info!(
                    "New version of `{}` tag({}) is available: `{}`",
                    SOURCE_TAG, upstream_tag, hash
                );
                match self.redeploy(&hash) {
                    Err(e) => {
                        self.report_failure(format!("Failed to deploy `{}`: {}", hash, e));
                        return;
                    }
                    Ok(()) => {
                        info!("Deployed new version `{}`, running test suite", hash);
                        hash_to_tag = Some(hash);
                    }
                }
                if let Err(e) = self.run_ci_suite(hash_to_tag) {
                    self.report_failure(format!("{}", e));
                    return;
                }
            }
            thread::sleep(Duration::from_secs(60));
        }
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

    fn report_failure(&self, msg: String) {
        self.slack_message(msg);
    }

    fn redeploy(&mut self, hash: &str) -> failure::Result<()> {
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
        let marker = self
            .deployment_manager
            .get_upstream_tag(hash)
            .map_err(|e| format_err!("Failed to get upstream tag: {}", e))?;
        self.fetch_genesis(&marker)?;
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

    fn fetch_genesis(&self, marker: &str) -> failure::Result<()> {
        let cmd = format!(
            "sudo aws s3 cp s3://toro-validator-sets/{}/100/genesis.blob /opt/libra/genesis.blob",
            marker
        );
        info!("Running {} to fetch genesis blob", cmd);
        let jobs = self
            .cluster
            .instances()
            .iter()
            .map(|instance| {
                let cmd = &cmd;
                move || instance.run_cmd_tee_err(vec![cmd])
            })
            .collect();
        if self
            .thread_pool_executor
            .execute_jobs(jobs)
            .iter()
            .any(Result::is_err)
        {
            return Err(format_err!(
                "Failed to update genesis.blob on one of validators"
            ));
        }
        Ok(())
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

    pub fn perf_run(&mut self) {
        let results = self.measure_performance().unwrap();
        println!("{}", results.to_slack_message())
    }

    fn measure_performance(&mut self) -> failure::Result<SuiteReport> {
        info!("Starting warm up job");
        self.emit_txn_for(Duration::from_secs(60), self.cluster.instances().clone())?;
        info!("Warm up done, measuring tps");
        let window = Duration::from_secs(180);
        self.emit_txn_for(
            window + Duration::from_secs(30),
            self.cluster.instances().clone(),
        )?;
        let stats_all_up = print_stat(&self.prometheus, window)
            .map_err(|e| format_err!("Failed to query stats: {}", e))?;
        let (stop, keep) = self.cluster.split_n_random(10);
        let mut stop_effects: Vec<_> = stop
            .into_instances()
            .into_iter()
            .map(StopContainer::new)
            .collect();
        self.activate_all(&mut stop_effects);
        self.emit_txn_for(window + Duration::from_secs(30), keep.instances().clone())?;
        let stats_10_down = print_stat(&self.prometheus, window)
            .map_err(|e| format_err!("Failed to query stats: {}", e))?;
        self.deactivate_all(&mut stop_effects);
        self.wait_until_all_healthy()?;
        Ok(SuiteReport {
            stats_all_up,
            stats_10_down,
        })
    }

    fn emit_txn_for(
        &mut self,
        duration: Duration,
        instances: Vec<Instance>,
    ) -> failure::Result<()> {
        let job = self.tx_emitter.start_job(EmitJobRequest {
            instances,
            accounts_per_client: 10,
            thread_params: EmitThreadParams::default(),
        })?;
        thread::sleep(duration);
        self.tx_emitter.stop_job(job);
        Ok(())
    }

    pub fn cleanup_and_run(&mut self, experiment: Box<dyn Experiment>) -> failure::Result<()> {
        self.cleanup();
        self.run_single_experiment(experiment)
    }

    pub fn run_single_experiment(
        &mut self,
        experiment: Box<dyn Experiment>,
    ) -> failure::Result<()> {
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
        let (exp_result_sender, exp_result_recv) = mpsc::channel();
        thread::spawn(move || {
            let result = experiment.run();
            exp_result_sender
                .send(result)
                .expect("Failed to send experiment result");
        });

        // We expect experiments completes and cluster go into healthy state within timeout
        let experiment_deadline = Instant::now() + deadline;

        loop {
            if Instant::now() > experiment_deadline {
                bail!("Experiment did not complete in time");
            }
            let deadline = Instant::now() + HEALTH_POLL_INTERVAL;
            // Receive all events that arrived to aws log tail within next 1 second
            // This assumes so far that event propagation time is << 1s, this need to be refined
            // in future to account for actual event propagation delay
            let events = self.logs.recv_all_until_deadline(deadline);
            if let Err(s) = self.health_check_runner.run(
                &events,
                &affected_validators,
                PrintFailures::UnexpectedOnly,
            ) {
                bail!("Validators which were not under experiment failed : {}", s);
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

    fn wait_until_all_healthy(&mut self) -> failure::Result<()> {
        let wait_deadline = Instant::now() + Duration::from_secs(20 * 60);
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

    fn slack_message(&self, msg: String) {
        info!("{}", msg);
        if let Some(ref log_url) = self.slack_log_url {
            if let Err(e) = self.slack.send_message(log_url, &msg) {
                info!("Failed to send slack message: {}", e);
            }
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

    fn wipe_all_db(&self, safety_wait: bool) {
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
            .instances()
            .iter()
            .map(|instance| {
                let instance = instance.clone();
                move || {
                    instance
                        .run_cmd_tee_err(vec!["sudo", "rm", "-rf", "/data/libra/*db"])
                        .map_err(|e| info!("Failed to wipe {}: {:?}", instance, e))
                        .ok();
                    instance
                        .run_cmd_tee_err(vec![format!(
                            "test -f {f} && sudo gzip -S {s} {f}",
                            f = log_file,
                            s = suffix
                        )])
                        .map_err(|e| info!("Failed to gzip log file {}: {:?}", instance, e))
                        .ok();
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(jobs);
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

    fn cleanup(&self) {
        let cleanup_all_instances: Vec<_> = self
            .cluster
            .instances()
            .clone()
            .into_iter()
            .map(|instance| {
                move || {
                    if let Err(e) = RemoveNetworkEffects::new(instance.clone()).apply() {
                        info!(
                            "Failed to remove network effects for {}. Error: {}",
                            instance, e
                        );
                    }
                }
            })
            .collect();
        self.thread_pool_executor
            .execute_jobs(cleanup_all_instances);
    }

    pub fn stop(&self) {
        self.activate_all(&mut self.make_stop_effects())
    }

    pub fn start(&self) {
        self.deactivate_all(&mut self.make_stop_effects())
    }

    fn make_stop_effects(&self) -> Vec<StopContainer> {
        self.cluster
            .instances()
            .clone()
            .into_iter()
            .map(StopContainer::new)
            .collect()
    }

    fn activate_all<T: Effect>(&self, effects: &mut [T]) {
        let jobs = effects
            .iter_mut()
            .map(|effect| {
                move || {
                    if let Err(e) = effect.activate() {
                        info!("Failed to activate {}: {:?}", effect, e);
                    }
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(jobs);
    }

    fn deactivate_all<T: Effect>(&self, effects: &mut [T]) {
        let jobs = effects
            .iter_mut()
            .map(|effect| {
                move || {
                    if let Err(e) = effect.deactivate() {
                        info!("Failed to deactivate {}: {:?}", effect, e);
                    }
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(jobs);
    }
}

struct SuiteReport {
    stats_all_up: (f64, f64),
    stats_10_down: (f64, f64),
}

impl SuiteReport {
    pub fn to_slack_message(&self) -> String {
        format!(
            "all up: {:.0} TPS, {:.1} s latency\n10% down: {:.0} TPS, {:.1} s latency",
            self.stats_all_up.0, self.stats_all_up.1, self.stats_10_down.0, self.stats_10_down.1,
        )
    }
}
