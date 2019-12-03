// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTarget;
use anyhow::{bail, format_err, Context, Result};
use libra_proptest_helpers::ValueGenerator;
use sha1::{Digest, Sha1};
use std::{
    env,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

/// Generates data for this fuzz target into the output directory. Returns the number of items
/// generated.
///
/// The corpus directory should be present at the time this method is called.
pub fn make_corpus(
    target: FuzzTarget,
    num_items: usize,
    corpus_dir: &Path,
    debug: bool,
) -> Result<usize> {
    // TODO: Allow custom proptest configs?
    let mut gen = ValueGenerator::new();

    let mut sha1 = Sha1::new();

    let mut idx = 0;
    while idx < num_items {
        let result = match target.generate(idx, &mut gen) {
            Some(bytes) => bytes,
            None => {
                // No value could be generated. Assume that corpus generation has been exhausted.
                break;
            }
        };

        // Use the SHA-1 of the result as the file name.
        sha1.input(&result);
        let hash = sha1.result_reset();
        let name = hex::encode(hash.as_slice());
        let path = corpus_dir.join(name);
        let mut f = fs::File::create(&path)
            .with_context(|| format!("Failed to create file: {:?}", path))?;
        if debug {
            println!("Writing {} bytes to file: {:?}", result.len(), path);
        }

        f.write_all(&result)
            .with_context(|| format!("Failed to write to file: {:?}", path))?;
        idx += 1;
    }
    Ok(idx)
}

/// Fuzz a target by running `cargo fuzz run`.
pub fn fuzz_target(
    target: FuzzTarget,
    corpus_dir: PathBuf,
    artifact_dir: PathBuf,
    mut args: Vec<OsString>,
) -> Result<()> {
    static FUZZ_RUNNER: &str = "fuzz_runner";

    // Do a bit of arg parsing -- look for a "--" and insert the target and corpus directory
    // before that.
    let dash_dash_pos = args.iter().position(|x| x == "--");
    let splice_pos = dash_dash_pos.unwrap_or_else(|| args.len());
    args.splice(
        splice_pos..splice_pos,
        vec![FUZZ_RUNNER.into(), corpus_dir.into()],
    );

    // The artifact dir goes at the end.
    if dash_dash_pos.is_none() {
        args.push("--".into());
    }
    let mut artifact_arg: OsString = "-artifact_prefix=".into();
    artifact_arg.push(&artifact_dir);
    // Add a trailing slash as required by libfuzzer to put the artifact in a directory.
    artifact_arg.push("/");
    args.push(artifact_arg);

    // Pass the target name in as an environment variable.
    // Use the manifest directory as the current one.
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or_else(|| {
        format_err!("Fuzzing requires CARGO_MANIFEST_DIR to be set (are you using `cargo run`?)")
    })?;

    let status = Command::new("cargo")
        .arg("fuzz")
        .arg("run")
        .args(args)
        .current_dir(manifest_dir)
        .env(FuzzTarget::ENV_VAR, target.name())
        // We want to fuzz with the same version of the compiler as production, but cargo-fuzz only
        // runs on nightly. This is a test-only environment so this use of RUSTC_BOOTSTRAP seems
        // appropriate.
        .env("RUSTC_BOOTSTRAP", "1")
        .status()
        .context("cargo fuzz run errored")?;
    if !status.success() {
        bail!("cargo fuzz run failed with status {}", status);
    }
    Ok(())
}

/// List all known fuzz targets.
pub fn list_targets(no_desc: bool) {
    for target in FuzzTarget::all_targets() {
        if no_desc {
            println!("{}", target.name())
        } else {
            println!("  * {0: <24}    {1}", target.name(), target.description())
        }
    }
}
