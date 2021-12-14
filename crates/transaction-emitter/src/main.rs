// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use diem_config::config::DEFAULT_PORT;
use diem_sdk::types::chain_id::ChainId;
use std::{process, time::Duration};
use structopt::StructOpt;
use transaction_emitter::{
    cluster::{Cluster, DPNCluster, NFTCluster},
    instance::Instance,
    EmitJobRequest, EmitThreadParams, Mode,
};

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short = "p", long, use_delimiter = true)]
    peers: Vec<String>,

    #[structopt(long, help = "If set, tries to use public peers instead of localhost")]
    vasp: bool,

    #[structopt(long)]
    health_check: bool,
    #[structopt(long)]
    emit_tx: bool,
    #[structopt(long)]
    diag: bool,

    // emit_tx options
    #[structopt(long, default_value = "15")]
    accounts_per_client: usize,
    #[structopt(long)]
    workers_per_ac: Option<usize>,
    #[structopt(long, default_value = "0")]
    wait_millis: u64,
    #[structopt(long)]
    burst: bool,
    // experiment
    #[structopt(long)]
    nft: bool,
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
}

#[tokio::main]
pub async fn main() {
    let args = Args::from_args();

    if !args.emit_tx && !args.diag {
        panic!("Can only use --emit-tx or --diag mode");
    }

    if args.peers.is_empty() {
        panic!("Peers not set in args");
    }
    let parsed_peers: Vec<_> = args
        .peers
        .iter()
        .map(|peer| parse_host_port(peer).expect("Failed to parse host_port"))
        .collect();
    let cluster: Box<dyn Cluster> = if !args.nft {
        println!("Initialize DPN cluster");
        Box::new(DPNCluster::from_host_port(
            parsed_peers,
            &args.mint_file,
            args.chain_id,
            args.vasp,
        ))
    } else {
        println!("Initialize NFT cluster");
        Box::new(NFTCluster::from_host_port(
            parsed_peers,
            &args.mint_file,
            args.chain_id,
        ))
    };
    if args.diag {
        exit_on_error(cluster.diag().await);
        return;
    } else if args.emit_tx {
        exit_on_error(emit_tx(&*cluster, &args).await);
        return;
    }

    exit_on_error(cluster.diag().await);
}

async fn emit_tx(cluster: &dyn Cluster, args: &Args) -> Result<()> {
    let thread_params = EmitThreadParams {
        wait_millis: args.wait_millis,
        wait_committed: !args.burst,
    };
    let duration = Duration::from_secs(args.duration);
    let mut emitter = cluster.gen_txn_emitter().await?;
    let mode = if args.nft {
        Mode::NFT
    } else {
        Mode::DPN {
            gas_price: 0,
            vasp: args.vasp,
        }
    };
    let mut emit_job_request = EmitJobRequest::new(
        mode,
        cluster.all_instances().map(Instance::rest_client).collect(),
    )
    .accounts_per_client(args.accounts_per_client)
    .thread_params(thread_params)
    .invalid_transaction_ratio(args.invalid_tx);
    if let Some(workers_per_endpoint) = args.workers_per_ac {
        emit_job_request = emit_job_request.workers_per_endpoint(workers_per_endpoint);
    }
    if args.vasp {
        emit_job_request = emit_job_request.vasp();
    }
    let stats = emitter
        .emit_txn_for_with_stats(duration, emit_job_request, 10)
        .await?;
    println!("Total stats: {}", stats);
    println!("Average rate: {}", stats.rate(duration));
    Ok(())
}

fn parse_host_port(s: &str) -> Result<(String, u32, Option<u32>)> {
    let v = s.split(':').collect::<Vec<&str>>();
    if v.len() == 1 {
        let default_port = DEFAULT_PORT as u32;
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

fn exit_on_error<T>(r: Result<T>) -> T {
    match r {
        Ok(r) => r,
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    }
}
