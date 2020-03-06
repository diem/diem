// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use move_prover::cli::Options;
use move_prover::run_move_prover;
use std::env;

fn main() -> anyhow::Result<()> {
    let mut options = Options::default();
    let args: Vec<String> = env::args().collect();
    options.initialize_from_args(&args);
    options.setup_logging();
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    run_move_prover(&mut error_writer, options)
}
