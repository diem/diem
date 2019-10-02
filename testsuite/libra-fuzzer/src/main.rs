// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Helpers for fuzz testing.

use lazy_static::lazy_static;
use libra_fuzzer::{commands, FuzzTarget};
use std::{env, ffi::OsString, fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fuzzer", about = "Libra fuzzer")]
struct Opt {
    /// Print extended debug output
    #[structopt(long = "debug")]
    debug: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

/// The default number of items to generate in a corpus.
const GENERATE_DEFAULT_ITEMS: usize = 128;
lazy_static! {
    /// A stringified form of `GENERATE_DEFAULT_ITEMS`.
    ///
    /// Required because structopt only accepts strings as default values.
    static ref GENERATE_DEFAULT_ITEMS_STR: String = GENERATE_DEFAULT_ITEMS.to_string();
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Generate corpus for a particular fuzz target
    #[structopt(name = "generate")]
    Generate {
        /// Number of items to generate in the corpus
        #[structopt(
            short = "n",
            long = "num-items",
            default_value("&GENERATE_DEFAULT_ITEMS_STR")
        )]
        num_items: usize,
        /// Custom directory for corpus output to be stored in (required if not running under
        /// `cargo run`)
        #[structopt(long = "corpus-dir", parse(from_os_str))]
        corpus_dir: Option<PathBuf>,
        #[structopt(name = "TARGET")]
        /// Name of target to generate (use `list` to list)
        target: FuzzTarget,
    },
    /// Run fuzzer on specified target (must be run under `cargo run`)
    #[structopt(name = "fuzz", usage = "fuzzer fuzz <TARGET> -- [ARGS]")]
    Fuzz {
        /// Target to fuzz (use `list` to list targets)
        #[structopt(name = "TARGET", required = true)]
        target: FuzzTarget,
        /// Custom directory for corpus
        #[structopt(long = "corpus-dir", parse(from_os_str))]
        corpus_dir: Option<PathBuf>,
        /// Custom directory for artifacts
        #[structopt(long = "artifact-dir", parse(from_os_str))]
        artifact_dir: Option<PathBuf>,
        /// Arguments for `cargo fuzz run`
        #[structopt(name = "ARGS", parse(from_os_str), allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
    /// List fuzz targets
    #[structopt(name = "list")]
    List {
        /// Only print out names, no descriptions.
        #[structopt(long = "no-desc", short = "n")]
        no_desc: bool,
    },
}

/// The default directory for corpuses. Also return whether the directory was freshly created.
fn default_corpus_dir(target: FuzzTarget) -> (PathBuf, bool) {
    default_dir(target, "corpus")
}

/// The default directory for artifacts.
fn default_artifact_dir(target: FuzzTarget) -> PathBuf {
    default_dir(target, "artifacts").0
}

fn default_dir(target: FuzzTarget, intermediate_dir: &str) -> (PathBuf, bool) {
    let mut dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect(
        "--corpus-dir not set and this binary is not running under cargo run. \
         Either use cargo run or pass in the --corpus-dir flag.",
    ));
    // If a "fuzz" subdirectory doesn't exist, the user might be doing it wrong.
    dir.push("fuzz");
    if !dir.is_dir() {
        panic!(
            "Subdirectory {:?} of cargo manifest directory does not exist \
             (did you run `cargo fuzz init`?)",
            dir
        );
    }

    // The name of the corpus is derived from the name of the target.
    dir.push(intermediate_dir);
    dir.push(target.name());

    println!("Using default {} directory: {:?}", intermediate_dir, dir);
    let created = !dir.exists();
    fs::create_dir_all(&dir).expect("Failed to create directory");
    (dir, created)
}

fn main() {
    let opt: Opt = Opt::from_args();

    match opt.cmd {
        Command::Generate {
            num_items,
            corpus_dir,
            target,
        } => {
            let corpus_dir = corpus_dir.unwrap_or_else(|| default_corpus_dir(target).0);
            let item_count = commands::make_corpus(target, num_items, &corpus_dir, opt.debug)
                .expect("Failed to create corpus");
            println!("Wrote {} items to corpus", item_count);
        }
        Command::Fuzz {
            corpus_dir,
            artifact_dir,
            target,
            args,
        } => {
            let corpus_dir = match corpus_dir {
                Some(dir) => {
                    // Don't generate the corpus here -- custom directory means the user knows
                    // what they're doing.
                    dir
                }
                None => {
                    let (dir, created) = default_corpus_dir(target);
                    if created {
                        println!("New corpus, generating...");
                        commands::make_corpus(target, GENERATE_DEFAULT_ITEMS, &dir, opt.debug)
                            .expect("Failed to create corpus");
                    }
                    dir
                }
            };
            let artifact_dir = artifact_dir.unwrap_or_else(|| default_artifact_dir(target));
            commands::fuzz_target(target, corpus_dir, artifact_dir, args).unwrap();
        }
        Command::List { no_desc } => {
            commands::list_targets(no_desc);
        }
    }
}
