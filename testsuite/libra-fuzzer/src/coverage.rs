// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTarget;
use std::{env, fs, path::PathBuf};

//
// Fuzzing Coverage
// ================
//
// This test is here to test coverage of our fuzzers.
// The current tool we use (tarpaulin) only works on test,
// hence, we need this to be a test.
//
// Furthermore, `#[ignore]` for tests seems to be broken,
// so for now we run this only if some fuzzing corpus has been generated
// or if a `CORPUS_PATH` env var is set.
//

#[test]
#[ignore]
fn coverage() {
    // get path to corpus/ folder from env var CORPUS_PATH.
    let corpus_path = match env::var_os("CORPUS_PATH") {
        Some(corpus_path) => PathBuf::from(&corpus_path),
        None => {
            let crate_dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(&crate_dir).join("fuzz/corpus")
        }
    };

    // check if it exists
    assert!(
        corpus_path.exists(),
        "path to fuzzing corpus must exists or be provided"
    );

    // iterate over each target (corpus/<target>/...)
    for filepath in corpus_path
        .read_dir()
        .expect("CORPUS_PATH should be a readable directory")
    {
        // get target
        let filepath = filepath.unwrap().path();
        let target_name = filepath.file_name().unwrap().to_str().unwrap();
        let target = FuzzTarget::by_name(target_name)
            .unwrap_or_else(|| panic!("unknown fuzz target: {}", target_name));

        // run every corpus files for this target
        for corpus_file in filepath
            .read_dir()
            .expect("every corpus/<target> should be a readable directory")
        {
            let corpus_file = corpus_file.unwrap().path();
            let data = fs::read(corpus_file).expect("failed to read artifact");
            target.fuzz(&data);
        }
    }
}
