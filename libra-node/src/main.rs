// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executable_helpers::helpers::setup_executable;
use signal_hook;
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

fn register_signals(term: Arc<AtomicBool>) {
    for signal in &[
        signal_hook::SIGTERM,
        signal_hook::SIGINT,
        signal_hook::SIGHUP,
    ] {
        let term_clone = Arc::clone(&term);
        let thread = std::thread::current();
        unsafe {
            signal_hook::register(*signal, move || {
                term_clone.store(true, Ordering::Release);
                thread.unpark();
            })
            .expect("failed to register signal handler");
        }
    }
}

fn main() {
    let args = Args::from_args();

    let (mut config, _logger) =
        setup_executable(args.config.as_ref().map(PathBuf::as_path), args.no_logging);

    let _node_handle = libra_node::main_node::setup_environment(&mut config);

    let term = Arc::new(AtomicBool::new(false));
    register_signals(Arc::clone(&term));

    while !term.load(Ordering::Acquire) {
        std::thread::park();
    }
}
