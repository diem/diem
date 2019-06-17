// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{value_t, App, Arg};
use generate_keypair::create_faucet_key_file;

const OUTPUT_ARG: &str = "output";

fn main() {
    let args = App::new("Libra Key Generation Tool")
        .version("0.1.0")
        .author("Libra Association <opensource@libra.org>")
        .about("Tool to generate public/private keypairs")
        .arg(
            Arg::with_name(OUTPUT_ARG)
                .short("o")
                .long(OUTPUT_ARG)
                .takes_value(true)
                .help("Output file path.  Keypair is written to this file"),
        )
        .get_matches();

    let output_file =
        value_t!(args, OUTPUT_ARG, String).expect("Missing output file path argument");
    create_faucet_key_file(&output_file);
}
