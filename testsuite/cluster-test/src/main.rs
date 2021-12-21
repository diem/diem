// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashSet,
    env, fmt, process,
    time::{Duration, Instant},
};

use diem_logger::{info, warn};
use diem_types::chain_id::ChainId;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use reqwest::Url;
use structopt::{clap::ArgGroup, StructOpt};
use termion::{color, style};

use anyhow::{bail, format_err, Result};
use cluster_test::{
    aws,
    cluster::Cluster,
    cluster_builder::{ClusterBuilder, ClusterBuilderParams},
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    experiments::{get_experiment, Context, Experiment},
    health::{DebugPortLogWorker, HealthCheckRunner, LogTail, PrintFailures},
    instance::Instance,
    prometheus::Prometheus,
    report::SuiteReport,
    suite::ExperimentSuite,
};
use diem_config::config::DEFAULT_JSON_RPC_PORT;
use diem_sdk::{transaction_builder::TransactionFactory, types::LocalAccount};
use forge::{
    query_sequence_numbers, EmitJobRequest, EmitThreadParams, GitHub, SlackClient, TxnEmitter,
};
use futures::{
    future::{join_all, FutureExt},
    select,
};
use itertools::zip;
use std::cmp::min;
use tokio::time::{sleep, sleep_until, Instant as TokioInstant};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("action"))]
struct Args {
    #[structopt(short = "p", long, use_delimiter = true, requires = "swarm")]
    peers: Vec<String>,

    #[structopt(long, help = "If set, tries to connect to a diem-swarm instead of aws")]
    swarm: bool,
    #[structopt(long, help = "If set, tries to use public peers instead of localhost")]
    vasp: bool,

    #[structopt(long, group = "action")]
    run: Option<String>,
    #[structopt(long, group = "action")]
    health_check: bool,
    #[structopt(long, group = "action")]
    emit_tx: bool,
    #[structopt(long, group = "action", requires = "swarm")]
    diag: bool,
    #[structopt(long, group = "action")]
    no_teardown: bool,
    #[structopt(long, group = "action")]
    suite: Option<String>,
    #[structopt(long, group = "action")]
    exec: Option<String>,

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
    #[structopt(long, default_value = "TESTING")]
    chain_id: ChainId,
    #[structopt(
        long,
        help = "Time to run --emit-tx for in seconds",
        default_value = "60"
    )]
    duration: u64,
    #[structopt(long, help = "Percentage of invalid txs", default_value = "0")]
    invalid_tx: usize,

    #[structopt(
        long,
        help = "Whether transactions should be submitted to validators or full nodes"
    )]
    pub emit_to_validator: Option<bool>,

    #[structopt(
        long,
        help = "Wait for given number of seconds if experiment fails. This require experiment to return error, it does not catch panics"
    )]
    pub wait_on_failure: Option<u64>,

    #[structopt(flatten)]
    pub cluster_builder_params: ClusterBuilderParams,
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
        exit_on_error(util.diag(args.vasp).await);
        return;
    } else if args.emit_tx && args.swarm {
        let util = BasicSwarmUtil::setup(&args);
        exit_on_error(emit_tx(&util.cluster, &args).await);
        return;
    } else if args.health_check && args.swarm {
        let util = BasicSwarmUtil::setup(&args);
        let logs = DebugPortLogWorker::spawn_new(&util.cluster);
        let mut health_check_runner = HealthCheckRunner::new_all(util.cluster);
        let duration = Duration::from_secs(args.duration);
        exit_on_error(run_health_check(&logs, &mut health_check_runner, duration).await);
        return;
    }

    let wait_on_failure = if let Some(wait_on_failure) = args.wait_on_failure {
        if wait_on_failure > 20 * 60 {
            println!("wait_on_failure can not be more then 1200 seconds on shared cluster");
            process::exit(1);
        }
        Some(Duration::from_secs(wait_on_failure))
    } else {
        None
    };

    let runner = ClusterTestRunner::setup(&args).await;
    let mut runner = match runner {
        Ok(r) => r,
        Err(e) => {
            if let Some(wait_on_failure) = wait_on_failure {
                warn!(
                    "Setting up runner failed with {}, waiting for {:?} before terminating",
                    e, wait_on_failure
                );
                sleep(wait_on_failure).await;
            }
            panic!("Failed to setup cluster test runner: {}", e);
        }
    };

    let result = handle_cluster_test_runner_commands(&args, &mut runner).await;
    if let Err(e) = &result {
        if let Some(wait_on_failure) = wait_on_failure {
            warn!(
                "Command failed with {}, waiting for {:?} before terminating",
                e, wait_on_failure
            );
            sleep(wait_on_failure).await;
            warn!("Tearing down cluster now");
        }
    }
    if !args.no_teardown {
        runner.teardown().await;
    }
    let perf_msg = exit_on_error(result);

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

