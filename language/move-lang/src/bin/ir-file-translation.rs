// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::ir_translation::fix_syntax_and_write;
use std::{fs, path::Path};
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "IR Test Translation",
    about = "Regex based translation tool for IR Functional Tests into the source language"
)]
pub struct Options {
    /// The IR file to translatee
    #[structopt(name = "FILE_TO_TRANSLATE", short = "i", long = "input")]
    pub input: String,
    /// The file to write the outpput
    #[structopt(name = "OUTPUT_FILE", short = "o", long = "output")]
    pub output: String,
}

pub fn main() -> std::io::Result<()> {
    let Options { input, output } = Options::from_args();
    let input_path = Path::new(&input);
    let contents = fs::read_to_string(input_path).unwrap();
    let output_path = Path::new(&output);
    fix_syntax_and_write(output_path, contents);
    Ok(())
}
