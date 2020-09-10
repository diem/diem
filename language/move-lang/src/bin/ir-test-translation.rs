// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::{ir_translation::fix_syntax_and_write, test_utils::*};
use regex::Regex;
use std::{fs, path::Path};
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "IR Test Translation",
    about = "Regex based translation tool for IR Functional Tests into the source language"
)]
pub struct Options {
    /// The IR dirctory to translate
    #[structopt(name = "DIRECTORY_TO_TRANSLATE", short = "d", long = "directory")]
    pub directory: String,
}

const PATH_TO_IR_TESTS: &str = "../ir-testsuite/tests";

pub fn main() -> std::io::Result<()> {
    let Options { directory } = Options::from_args();
    let main_regex = Regex::new(r".*main\(.*\)\s*\{").unwrap();
    for (subdir, name) in ir_tests().filter(|(subdir, _)| subdir == &directory) {
        let pname = format!("{}/{}/{}", PATH_TO_IR_TESTS, subdir, name);
        let path = Path::new(&pname);
        let basename = path.file_stem().unwrap().to_str().unwrap();
        let contents = fs::read_to_string(path).unwrap();

        let has_main = main_regex.is_match(&contents);
        let out_name = match translated_ir_test_name(has_main, &subdir, basename) {
            None => continue,
            Some(n) => n,
        };
        let out_path = Path::new(&out_name);
        let parent_dir = out_path.parent().unwrap();
        if !parent_dir.is_dir() {
            fs::create_dir_all(parent_dir).unwrap();
        }

        fix_syntax_and_write(out_path, contents);
    }
    Ok(())
}
