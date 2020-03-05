// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use executable_helpers::helpers::setup_executable;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Node")]
struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();

    let mut config = setup_executable(args.config.as_deref(), args.no_logging);

    let _node_handle = libra_node::main_node::setup_environment(&mut config);

    let term = Arc::new(AtomicBool::new(false));

    while !term.load(Ordering::Acquire) {
        std::thread::park();
    }
}