// This function contain handlers for commands that require cluster running for executing them
async fn handle_cluster_test_runner_commands(
    args: &Args,
    runner: &mut ClusterTestRunner,
) -> Result<Option<String>> {
    let startup_timeout = Duration::from_secs(5 * 60);
    runner
        .wait_until_all_healthy(Instant::now() + startup_timeout)
        .await
        .map_err(|err| {
            runner
                .report
                .report_text(format!("Cluster setup failed: `{}`", err));
            runner.print_report();
            err
        })?;
    let mut perf_msg = None;
    if args.health_check {
        let duration = Duration::from_secs(args.duration);
        run_health_check(&runner.logs, &mut runner.health_check_runner, duration).await?
    } else if let Some(suite) = args.suite.as_ref() {
        perf_msg = Some(runner.run_named_suite(suite).await?);
    } else if let Some(experiment_name) = args.run.as_ref() {
        runner
            .run_and_report(get_experiment(experiment_name, &args.last, &runner.cluster))
            .await?;
        info!(
            "{}Experiment Result: {}{}",
            Bold {},
            runner.report,
            Reset {}
        );
    } else if args.emit_tx {
        emit_tx(&runner.cluster, args).await?;
    } else if let Some(ref exec) = args.exec {
        let pos = exec.find(':');
        let pos = pos.ok_or_else(|| {
            format_err!("Format for exec command is pod:command, for example val-1:date")
        })?;
        let (pod, cmd) = exec.split_at(pos);
        let cmd = &cmd[1..];
        runner.exec_on_pod(pod, cmd).await?;
    }
    Ok(perf_msg)
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
    ::diem_logger::Logger::new().is_async(true).init();
}

struct BasicSwarmUtil {
    cluster: Cluster,
}

struct ClusterTestRunner {
    logs: LogTail,
    cluster_builder: ClusterBuilder,
    cluster_builder_params: ClusterBuilderParams,
    cluster: Cluster,
    health_check_runner: HealthCheckRunner,
    slack: SlackClient,
    slack_changelog_url: Option<Url>,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
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

async fn emit_tx(cluster: &Cluster, args: &Args) -> Result<()> {
    let thread_params = EmitThreadParams {
        wait_millis: args.wait_millis,
        wait_committed: !args.burst,
    };
    let duration = Duration::from_secs(args.duration);
    let client = cluster.random_validator_instance().rest_client();
    let mut treasury_compliance_account = cluster.load_tc_account(&client).await?;
    let mut designated_dealer_account = cluster.load_faucet_account(&client).await?;
    let mut emitter = TxnEmitter::new(
        &mut treasury_compliance_account,
        &mut designated_dealer_account,
        client,
        TransactionFactory::new(cluster.chain_id),
        StdRng::from_seed(OsRng.gen()),
    );
    let mut emit_job_request = EmitJobRequest::new(
        cluster
            .validator_instances()
            .iter()
            .map(Instance::rest_client)
            .collect(),
    )
    .accounts_per_client(args.accounts_per_client)
    .thread_params(thread_params)
    .invalid_transaction_ratio(args.invalid_tx);
    if let Some(workers_per_endpoint) = args.workers_per_ac {
        emit_job_request = emit_job_request.workers_per_endpoint(workers_per_endpoint);
    }
    let stats = emitter
        .emit_txn_for_with_stats(duration, emit_job_request, 10)
        .await?;
    println!("Total stats: {}", stats);
    println!("Average rate: {}", stats.rate(duration));
    Ok(())
}

async fn run_health_check(
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
        let result = health_check_runner
            .run(&events, &HashSet::new(), PrintFailures::All)
            .await;
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

        let cluster =
            Cluster::from_host_port(parsed_peers, &args.mint_file, args.chain_id, args.vasp);
        Self { cluster }
    }

