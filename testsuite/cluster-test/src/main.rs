// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashSet,
    env, process, thread,
    time::{Duration, Instant},
};

use libra_logger::info;
use reqwest::Url;
use structopt::{clap::ArgGroup, StructOpt};
use termion::{color, style};

use anyhow::{bail, format_err, Result};
use cluster_test::{
    aws,
    cluster::Cluster,
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    experiments::{get_experiment, Context, Experiment},
    github::GitHub,
    health::{DebugPortLogThread, HealthCheckRunner, LogTail, PrintFailures, TraceTail},
    prometheus::Prometheus,
    report::SuiteReport,
    slack::SlackClient,
    stats,
    suite::ExperimentSuite,
    tx_emitter::{AccountData, EmitJobRequest, EmitThreadParams, TxEmitter, TxStats},
    util::unix_timestamp_now,
};
use futures::{
    future::{join_all, FutureExt},
    select,
};
use itertools::zip;
use libra_config::config::DEFAULT_JSON_RPC_PORT;
use std::cmp::min;
use tokio::{
    runtime::{Builder, Runtime},
    time::{delay_for, delay_until, Instant as TokioInstant},
};

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
    run: Option<String>,
    #[structopt(long, group = "action")]
    health_check: bool,
    #[structopt(long, group = "action")]
    emit_tx: bool,
    #[structopt(long, group = "action", requires = "swarm")]
    diag: bool,
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
    #[structopt(long, default_value = "15")]
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

    #[structopt(
        long,
        help = "Whether transactions should be submitted to validators or full nodes"
    )]
    pub emit_to_validator: Option<bool>,

    #[structopt(long, default_value = "1")]
    pub k8s_fullnodes_per_validator: u32,
    #[structopt(long, parse(try_from_str), default_value = "30")]
    pub k8s_num_validators: u32,
}

