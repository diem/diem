// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::Corpus;
use std::{fs::File, io::Write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra format generator",
    about = "Trace serde (de)serialization to generate format descriptions for Libra types"
)]
struct Options {
    #[structopt(long, possible_values = &Corpus::variants(), default_value = "Libra", case_insensitive = true)]
    corpus: Corpus,

    #[structopt(long)]
    record: bool,
}

fn main() {
    let options = Options::from_args();

    let registry = options.corpus.get_registry();
    let output_file = options.corpus.output_file();

    let content = serde_yaml::to_string(&registry).unwrap();
    if options.record {
        match output_file {
            Some(path) => {
                let mut f = File::create("testsuite/generate-format/".to_string() + path).unwrap();
                writeln!(f, "{}", content).unwrap();
            }
            None => panic!("Corpus {:?} doesn't record formats on disk", options.corpus),
        }
    } else {
        println!("{}", content);
    }
}
