// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::env;

use slog::{o, Drain};
use slog_scope::info;
use structopt::{clap::ArgGroup, StructOpt};

use anyhow::{format_err, Result};
use cluster_test::tx_emitter::{EmitJobRequest, EmitThreadParams};
use cluster_test::{cluster::Cluster, tx_emitter::TxEmitter};

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("action"))]
// example: transaction_gen -p 39.98.196.244:8001 --emit_tx=true --mint-file=sgterraform/sgchain/dev/mint.key
struct Args {
    #[structopt(short = "p", long, use_delimiter = true)]
    peers: Vec<String>,

    #[structopt(
        long,
        help = "If set, tries to connect to a libra-swarm instead of aws"
    )]
    swarm: bool,

    #[structopt(short = "e", long = "emit_tx")]
    emit_tx: bool,

    // emit_tx options
    #[structopt(long, default_value = "10")]
    accounts_per_client: usize,
    #[structopt(long, default_value = "50")]
    wait_millis: u64,
    #[structopt(long)]
    burst: bool,
    #[structopt(long, default_value = "mint.key")]
    mint_file: String,
}

pub fn main() {
    setup_log();

    let args = Args::from_args();

    if args.emit_tx {
        let thread_params = EmitThreadParams {
            wait_millis: args.wait_millis,
            wait_committed: !args.burst,
        };
        let util = ClusterUtil::setup(&args);
        util.emit_tx(args.accounts_per_client, thread_params);
        return;
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

struct ClusterUtil {
    cluster: Cluster,
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

impl ClusterUtil {
    pub fn setup(args: &Args) -> Self {
        //parse peers
        let parsed_peers: Vec<_> = args
            .peers
            .iter()
            .map(|peer| parse_host_port(peer).unwrap())
            .collect();
        //        from_host_port(peers: Vec<(String, u32)>, mint_file: &str)
        let cluster = Cluster::from_host_port(parsed_peers, &args.mint_file);
        let cluster = if args.peers.is_empty() {
            cluster
        } else {
            cluster.sub_cluster(args.peers.clone())
        };
        info!(
            "Discovered {} peers in  workspace",
            cluster.instances().len()
        );
        Self { cluster }
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
    }
}