pub fn main() {
    setup_log();

    let args = Args::from_args();

    if args.swarm && !(args.emit_tx || args.diag) {
        panic!("Can only use --emit-tx or --diag in --swarm mode");
    }

    if args.diag {
        let util = BasicSwarmUtil::setup(&args);
        let mut rt = Runtime::new().unwrap();
        exit_on_error(rt.block_on(util.diag()));
        return;
    } else if args.emit_tx {
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
    }

    let mut runner = ClusterTestRunner::setup(&args);

    let mut perf_msg = None;

    if args.health_check {
        runner.run_health_check();
    } else if args.perf_run {
        perf_msg = Some(runner.perf_run());
    } else if args.cleanup {
        runner.cleanup();
    } else if args.run_ci_suite {
        perf_msg = Some(exit_on_error(runner.run_ci_suite()));
    } else if let Some(experiment_name) = args.run {
        let result = runner.cleanup_and_run(get_experiment(
            &experiment_name,
            &args.last,
            &runner.cluster,
        ));
        runner.cleanup();
        runner.teardown();

        result.unwrap();
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
    ::libra_logger::Logger::new().is_async(true).init();
}

struct BasicSwarmUtil {
    cluster: Cluster,
}

struct ClusterUtil {
    cluster: Cluster,
    prometheus: Prometheus,
    cluster_swarm: ClusterSwarmKube,
}

struct ClusterTestRunner {
    logs: LogTail,
    trace_tail: TraceTail,
    cluster: Cluster,
    health_check_runner: HealthCheckRunner,
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
    cluster_swarm: ClusterSwarmKube,
}

fn parse_host_port(s: &str) -> Result<(String, u32)> {
    let v = s.split(':').collect::<Vec<&str>>();
    if v.len() == 1 {
        let default_port = DEFAULT_JSON_RPC_PORT as u32;
        return Ok((v[0].to_string(), default_port));
    }
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

    pub async fn diag(&self) -> Result<()> {
        let emitter = TxEmitter::new(&self.cluster);
        let mut faucet_account: Option<AccountData> = None;
        let instances: Vec<_> = self.cluster.all_instances().collect();
        for instance in &instances {
            print!("Getting faucet account sequence number on {}...", instance);
            let account = emitter
                .load_faucet_account(instance)
                .await
                .map_err(|e| format_err!("Failed to get faucet account sequence number: {}", e))?;
            println!("seq={}", account.sequence_number);
            if let Some(faucet_account) = &faucet_account {
                if account.sequence_number != faucet_account.sequence_number {
                    bail!(
                        "Loaded sequence number {}, which is different from seen before {}",
                        account.sequence_number,
                        faucet_account.sequence_number
                    );
                }
            } else {
                faucet_account = Some(account);
            }
        }
        let mut faucet_account = faucet_account.unwrap();
        let faucet_account_address = faucet_account.address;
        for instance in &instances {
            print!("Submitting txn through {}...", instance);
            let deadline = emitter
                .submit_single_transaction(instance, &mut faucet_account)
                .await
                .map_err(|e| format_err!("Failed to submit txn through {}: {}", instance, e))?;
            println!("seq={}", faucet_account.sequence_number);
            println!(
                "Waiting all full nodes to get to seq {}",
                faucet_account.sequence_number
            );
            loop {
                let futures = instances.iter().map(|instance| {
                    emitter.query_sequence_numbers(instance, &faucet_account_address)
                });
                let results = join_all(futures).await;
                let mut all_good = true;
                for (instance, result) in zip(instances.iter(), results) {
                    let seq = result.map_err(|e| {
                        format_err!("Failed to query sequence number from {}: {}", instance, e)
                    })?;
                    let ip = instance.ip();
                    let color = if seq != faucet_account.sequence_number {
                        all_good = false;
                        color::Fg(color::Red).to_string()
                    } else {
                        color::Fg(color::Green).to_string()
                    };
                    print!(
                        "[{}{}:{}{}]  ",
                        color,
                        &ip[..min(ip.len(), 10)],
                        seq,
                        color::Fg(color::Reset)
                    );
                }
                println!();
                if all_good {
                    break;
                }
                if Instant::now() > deadline {
                    bail!("Not all full nodes were updated and transaction expired");
                }
                tokio::time::delay_for(Duration::from_secs(1)).await;
            }
        }
        println!("Looks like all full nodes are healthy!");
        Ok(())
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
        let deadline = Instant::now() + duration;
        let mut prev_stats: Option<TxStats> = None;
        while Instant::now() < deadline {
            let window = Duration::from_secs(10);
            tokio::time::delay_for(window).await;
            let stats = emitter.peek_job_stats(&job);
            let delta = &stats - &prev_stats.unwrap_or_default();
            prev_stats = Some(stats);
            println!("{}", delta.rate(window));
        }
        let stats = emitter.stop_job(job).await;
        println!("Total stats: {}", stats);
        println!("Average rate: {}", stats.rate(duration));
    }
}

impl ClusterUtil {
    pub fn setup(args: &Args) -> Self {
        Runtime::new().unwrap().block_on(async move {
            let cluster_swarm = ClusterSwarmKube::new()
                .await
                .expect("Failed to initialize ClusterSwarmKube");
            cluster_swarm.delete_all().await.expect("delete_all failed");
            let image_tag = args.deploy.as_deref().unwrap_or("master");
            info!(
                "Deploying with {} tag for validators and fullnodes",
                image_tag
            );
            let asg_name = format!(
                "{}-k8s-testnet-validators",
                cluster_swarm
                    .get_workspace()
                    .await
                    .expect("Failed to get workspace")
            );
            aws::set_asg_size(
                (args.k8s_num_validators
                    + (args.k8s_fullnodes_per_validator * args.k8s_num_validators))
                    as i64,
                5.0,
                &asg_name,
                true,
            )
            .await
            .unwrap_or_else(|_| panic!("{} scaling failed", asg_name));
            cluster_swarm
                .create_validator_and_fullnode_set(
                    args.k8s_num_validators,
                    args.k8s_fullnodes_per_validator,
                    &image_tag,
                    true,
                )
                .await
                .expect("Failed to create_validator_and_fullnode_set");
            info!("Deployment complete");
            let cluster = Cluster::new_k8s(
                cluster_swarm.validator_instances().await,
                cluster_swarm.fullnode_instances().await,
            )
            .unwrap();

            let cluster = if args.peers.is_empty() {
                cluster
            } else {
                cluster.validator_sub_cluster(args.peers.clone())
            };
            let prometheus_ip = "libra-testnet-prometheus-server.default.svc.cluster.local";
            let grafana_base_url = cluster_swarm
                .get_grafana_baseurl()
                .await
                .expect("Failed to discover grafana url in k8s");
            let prometheus = Prometheus::new(prometheus_ip, grafana_base_url);
            info!(
                "Discovered {} validators and {} fns",
                cluster.validator_instances().len(),
                cluster.fullnode_instances().len(),
            );
            Self {
                cluster,
                prometheus,
                cluster_swarm,
            }
        })
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
    pub fn teardown(&mut self) {
        let workspace = self
            .runtime
            .block_on(self.cluster_swarm.get_workspace())
            .expect("Failed to get workspace");
        let asg_name = format!("{}-k8s-testnet-validators", workspace);
        self.runtime
            .block_on(aws::set_asg_size(0, 0.0, &asg_name, false))
            .unwrap_or_else(|_| panic!("{} scaling failed", asg_name));
    }

    /// Discovers cluster, setup log, etc
    pub fn setup(args: &Args) -> Self {
        let util = ClusterUtil::setup(args);
        let cluster = util.cluster;
        let cluster_swarm = util.cluster_swarm;
        let log_tail_started = Instant::now();
        let (logs, trace_tail) = DebugPortLogThread::spawn_new(&cluster);
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
        let slack = SlackClient::new();
        let slack_changelog_url = env::var("SLACK_CHANGELOG_URL")
            .map(|u| u.parse().expect("Failed to parse SLACK_CHANGELOG_URL"))
            .ok();
        let tx_emitter = TxEmitter::new(&cluster);
        let prometheus = util.prometheus;
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
            trace_tail,
            cluster,
            health_check_runner,
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
            cluster_swarm,
        }
    }

    pub fn run_ci_suite(&mut self) -> Result<String> {
        let suite = ExperimentSuite::new_pre_release(&self.cluster);
        let result = self.run_suite(suite);
        self.runtime.block_on(self.cluster_swarm.delete_all())?;
        result?;
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

    fn run_suite(&mut self, suite: ExperimentSuite) -> Result<()> {
        info!("Starting suite");
        let suite_started = Instant::now();
        for experiment in suite.experiments {
            let experiment_name = format!("{}", experiment);
            let result = self
                .run_single_experiment(experiment, None)
                .map_err(move |e| format_err!("Experiment `{}` failed: `{}`", experiment_name, e));
            if result.is_err() {
                self.teardown();
                return result;
            }
            thread::sleep(self.experiment_interval);
        }
        info!(
            "Suite completed in {:?}",
            Instant::now().duration_since(suite_started)
        );
        self.print_report();
        self.teardown();
        Ok(())
    }

    pub fn print_report(&self) {
        let json_report =
            serde_json::to_string_pretty(&self.report).expect("Failed to serialize report to json");
        info!(
            "\n====json-report-begin===\n{}\n====json-report-end===",
            json_report
        );
    }

    pub fn perf_run(&mut self) -> String {
        let suite = ExperimentSuite::new_perf_suite(&self.cluster);
        self.run_suite(suite).unwrap();
        self.report.to_string()
    }

    pub fn cleanup_and_run(&mut self, experiment: Box<dyn Experiment>) -> Result<()> {
        self.cleanup();
        let result =
            self.run_single_experiment(experiment, Some(self.global_emit_job_request.clone()));
        self.runtime.block_on(self.cluster_swarm.delete_all())?;
        result?;
        self.print_report();
        Ok(())
    }

    pub fn run_single_experiment(
        &mut self,
        mut experiment: Box<dyn Experiment>,
        mut global_emit_job_request: Option<EmitJobRequest>,
    ) -> Result<()> {
        self.wait_until_all_healthy()?;
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
            &mut self.trace_tail,
            &self.prometheus,
            &self.cluster,
            &mut self.report,
            &mut global_emit_job_request,
            self.emit_to_validator,
            &self.cluster_swarm,
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
        info!("Waiting for all validators to be healthy");
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
        info!("All validators are now healthy");
        Ok(())
    }

    fn slack_changelog_message(&self, msg: String) {
        info!("{}", msg);
        if let Some(ref changelog_url) = self.slack_changelog_url {
            if let Err(e) = self.slack.send_message(changelog_url, &msg) {
                info!("Failed to send slack message: {}", e);
            }
        }
    }

    fn cleanup(&mut self) {
        self.runtime
            .block_on(self.cluster_swarm.remove_all_network_effects())
            .expect("remove_all_network_effects failed on cluster_swarm");
    }
}
