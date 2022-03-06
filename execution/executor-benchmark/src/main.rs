// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::MetricsPusher;
use std::path::PathBuf;
use structopt::StructOpt;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "500")]
    block_size: usize,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    CreateDb {
        #[structopt(long, parse(from_os_str))]
        data_dir: PathBuf,

        #[structopt(long, default_value = "1000000")]
        num_accounts: usize,

        #[structopt(long, default_value = "1000000")]
        init_account_balance: u64,

        #[structopt(long)]
        prune_window: Option<u64>,
    },
    RunExecutor {
        #[structopt(
            long,
            default_value = "1000",
            about = "number of transfer blocks to run"
        )]
        blocks: usize,

        #[structopt(long, parse(from_os_str))]
        data_dir: PathBuf,

        #[structopt(long, parse(from_os_str))]
        checkpoint_dir: PathBuf,

        #[structopt(
            long,
            about = "Verify sequence number of all the accounts after execution finishes"
        )]
        verify: bool,
    },
}

fn main() {
    let _mp = MetricsPusher::start();
    let opt = Opt::from_args();

    rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("rayon-global-{}", index))
        .num_threads(2)
        .build_global()
        .expect("Failed to build rayon global thread pool.");

    match opt.cmd {
        Command::CreateDb {
            data_dir,
            num_accounts,
            init_account_balance,
            prune_window,
        } => {
            executor_benchmark::db_generator::run(
                num_accounts,
                init_account_balance,
                opt.block_size,
                data_dir,
                prune_window,
            );
        }
        Command::RunExecutor {
            blocks,
            data_dir,
            checkpoint_dir,
            verify,
        } => {
            diem_logger::Logger::new().init();
            executor_benchmark::run_benchmark(
                opt.block_size,
                blocks,
                data_dir,
                checkpoint_dir,
                verify,
            );
        }
    }
}
