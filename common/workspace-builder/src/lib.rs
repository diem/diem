// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use once_cell::sync::Lazy;
use std::{env, path::PathBuf, process::Command};

const WORKSPACE_BUILD_ERROR_MSG: &str = r#"
    Unable to build all workspace binaries. Cannot continue running tests.

    Try running 'cargo build --all --bins --exclude cluster-test' yourself.
"#;

// Global flag indicating if all binaries in the workspace have been built.
static WORKSPACE_BUILT: Lazy<bool> = Lazy::new(|| {
    info!("Building project binaries");
    let args = if cfg!(debug_assertions) {
        // special case: excluding cluster-test as it exports no-struct-opt feature that poisons everything
        vec!["build", "--all", "--bins", "--exclude", "cluster-test"]
    } else {
        vec!["build", "--all", "--bins", "--release"]
    };

    let cargo_build = Command::new("cargo")
        .current_dir(workspace_root())
        .args(&args)
        .output()
        .expect(WORKSPACE_BUILD_ERROR_MSG);
    if cargo_build.status.success() {
        info!("Finished building project binaries");
        true
    } else {
        false
    }
});

// Path to top level workspace
pub fn workspace_root() -> PathBuf {
    let mut path = build_dir();
    while !path.ends_with("target") {
        path.pop();
    }
    path.pop();
    path
}

// Path to the directory where build artifacts live.
//TODO maybe add an Environment Variable which points to built binaries
fn build_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .expect("Can't find the build directory. Cannot continue running tests")
}

// Path to a specified binary
pub fn get_bin<S: AsRef<str>>(bin_name: S) -> PathBuf {
    // We have to check to see if the workspace is built first to ensure that the binaries we're
    // testing are up to date.
    if !*WORKSPACE_BUILT {
        panic!(WORKSPACE_BUILD_ERROR_MSG);
    }

    let bin_name = bin_name.as_ref();
    let bin_path = build_dir().join(format!("{}{}", bin_name, env::consts::EXE_SUFFIX));

    // If the binary doesn't exist then either building them failed somehow or the supplied binary
    // name doesn't match any binaries this workspace can produce.
    if !bin_path.exists() {
        panic!(format!(
            "Can't find binary '{}' in expected path {:?}",
            bin_name, bin_path
        ));
    }

    bin_path
}
