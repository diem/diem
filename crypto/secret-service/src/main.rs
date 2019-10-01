// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executable_helpers::helpers::setup_executable;
use secret_service::secret_service_node;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Secret Service")]
struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

/// Run a SecretService in its own process.
fn main() {
    let args = Args::from_args();

    let (config, _logger) =
        setup_executable(args.config.as_ref().map(PathBuf::as_path), args.no_logging);

    let secret_service_node = secret_service_node::SecretServiceNode::new(config);

    secret_service_node
        .run()
        .expect("Unable to run SecretService");
}
