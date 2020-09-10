// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{env, fs::File, io::prelude::*, path::PathBuf};

fn main() {
    // needed to build different binaries based on env var
    println!("cargo:rerun-if-env-changed=SINGLE_FUZZ_TARGET");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    let fuzz_target = match env::var("SINGLE_FUZZ_TARGET") {
        Ok(x) => x,
        // default value for build to work
        Err(_) => "vm_value".to_string(),
    };

    // fuzzer file to write
    let fuzzer_content = format!(
        "const FUZZ_TARGET: &str = \"{fuzz_target}\";",
        fuzz_target = fuzz_target,
    );

    // path of file to create (OUT_DIR/fuzzer.rs)
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_path.join("fuzzer.rs");

    // write to file
    let mut file = File::create(out_path).unwrap();
    file.write_all(fuzzer_content.as_bytes()).unwrap();
}
