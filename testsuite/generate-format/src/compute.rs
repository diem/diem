// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::{add_deserialization_tracing, add_proptest_serialization_tracing, FILE_PATH};
use serde_reflection::Tracer;
use serde_yaml;
use std::{fs::File, io::Write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra format generator",
    about = "Trace serde (de)serialization to generate format descriptions for Libra types"
)]
struct Options {
    #[structopt(short, long)]
    with_deserialize: bool,

    #[structopt(short, long)]
    record: bool,
}

fn main() {
    let options = Options::from_args();

    let mut tracer = Tracer::new(lcs::is_human_readable());
    tracer = add_proptest_serialization_tracing(tracer);
    if options.with_deserialize {
        tracer = add_deserialization_tracing(tracer);
    }

    let registry = tracer.registry().unwrap();
    let content = serde_yaml::to_string(&registry).unwrap();
    if options.record {
        let mut f = File::create("testsuite/generate-format/".to_string() + FILE_PATH).unwrap();
        writeln!(f, "{}", content).unwrap();
    } else {
        println!("{}", content);
    }
}
