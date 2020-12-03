// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_fuzzer::FuzzTarget;
use std::{env, fs, path::PathBuf};

//
// Flamegraph
// ================
//
// See README for more information

fn main() {
    // get target
    let fuzz_target = env::var("FUZZ_TARGET").expect("pass fuzz target as FUZZ_TARGET env var");
    let fuzz_target = FuzzTarget::by_name(&fuzz_target)
        .expect("valid target passed as FUZZ_TARGET environment variable");

    // get path to corpus/ folder
    let mut corpus_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    corpus_dir.push("fuzz");
    corpus_dir.push("corpus");
    corpus_dir.push(fuzz_target.name());

    // check if it exists
    assert!(
        corpus_dir.exists(),
        "path to fuzzing corpus does not exist: {:?}",
        corpus_dir,
    );

    // run every corpus files for this target
    for corpus_file in corpus_dir.read_dir().unwrap() {
        let corpus_file = corpus_file.unwrap().path();
        let data = fs::read(corpus_file).expect("failed to read artifact");
        fuzz_target.fuzz(&data);
    }
}
