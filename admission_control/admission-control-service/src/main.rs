// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_service::admission_control_node;
use executable_helpers::helpers::setup_executable;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Admission Control Service")]
struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

/// Run a Admission Control service in its own process.
/// It will also setup global logger and initialize config.
fn main() {
    let args = Args::from_args();

    let (config, _logger) =
        setup_executable(args.config.as_ref().map(PathBuf::as_path), args.no_logging);

    let admission_control_node = admission_control_node::AdmissionControlNode::new(config);

    admission_control_node
        .run()
        .expect("Unable to run AdmissionControl node");
}
