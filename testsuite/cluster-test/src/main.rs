// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashSet,
    env, process,
    time::{Duration, Instant},
};

use libra_logger::{info, warn};
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
    health::{DebugPortLogWorker, HealthCheckRunner, LogTail, PrintFailures, TraceTail},
    instance::Instance,
    prometheus::Prometheus,
    report::SuiteReport,
    slack::SlackClient,
    suite::ExperimentSuite,
    tx_emitter::{AccountData, EmitJobRequest, EmitThreadParams, TxEmitter, TxStats},
};
use futures::{
    future::{join_all, FutureExt},
    select,
};
use itertools::zip;
use libra_config::config::DEFAULT_JSON_RPC_PORT;
use std::cmp::min;
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
    #[structopt(long)]
    pub enable_lsr: bool,
    #[structopt(
        long,
        help = "Backend used by lsr. Possible Values are in-memory, on-disk, vault",
        default_value = "vault"
    )]
    pub lsr_backend: String,
}

#[tokio::main]
pub async fn main() {
    setup_log();

    let args = Args::from_args();

    if args.swarm && !(args.emit_tx || args.diag || args.health_check) {
        panic!("Can only use --emit-tx or --diag or --health-check in --swarm mode");
    }

    if args.diag {
        let util = BasicSwarmUtil::setup(&args);
        exit_on_error(util.diag().await);
        return;
    } else if args.emit_tx {
        let thread_params = EmitThreadParams {
            wait_millis: args.wait_millis,
            wait_committed: !args.burst,
        };
        let duration = Duration::from_secs(args.duration);
        if args.swarm {
            let util = BasicSwarmUtil::setup(&args);
            emit_tx(
                &util.cluster,
                args.accounts_per_client,
                args.workers_per_ac,
                thread_params,
                duration,
            )
            .await;
            return;
        } else {
            let util = ClusterUtil::setup(&args).await;
            emit_tx(
                &util.cluster,
                args.accounts_per_client,
                args.workers_per_ac,
                thread_params,
                duration,
            )
            .await;
            return;
        }
    } else if args.health_check && args.swarm {
        let util = BasicSwarmUtil::setup(&args);
        let logs = DebugPortLogWorker::spawn_new(&util.cluster).0;
        let mut health_check_runner = HealthCheckRunner::new_all(util.cluster);
        let duration = Duration::from_secs(args.duration);
        exit_on_error(run_health_check(&logs, &mut health_check_runner, duration));
        return;
    }

    let mut runner = ClusterTestRunner::setup(&args).await;

    let mut perf_msg = None;

    if args.health_check {
        let duration = Duration::from_secs(args.duration);
        exit_on_error(run_health_check(
            &runner.logs,
            &mut runner.health_check_runner,
            duration,
        ));
    } else if args.perf_run {
        perf_msg = Some(exit_on_error(runner.perf_run().await));
    } else if args.cleanup {
        runner.cleanup().await;
    } else if args.run_ci_suite {
        perf_msg = Some(exit_on_error(runner.run_ci_suite().await));
    } else if let Some(experiment_name) = args.run {
        let result = runner
            .cleanup_and_run(get_experiment(
                &experiment_name,
                &args.last,
                &runner.cluster,
            ))
            .await;
        runner.cleanup().await;
        runner.teardown().await;

        exit_on_error(result);
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
    current_tag: String,
}

struct ClusterTestRunner {
    logs: LogTail,
    trace_tail: TraceTail,
    cluster: Cluster,
    health_check_runner: HealthCheckRunner,
    experiment_interval: Duration,
    slack: SlackClient,
    slack_changelog_url: Option<Url>,
    tx_emitter: TxEmitter,
    prometheus: Prometheus,
    github: GitHub,
    report: SuiteReport,
    global_emit_job_request: EmitJobRequest,
    emit_to_validator: bool,
    cluster_swarm: ClusterSwarmKube,
    current_tag: String,
}

fn parse_host_port(s: &str) -> Result<(String, u32, Option<u32>)> {
    let v = s.split(':').collect::<Vec<&str>>();
    if v.len() == 1 {
        let default_port = DEFAULT_JSON_RPC_PORT as u32;
        return Ok((v[0].to_string(), default_port, None));
    }
    if v.len() != 2 && v.len() != 3 {
        return Err(format_err!(
            "Failed to parse {:?} in host:port or host:port:debug_interface_port format",
            s
        ));
    }
    let host = v[0].to_string();
    let port = v[1].parse::<u32>()?;
    if v.len() == 3 {
        let debug_interface_port = v[2].parse::<u32>()?;
        return Ok((host, port, Some(debug_interface_port)));
    }
    Ok((host, port, None))
}

pub async fn emit_tx(
    cluster: &Cluster,
    accounts_per_client: usize,
    workers_per_ac: Option<usize>,
    thread_params: EmitThreadParams,
    duration: Duration,
) {
    let mut emitter = TxEmitter::new(cluster);
    let job = emitter
        .start_job(EmitJobRequest {
            instances: cluster.validator_instances().to_vec(),
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

fn run_health_check(
    logs: &LogTail,
    health_check_runner: &mut HealthCheckRunner,
    duration: Duration,
) -> Result<()> {
    let health_check_deadline = Instant::now() + duration;
    loop {
        let deadline = Instant::now() + Duration::from_secs(1);
        // Receive all events that arrived to log tail within next 1 second
        // This assumes so far that event propagation time is << 1s, this need to be refined
        // in future to account for actual event propagation delay
        let events = logs.recv_all_until_deadline(deadline);
        let result = health_check_runner.run(&events, &HashSet::new(), PrintFailures::All);
        let now = Instant::now();
        if now > health_check_deadline {
            return result.map(|_| ());
        }
    }
}

impl BasicSwarmUtil {
    pub fn setup(args: &Args) -> Self {
        if args.peers.is_empty() {
            panic!("Peers not set in args");
        }
        let parsed_peers: Vec<_> = args
            .peers
            .iter()
            .map(|peer| parse_host_port(peer).expect("Failed to parse host_port"))
            .collect();

        let cluster = Cluster::from_host_port(parsed_peers, &args.mint_file);
        Self { cluster }
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
        let mut faucet_account =
            faucet_account.expect("There is no faucet account set (not expected)");
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
}

impl ClusterUtil {
    pub async fn setup(args: &Args) -> Self {
        let cluster_swarm = ClusterSwarmKube::new()
            .await
            .expect("Failed to initialize ClusterSwarmKube");
        cluster_swarm.delete_all().await.expect("delete_all failed");
        let current_tag = args.deploy.as_deref().unwrap_or("master");
        info!(
            "Deploying with {} tag for validators and fullnodes",
            current_tag
        );
        let asg_name = format!(
            "{}-k8s-testnet-validators",
            cluster_swarm
                .get_workspace()
                .await
                .expect("Failed to get workspace")
        );
        let mut instance_count =
            args.k8s_num_validators + (args.k8s_fullnodes_per_validator * args.k8s_num_validators);
        if args.enable_lsr {
            if args.lsr_backend == "vault" {
                instance_count += args.k8s_num_validators * 2;
            } else {
                instance_count += args.k8s_num_validators;
            }
        }
        aws::set_asg_size(instance_count as i64, 5.0, &asg_name, true)
            .await
            .unwrap_or_else(|_| panic!("{} scaling failed", asg_name));
        let (validators, fullnodes) = cluster_swarm
            .spawn_validator_and_fullnode_set(
                args.k8s_num_validators,
                args.k8s_fullnodes_per_validator,
                args.enable_lsr,
                &args.lsr_backend,
                current_tag,
            )
            .await
            .expect("Failed to spawn_validator_and_fullnode_set");
        info!("Deployment complete");
        let cluster = Cluster::new(validators, fullnodes);

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
            current_tag: current_tag.to_string(),
        }
    }
}

impl ClusterTestRunner {
    pub async fn teardown(&mut self) {
        let workspace = self
            .cluster_swarm
            .get_workspace()
            .await
            .expect("Failed to get workspace");
        let asg_name = format!("{}-k8s-testnet-validators", workspace);
        aws::set_asg_size(0, 0.0, &asg_name, false)
            .await
            .unwrap_or_else(|_| panic!("{} scaling failed", asg_name));
    }

    /// Discovers cluster, setup log, etc
    pub async fn setup(args: &Args) -> Self {
        let util = ClusterUtil::setup(args).await;
        let cluster = util.cluster;
        let cluster_swarm = util.cluster_swarm;
        let current_tag = util.current_tag;
        let log_tail_started = Instant::now();
        let (logs, trace_tail) = DebugPortLogWorker::spawn_new(&cluster);
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
            slack_changelog_url,
            tx_emitter,
            prometheus,
            github,
            report,
            global_emit_job_request,
            emit_to_validator,
            cluster_swarm,
            current_tag,
        }
    }

    pub async fn run_ci_suite(&mut self) -> Result<String> {
        let suite = ExperimentSuite::new_pre_release(&self.cluster);
        let result = self.run_suite(suite).await;
        self.cluster_swarm.delete_all().await?;
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

    async fn run_suite(&mut self, suite: ExperimentSuite) -> Result<()> {
        info!("Starting suite");
        let suite_started = Instant::now();
        for experiment in suite.experiments {
            let experiment_name = format!("{}", experiment);
            let result = self
                .run_single_experiment(experiment, None)
                .await
                .map_err(move |e| format_err!("Experiment `{}` failed: `{}`", experiment_name, e));
            if result.is_err() {
                self.teardown().await;
                return result;
            }
            delay_for(self.experiment_interval).await;
        }
        info!(
            "Suite completed in {:?}",
            Instant::now().duration_since(suite_started)
        );
        self.print_report();
        self.teardown().await;
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

    pub async fn perf_run(&mut self) -> Result<String> {
        let suite = ExperimentSuite::new_perf_suite(&self.cluster);
        self.run_suite(suite).await?;
        Ok(self.report.to_string())
    }

    pub async fn cleanup_and_run(&mut self, experiment: Box<dyn Experiment>) -> Result<()> {
        self.cleanup().await;
        let result = self
            .run_single_experiment(experiment, Some(self.global_emit_job_request.clone()))
            .await;
        self.cluster_swarm.delete_all().await?;
        result?;
        self.print_report();
        Ok(())
    }

    pub async fn run_single_experiment(
        &mut self,
        experiment: Box<dyn Experiment>,
        global_emit_job_request: Option<EmitJobRequest>,
    ) -> Result<()> {
        self.wait_until_all_healthy().await?;
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

        self.experiment_loop(experiment, global_emit_job_request)
            .await?;

        info!(
            "{}Experiment finished, waiting until all affected validators recover{}",
            style::Bold,
            style::Reset
        );

        self.wait_until_all_healthy().await?;

        info!("Experiment completed");
        Ok(())
    }

    // inner poll loop of run_single_experiment
    // do not use this fn, use run_single_experiment to run experiments
    async fn experiment_loop(
        &mut self,
        mut experiment: Box<dyn Experiment>,
        mut global_emit_job_request: Option<EmitJobRequest>,
    ) -> Result<()> {
        let affected_validators = experiment.affected_validators();
        let deadline = experiment.deadline();
        let experiment_deadline = Instant::now() + deadline;
        let mut context = Context::new(
            &mut self.tx_emitter,
            &mut self.trace_tail,
            &self.prometheus,
            &self.cluster,
            &mut self.report,
            &mut global_emit_job_request,
            self.emit_to_validator,
            &self.cluster_swarm,
            &self.current_tag[..],
        );
        let mut deadline_future = delay_until(TokioInstant::from_std(experiment_deadline)).fuse();
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
                    let events = self.logs.recv_all();
                    if let Err(s) = self.health_check_runner.run(
                        &events,
                        &affected_validators,
                        PrintFailures::UnexpectedOnly,
                    ) {
                        bail!("Validators which were not under experiment failed : {}", s);
                    }
                }
            }
        }
    }

    async fn wait_until_all_healthy(&mut self) -> Result<()> {
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
        info!("All validators are now healthy. Checking json rpc endpoints of validators and full nodes");
        loop {
            let results = join_all(self.cluster.all_instances().map(Instance::try_json_rpc)).await;

            if results.iter().all(Result::is_ok) {
                break;
            }
            if Instant::now() > wait_deadline {
                for (instance, result) in zip(self.cluster.all_instances(), results) {
                    if let Err(err) = result {
                        warn!("Instance {} still unhealthy: {}", instance, err);
                    }
                }
                bail!("Some json rpc endpoints did not become healthy after deployment");
            }
        }
        info!("All json rpc endpoints are healthy");
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

    async fn cleanup(&mut self) {
        self.cluster_swarm
            .remove_all_network_effects()
            .await
            .expect("remove_all_network_effects failed on cluster_swarm");
    }
}