    pub async fn diag(&self, vasp: bool) -> Result<()> {
        let client = self.cluster.random_validator_instance().rest_client();
        let mut treasury_compliance_account = self.cluster.load_tc_account(&client).await?;
        let mut designated_dealer_account = self.cluster.load_faucet_account(&client).await?;
        let emitter = TxnEmitter::new(
            &mut treasury_compliance_account,
            &mut designated_dealer_account,
            client,
            TransactionFactory::new(self.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );
        let mut faucet_account: Option<LocalAccount> = None;
        let instances: Vec<_> = self.cluster.validator_and_fullnode_instances().collect();
        for instance in &instances {
            let client = instance.rest_client();
            print!("Getting faucet account sequence number on {}...", instance);
            let account = if vasp {
                self.cluster
                    .load_dd_account(&client)
                    .await
                    .map_err(|e| format_err!("Failed to get dd account: {}", e))?
            } else {
                self.cluster
                    .load_faucet_account(&client)
                    .await
                    .map_err(|e| {
                        format_err!("Failed to get faucet account sequence number: {}", e)
                    })?
            };
            println!("seq={}", account.sequence_number());
            if let Some(faucet_account) = &faucet_account {
                if account.sequence_number() != faucet_account.sequence_number() {
                    bail!(
                        "Loaded sequence number {}, which is different from seen before {}",
                        account.sequence_number(),
                        faucet_account.sequence_number()
                    );
                }
            } else {
                faucet_account = Some(account);
            }
        }
        let mut faucet_account =
            faucet_account.expect("There is no faucet account set (not expected)");
        let faucet_account_address = faucet_account.address();
        for instance in &instances {
            print!("Submitting txn through {}...", instance);
            let deadline = emitter
                .submit_single_transaction(
                    &instance.rest_client(),
                    &mut faucet_account,
                    &faucet_account_address,
                    10,
                )
                .await
                .map_err(|e| format_err!("Failed to submit txn through {}: {}", instance, e))?;
            println!("seq={}", faucet_account.sequence_number());
            println!(
                "Waiting all full nodes to get to seq {}",
                faucet_account.sequence_number()
            );
            loop {
                let addresses = &[faucet_account_address];
                let clients = instances
                    .iter()
                    .map(|instance| instance.rest_client())
                    .collect::<Vec<_>>();
                let futures = clients
                    .iter()
                    .map(|client| query_sequence_numbers(client, addresses));
                let results = join_all(futures).await;
                let mut all_good = true;
                for (instance, result) in zip(instances.iter(), results) {
                    let seq = result.map_err(|e| {
                        format_err!("Failed to query sequence number from {}: {}", instance, e)
                    })?[0];
                    let ip = instance.ip();
                    let color = if seq != faucet_account.sequence_number() {
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
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        println!("Looks like all full nodes are healthy!");
        Ok(())
    }
}

impl ClusterTestRunner {
    pub async fn teardown(&mut self) {
        self.cluster_swarm.cleanup().await.expect("Cleanup failed");
        let workspace = self
            .cluster_swarm
            .get_workspace()
            .await
            .expect("Failed to get workspace");
        let asg_name = format!("{}-k8s-testnet-validators", workspace);
        aws::set_asg_size(0, 0.0, &asg_name, false, true)
            .await
            .unwrap_or_else(|_| panic!("{} scaling failed", asg_name));
    }

    /// Discovers cluster, setup log, etc
    pub async fn setup(args: &Args) -> Result<Self> {
        let start_time = Instant::now();
        let current_tag = args.deploy.as_deref().unwrap_or("master");
        let cluster_swarm = ClusterSwarmKube::new()
            .await
            .map_err(|e| format_err!("Failed to initialize ClusterSwarmKube: {}", e))?;
        let prometheus_ip = "diem-testnet-prometheus-server.default.svc.cluster.local";
        let grafana_base_url = cluster_swarm
            .get_grafana_baseurl()
            .await
            .expect("Failed to discover grafana url in k8s");
        let prometheus = Prometheus::new(prometheus_ip, grafana_base_url);
        let cluster_builder = ClusterBuilder::new(current_tag.to_string(), cluster_swarm.clone());
        let cluster_builder_params = args.cluster_builder_params.clone();
        let cluster = cluster_builder
            .setup_cluster(&cluster_builder_params, true)
            .await
            .map_err(|e| format_err!("Failed to setup cluster: {}", e))?;
        let log_tail_started = Instant::now();
        let logs = DebugPortLogWorker::spawn_new(&cluster);
        let log_tail_startup_time = Instant::now() - log_tail_started;
        info!(
            "Log tail thread started in {} ms",
            log_tail_startup_time.as_millis()
        );
        let health_check_runner = HealthCheckRunner::new_all(cluster.clone());
        let slack = SlackClient::new();
        let slack_changelog_url = env::var("SLACK_CHANGELOG_URL")
            .map(|u| u.parse().expect("Failed to parse SLACK_CHANGELOG_URL"))
            .ok();
        let client = cluster.random_validator_instance().rest_client();
        let root_account = cluster.load_diem_root_account(&client).await?;
        let treasury_compliance_account = cluster.load_tc_account(&client).await?;
        let designated_dealer_account = cluster.load_faucet_account(&client).await?;
        let github = GitHub::new();
        let mut report = SuiteReport::new();
        let end_time = (Instant::now() - start_time).as_secs() as u64;
        report.report_text(format!("Test runner setup time spent {} secs", end_time));
        let mut global_emit_job_request = EmitJobRequest::default()
            .accounts_per_client(args.accounts_per_client)
            .thread_params(EmitThreadParams {
                wait_millis: args.wait_millis,
                wait_committed: !args.burst,
            })
            .invalid_transaction_ratio(args.invalid_tx);
        if let Some(workers_per_endpoint) = args.workers_per_ac {
            global_emit_job_request =
                global_emit_job_request.workers_per_endpoint(workers_per_endpoint);
        }
        let emit_to_validator =
            if cluster.fullnode_instances().len() < cluster.validator_instances().len() {
                true
            } else {
                args.emit_to_validator.unwrap_or(false)
            };
        Ok(Self {
            logs,
            cluster_builder,
            cluster_builder_params,
            cluster,
            health_check_runner,
            slack,
            slack_changelog_url,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            prometheus,
            github,
            report,
            global_emit_job_request,
            emit_to_validator,
            cluster_swarm,
            current_tag: current_tag.to_string(),
        })
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
        let changelog = self.get_changelog(from_commit.as_ref(), to_commit);
        self.slack_changelog_message(format!("{}\n\n{}", changelog, perf_msg));
    }

    fn get_changelog(&self, prev_commit: Option<&String>, upstream_commit: &str) -> String {
        let commits = self.github.get_commits("diem/diem", upstream_commit);
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
                    let commit_head = commit_head.replace("[breaking]", "*[breaking]*");
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
            let start_time = Instant::now();
            let experiment_name = format!("{}", experiment);
            let experiment_result = self
                .run_single_experiment(experiment, None)
                .await
                .map_err(move |e| format_err!("Experiment `{}` failed: `{}`", experiment_name, e));
            let end_time = (Instant::now() - start_time).as_secs() as u64;
            if let Err(e) = experiment_result.as_ref() {
                self.report.report_text(e.to_string());
                self.print_report();
                experiment_result?;
            }
            self.report
                .report_text_same_line(format!(", time spent {} secs", end_time))
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
        info!(
            "\n====json-report-begin===\n{}\n====json-report-end===",
            json_report
        );
    }

    pub async fn run_named_suite(&mut self, name: &str) -> Result<String> {
        let suite = ExperimentSuite::new_by_name(&self.cluster, name)?;
        self.run_suite(suite).await?;
        Ok(self.report.to_string())
    }

    pub async fn run_and_report(&mut self, experiment: Box<dyn Experiment>) -> Result<()> {
        let experiment_name = format!("{}", experiment);
        match self
            .run_single_experiment(experiment, Some(self.global_emit_job_request.clone()))
            .await
        {
            Ok(_) => {
                self.print_report();
                Ok(())
            }
            Err(err) => {
                self.report.report_text(format!(
                    "Experiment `{}` failed: `{}`",
                    experiment_name, err
                ));
                self.print_report();
                Err(err)
            }
        }
    }

    pub async fn run_single_experiment(
        &mut self,
        experiment: Box<dyn Experiment>,
        global_emit_job_request: Option<EmitJobRequest>,
    ) -> Result<()> {
        let events = self.logs.recv_all();
        if let Err(s) = self
            .health_check_runner
            .run(&events, &HashSet::new(), PrintFailures::UnexpectedOnly)
            .await
        {
            bail!(
                "Some validators are unhealthy before experiment started : {}",
                s
            );
        }

        info!(
            "{}Starting experiment {}{}{}{}",
            Bold {},
            color::Fg(color::Blue),
            experiment.to_string(),
            color::Fg(color::Reset),
            Reset {}
        );

        let deadline = Instant::now() + experiment.deadline();

        self.experiment_loop(experiment, global_emit_job_request, deadline)
            .await?;

        info!(
            "{}Experiment finished, waiting until all affected validators recover{}",
            Bold {},
            Reset {}
        );

        self.wait_until_all_healthy(deadline).await?;

        info!("Experiment completed");
        Ok(())
    }

    // inner poll loop of run_single_experiment
    // do not use this fn, use run_single_experiment to run experiments
    async fn experiment_loop(
        &mut self,
        mut experiment: Box<dyn Experiment>,
        mut global_emit_job_request: Option<EmitJobRequest>,
        deadline: Instant,
    ) -> Result<()> {
        let affected_validators = experiment.affected_validators();
        let mut context = Context::new(
            &mut self.root_account,
            &mut self.treasury_compliance_account,
            &mut self.designated_dealer_account,
            &self.prometheus,
            &mut self.cluster_builder,
            &self.cluster_builder_params,
            &self.cluster,
            &mut self.report,
            &mut global_emit_job_request,
            self.emit_to_validator,
            &self.cluster_swarm,
            &self.current_tag[..],
        );
        let deadline_future = sleep_until(TokioInstant::from_std(deadline)).fuse();
        let mut run_future = experiment.run(&mut context).fuse();
        let sleep = sleep(HEALTH_POLL_INTERVAL).fuse();
        tokio::pin!(sleep);
        tokio::pin!(deadline_future);

        loop {
            select! {
                _delay = deadline_future => {
                    bail!("Experiment deadline reached");
                }
                result = run_future => {
                    return result.map_err(|e|format_err!("Failed to run experiment: {}", e));
                }
                _delay = sleep => {
                    let events = self.logs.recv_all();
                    if let Err(s) = self.health_check_runner.run(
                        &events,
                        &affected_validators,
                        PrintFailures::UnexpectedOnly,
                    ).await {
                        bail!("Validators which were not under experiment failed : {}", s);
                    }
                }
            }
        }
    }

    async fn wait_until_all_healthy(&mut self, deadline: Instant) -> Result<()> {
        info!("Waiting for all nodes to be healthy");
        for instance in self.cluster.validator_instances() {
            self.health_check_runner.invalidate(instance.peer_name());
        }
        loop {
            let now = Instant::now();
            if now > deadline {
                bail!("Nodes did not become healthy after deployment");
            }
            let deadline = now + HEALTH_POLL_INTERVAL;
            let events = self.logs.recv_all_until_deadline(deadline);
            if let Ok(failed_instances) = self
                .health_check_runner
                .run(&events, &HashSet::new(), PrintFailures::None)
                .await
            {
                if failed_instances.is_empty() {
                    break;
                }
            }
        }
        info!(
            "All nodes are now healthy. Checking json rpc endpoints of validators and full nodes"
        );
        loop {
            let results = join_all(
                self.cluster
                    .validator_and_fullnode_instances()
                    .map(Instance::try_rest_api),
            )
            .await;

            if results.iter().all(Result::is_ok) {
                break;
            }
            if Instant::now() > deadline {
                for (instance, result) in
                    zip(self.cluster.validator_and_fullnode_instances(), results)
                {
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

    pub async fn exec_on_pod(&self, pod: &str, cmd: &str) -> Result<()> {
        let instance = self
            .cluster
            .find_instance_by_pod(pod)
            .ok_or_else(|| format_err!("Can not find instance with pod {}", pod))?;
        instance.exec(cmd, false).await
    }
}

struct Bold {}

struct Reset {}

impl fmt::Debug for Bold {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl fmt::Display for Bold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", style::Bold)
    }
}

impl fmt::Debug for Reset {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl fmt::Display for Reset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", style::Reset)
    }
}
